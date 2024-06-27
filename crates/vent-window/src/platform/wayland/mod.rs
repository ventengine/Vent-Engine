use std::{
    num::NonZeroU32,
    ptr::NonNull,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
    time::Duration,
};

use sctk::{reexports::protocols::xdg::shell::client::xdg_toplevel::ResizeEdge as XdgResizeEdge, seat::pointer::{ThemeSpec, ThemedPointer}};

use rwh_06::{RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle};
use sctk::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_keyboard, delegate_output, delegate_pointer, delegate_registry,
    delegate_seat, delegate_shm, delegate_subcompositor, delegate_xdg_shell, delegate_xdg_window,
    output::{OutputHandler, OutputState},
    reexports::{
        calloop::{self, EventLoop},
        calloop_wayland_source::WaylandSource,
    },
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        keyboard::KeyboardHandler,
        pointer::{PointerData, PointerEventKind, PointerHandler},
        Capability, SeatHandler, SeatState,
    },
    shell::{
        xdg::{
            window::{DecorationMode, Window, WindowDecorations, WindowHandler},
            XdgShell, XdgSurface,
        },
        WaylandSurface,
    },
    shm::{Shm, ShmHandler},
    subcompositor::SubcompositorState,
};
use sctk_adwaita::{AdwaitaFrame, FrameConfig};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{
        wl_keyboard, wl_output,
        wl_pointer::{self},
        wl_seat, wl_surface,
    },
    Connection, Proxy, QueueHandle,
};
use wayland_csd_frame::{CursorIcon, DecorationsFrame, FrameAction, FrameClick, ResizeEdge};
use xkbcommon::xkb;

use crate::{
    keyboard::{Key, KeyState},
    mouse, WindowAttribs, WindowEvent,
};

pub struct PlatformWindow {
    pub connection: Connection,
    event_loop: calloop::EventLoop<'static, WaylandWindow>,
    state: WaylandWindow,
}

struct WaylandWindow {
    running: bool,
    pub attribs: WindowAttribs,

    registry_state: RegistryState,
    shm_state: Shm,
    seat_state: SeatState,
    output_state: OutputState,
    compositor_state: Arc<CompositorState>,
    subcompositor_state: Arc<SubcompositorState>,
    _xdg_shell_state: XdgShell,

    // sctk window
    window: Window,

    configured: bool,

    // Decoration, Not every Window Manager supports Server Side decorations, Gnome ;D
    window_frame: Option<AdwaitaFrame<Self>>,
    decorations_cursor: Option<CursorIcon>,

    // Input
    keyboard: Option<wl_keyboard::WlKeyboard>,
    set_cursor: bool,
    keyboard_focus: bool,
    themed_pointer: Option<ThemedPointer>,

    event_sender: Sender<WindowEvent>,
    event_receiver: Receiver<WindowEvent>,
}

delegate_compositor!(WaylandWindow);
delegate_subcompositor!(WaylandWindow);
delegate_output!(WaylandWindow);
delegate_shm!(WaylandWindow);

delegate_seat!(WaylandWindow);
delegate_keyboard!(WaylandWindow);
delegate_pointer!(WaylandWindow);

delegate_xdg_shell!(WaylandWindow);
delegate_xdg_window!(WaylandWindow);

delegate_registry!(WaylandWindow);

impl WaylandWindow {
    fn setup_attribs(&mut self) {
        let window = &self.window;
        let attribs = &self.attribs;
        window.set_title(attribs.title.clone());
        // TODO WindowMode
        window.set_app_id(attribs.app_id.clone());
        window.set_max_size(attribs.max_size);
        window.set_min_size(attribs.min_size);
        window.commit();
    }

    fn frame_action(&mut self, pointer: &wl_pointer::WlPointer, serial: u32, action: FrameAction) {
        let pointer_data = pointer.data::<PointerData>().unwrap();
        let seat = pointer_data.seat();
        match action {
            FrameAction::Close => self.event_sender.send(WindowEvent::Close).unwrap(),
            FrameAction::Minimize => self.window.set_minimized(),
            FrameAction::Maximize => self.window.set_maximized(),
            FrameAction::UnMaximize => self.window.unset_maximized(),
            FrameAction::ShowMenu(x, y) => self.window.show_window_menu(seat, serial, (x, y)),
            FrameAction::Resize(edge) => {
                let edge = match edge {
                    ResizeEdge::None => XdgResizeEdge::None,
                    ResizeEdge::Top => XdgResizeEdge::Top,
                    ResizeEdge::Bottom => XdgResizeEdge::Bottom,
                    ResizeEdge::Left => XdgResizeEdge::Left,
                    ResizeEdge::TopLeft => XdgResizeEdge::TopLeft,
                    ResizeEdge::BottomLeft => XdgResizeEdge::BottomLeft,
                    ResizeEdge::Right => XdgResizeEdge::Right,
                    ResizeEdge::TopRight => XdgResizeEdge::TopRight,
                    ResizeEdge::BottomRight => XdgResizeEdge::BottomRight,
                    _ => return,
                };
                self.window.resize(seat, serial, edge);
            }
            FrameAction::Move => self.window.move_(seat, serial),
            _ => (),
        }
    }

    fn draw(&mut self, conn: &Connection, qh: &QueueHandle<WaylandWindow>) {
        // Draw cursor
        if self.set_cursor {
            if let Some(icon) = self.decorations_cursor {
                let _ = self.themed_pointer.as_mut().unwrap().set_cursor(conn, icon);
            }
            self.set_cursor = false;
        }

        // Draw the decorations frame.
        if let Some(frame) = self.window_frame.as_mut() {
            if frame.is_dirty() && !frame.is_hidden() {
                frame.draw();
            }
        }
        self.event_sender.send(WindowEvent::Draw).unwrap();
        self.window.wl_surface().damage_buffer(
            0,
            0,
            self.attribs.width.get() as i32,
            self.attribs.height.get() as i32,
        );
        self.window
            .wl_surface()
            .frame(qh, self.window.wl_surface().clone());
        self.window.commit();
    }
}

impl CompositorHandler for WaylandWindow {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        surface: &wl_surface::WlSurface,
        new_factor: i32,
    ) {
        if self.window.wl_surface() == surface {
            if let Some(frame) = self.window_frame.as_mut() {
                frame.set_scaling_factor(new_factor as f64);
            }
        }
    }

    fn transform_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        self.draw(conn, qh)
    }

    fn surface_enter(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        surface: &wl_surface::WlSurface,
        output: &wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        surface: &wl_surface::WlSurface,
        output: &wl_output::WlOutput,
    ) {
    }
}

impl OutputHandler for WaylandWindow {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
    }
}

impl ProvidesRegistryState for WaylandWindow {
    fn registry(&mut self) -> &mut sctk::registry::RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState,];
}

impl KeyboardHandler for WaylandWindow {
    fn enter(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        keyboard: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        serial: u32,
        raw: &[u32],
        keysyms: &[xkb::Keysym],
    ) {
        self.keyboard_focus = true;
    }

    fn leave(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        keyboard: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        serial: u32,
    ) {
        self.keyboard_focus = false;
    }

    fn press_key(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        keyboard: &wl_keyboard::WlKeyboard,
        serial: u32,
        event: sctk::seat::keyboard::KeyEvent,
    ) {
        self.event_sender
            .send(WindowEvent::Key {
                key: convert_key(event.keysym.raw()),
                state: KeyState::Pressed,
            })
            .expect("Failed to send key event");
    }

    fn release_key(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        keyboard: &wl_keyboard::WlKeyboard,
        serial: u32,
        event: sctk::seat::keyboard::KeyEvent,
    ) {
        self.event_sender
            .send(WindowEvent::Key {
                key: convert_key(event.keysym.raw()),
                state: KeyState::Released,
            })
            .expect("Failed to send key event");
    }

    fn update_modifiers(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        keyboard: &wl_keyboard::WlKeyboard,
        serial: u32,
        modifiers: sctk::seat::keyboard::Modifiers,
        layout: u32,
    ) {
    }
}

// Taken from <linux/input-event-codes.h>.
const BTN_LEFT: u32 = 0x110;
const BTN_RIGHT: u32 = 0x111;
const BTN_MIDDLE: u32 = 0x112;
const BTN_SIDE: u32 = 0x113;
const BTN_EXTRA: u32 = 0x114;
const BTN_FORWARD: u32 = 0x115;
const BTN_BACK: u32 = 0x116;

impl PointerHandler for WaylandWindow {
    fn pointer_frame(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        pointer: &wl_pointer::WlPointer,
        events: &[sctk::seat::pointer::PointerEvent],
    ) {
        for event in events {
            let (x, y) = event.position;
            match event.kind {
                PointerEventKind::Enter { serial } => {
                    self.decorations_cursor = self.window_frame.as_mut().and_then(|frame| {
                        frame.click_point_moved(Duration::ZERO, &event.surface.id(), x, y)
                    });
                }
                PointerEventKind::Leave { serial } => {
                    if &event.surface != self.window.wl_surface() {
                        if let Some(window_frame) = self.window_frame.as_mut() {
                            window_frame.click_point_left();
                        }
                    }
                }
                PointerEventKind::Motion { time } => {
                    if let Some(new_cursor) = self.window_frame.as_mut().and_then(|frame| {
                        frame.click_point_moved(
                            Duration::from_millis(time as u64),
                            &event.surface.id(),
                            x,
                            y,
                        )
                    }) {
                        self.set_cursor = true;
                        self.decorations_cursor = Some(new_cursor);
                    }
                }
                PointerEventKind::Press {
                    button,
                    serial,
                    time,
                }
                | PointerEventKind::Release {
                    button,
                    serial,
                    time,
                } => {
                    let pressed = matches!(event.kind, PointerEventKind::Press { .. });
                    if &event.surface != self.window.wl_surface() {
                        let click = match button {
                            0x110 => FrameClick::Normal,
                            0x111 => FrameClick::Alternate,
                            _ => continue,
                        };

                        if let Some(action) = self.window_frame.as_mut().and_then(|frame| {
                            frame.on_click(Duration::from_millis(time as u64), click, pressed)
                        }) {
                            self.frame_action(pointer, serial, action);
                        }
                    }
                    let mouse_state = if pressed {
                        mouse::ButtonState::Pressed
                    } else {
                        mouse::ButtonState::Released
                    };
                    press_mouse(button, self, mouse_state);
                }
                _ => {}
            }
        }
    }
}

fn press_mouse(button: u32, state: &WaylandWindow, mouse_state: mouse::ButtonState) {
    match button {
        BTN_LEFT => state
            .event_sender
            .send(WindowEvent::Mouse {
                button: crate::mouse::Button::LEFT,
                state: mouse_state,
            })
            .unwrap(),
        BTN_RIGHT => state
            .event_sender
            .send(WindowEvent::Mouse {
                button: crate::mouse::Button::RIGHT,
                state: mouse_state,
            })
            .unwrap(),
        BTN_MIDDLE => state
            .event_sender
            .send(WindowEvent::Mouse {
                button: crate::mouse::Button::MIDDLE,
                state: mouse_state,
            })
            .unwrap(),
        BTN_SIDE => state
            .event_sender
            .send(WindowEvent::Mouse {
                button: crate::mouse::Button::SIDE,
                state: mouse_state,
            })
            .unwrap(),
        BTN_EXTRA => state
            .event_sender
            .send(WindowEvent::Mouse {
                button: crate::mouse::Button::EXTRA,
                state: mouse_state,
            })
            .unwrap(),
        BTN_FORWARD => state
            .event_sender
            .send(WindowEvent::Mouse {
                button: crate::mouse::Button::FORWARD,
                state: mouse_state,
            })
            .unwrap(),
        BTN_BACK => state
            .event_sender
            .send(WindowEvent::Mouse {
                button: crate::mouse::Button::BACK,
                state: mouse_state,
            })
            .unwrap(),
        _ => (),
    }
}

impl SeatHandler for WaylandWindow {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, conn: &Connection, qh: &QueueHandle<Self>, seat: wl_seat::WlSeat) {}

    fn new_capability(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: sctk::seat::Capability,
    ) {
        if capability == Capability::Keyboard && self.keyboard.is_none() {
            println!("Set keyboard capability");
            let keyboard = self
                .seat_state
                .get_keyboard(qh, &seat, None)
                .expect("Failed to create keyboard");
            self.keyboard = Some(keyboard);
        }

        if capability == Capability::Pointer && self.themed_pointer.is_none() {
            let surface = self.compositor_state.create_surface(qh);
            let themed_pointer = self
                .seat_state
                .get_pointer_with_theme(
                    qh,
                    &seat,
                    self.shm_state.wl_shm(),
                    surface,
                    ThemeSpec::default(),
                )
                .expect("Failed to create pointer");
            self.themed_pointer.replace(themed_pointer);
        }
    }

    fn remove_capability(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: sctk::seat::Capability,
    ) {
        if capability == Capability::Keyboard && self.keyboard.is_some() {
            println!("Unset keyboard capability");
            self.keyboard.take().unwrap().release();
        }

        if capability == Capability::Pointer && self.themed_pointer.is_some() {
            println!("Unset pointer capability");
            self.themed_pointer.take().unwrap().pointer().release();
        }
    }

    fn remove_seat(&mut self, conn: &Connection, qh: &QueueHandle<Self>, seat: wl_seat::WlSeat) {}
}

impl ShmHandler for WaylandWindow {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm_state
    }
}
impl WindowHandler for WaylandWindow {
    fn request_close(&mut self, conn: &Connection, qh: &QueueHandle<Self>, window: &Window) {
        self.event_sender
            .send(WindowEvent::Close)
            .expect("Failed to send Close Event");
    }

    fn configure(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        window: &Window,
        configure: sctk::shell::xdg::window::WindowConfigure,
        serial: u32,
    ) {
        let (width, height) = if configure.decoration_mode == DecorationMode::Client {
            let window_frame = self.window_frame.get_or_insert_with(|| {
                let mut frame = AdwaitaFrame::new(
                    &self.window,
                    &self.shm_state,
                    self.compositor_state.clone(),
                    self.subcompositor_state.clone(),
                    qh.clone(),
                    FrameConfig::auto(),
                )
                .expect("failed to create client side decorations frame.");
                frame.set_title(self.attribs.title.clone());
                frame
            });

            // Un-hide the frame.
            window_frame.set_hidden(false);

            // Configure state before touching any resizing.
            window_frame.update_state(configure.state);

            // Configure the button state.
            window_frame.update_wm_capabilities(configure.capabilities);

            let (width, height) = match configure.new_size {
                (Some(width), Some(height)) => {
                    // The size could be 0.
                    window_frame.subtract_borders(width, height)
                }
                _ => {
                    // You might want to consider checking for configure bounds.
                    (Some(self.attribs.width), Some(self.attribs.height))
                }
            };

            // Clamp the size to at least one pixel.
            let width = width.unwrap_or(NonZeroU32::new(1).unwrap());
            let height = height.unwrap_or(NonZeroU32::new(1).unwrap());

            window_frame.resize(width, height);

            let (x, y) = window_frame.location();
            let outer_size = window_frame.add_borders(width.get(), height.get());
            window.xdg_surface().set_window_geometry(
                x,
                y,
                outer_size.0 as i32,
                outer_size.1 as i32,
            );

            (width, height)
        } else {
            // Hide the frame, if any.
            if let Some(frame) = self.window_frame.as_mut() {
                frame.set_hidden(true)
            }
            let width = configure.new_size.0.unwrap_or(self.attribs.width);
            let height = configure.new_size.1.unwrap_or(self.attribs.height);
            self.window.xdg_surface().set_window_geometry(
                0,
                0,
                width.get() as i32,
                height.get() as i32,
            );
            (width, height)
        };

        // Update new width and height;
        self.attribs.width = width;
        self.attribs.height = height;
        self.event_sender
            .send(WindowEvent::Resize {
                new_width: width.into(),
                new_height: height.into(),
            })
            .unwrap();

        // Initiate the first draw.
        if self.configured {
            self.configured = false;
            self.draw(conn, qh);
        }
    }
}

impl PlatformWindow {
    pub fn create_window(attribs: WindowAttribs) -> Self {
        let conn = wayland_client::Connection::connect_to_env().expect("Failed to get connection");
        log::debug!("Connected to Wayland Server");

        let (event_sender, event_receiver) = channel::<WindowEvent>();

        let (globals, event_queue) = registry_queue_init(&conn).unwrap();
        let qhandle = event_queue.handle();
        let event_loop: EventLoop<WaylandWindow> =
            EventLoop::try_new().expect("Failed to initialize the event loop!");
        let loop_handle = event_loop.handle();
        WaylandSource::new(conn.clone(), event_queue)
            .insert(loop_handle)
            .unwrap();

        let registry_state = RegistryState::new(&globals);
        let seat_state = SeatState::new(&globals, &qhandle);
        let output_state = OutputState::new(&globals, &qhandle);
        let compositor_state =
            CompositorState::bind(&globals, &qhandle).expect("wl_compositor not available");
        let subcompositor_state =
            SubcompositorState::bind(compositor_state.wl_compositor().clone(), &globals, &qhandle)
                .expect("wl_subcompositor not available");
        let shm_state = Shm::bind(&globals, &qhandle).expect("wl_shm not available");
        let xdg_shell_state = XdgShell::bind(&globals, &qhandle).expect("xdg shell not available");

        let window_surface = compositor_state.create_surface(&qhandle);

        let window = xdg_shell_state.create_window(
            window_surface,
            WindowDecorations::ServerDefault,
            &qhandle,
        );

        let mut state = WaylandWindow {
            running: true,
            attribs,
            configured: true,
            output_state,
            seat_state,
            event_receiver,
            event_sender,
            _xdg_shell_state: xdg_shell_state,
            registry_state,
            shm_state,
            compositor_state: compositor_state.into(),
            subcompositor_state: subcompositor_state.into(),
            window,
            window_frame: None,
            keyboard: None,
            keyboard_focus: false,
            decorations_cursor: None,
            set_cursor: false,
            themed_pointer: None,
        };

        state.setup_attribs();

        PlatformWindow {
            connection: conn,
            state,
            event_loop,
        }
    }

    pub fn poll<F>(mut self, mut event_handler: F)
    where
        F: FnMut(WindowEvent),
    {
        while self.state.running {
            self.connection.flush().unwrap();
            self.event_loop
                .dispatch(Duration::from_millis(16), &mut self.state)
                .expect("Failed to dispatch pending");

            while let Ok(event) = self.state.event_receiver.try_recv() {
                event_handler(event);
            }
            //   self.state.draw();
        }
    }

    pub fn width(&self) -> u32 {
        self.state.attribs.width.into()
    }

    pub fn height(&self) -> u32 {
        self.state.attribs.height.into()
    }

    pub fn raw_display_handle(&self) -> RawDisplayHandle {
        RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
            NonNull::new(self.connection.display().id().as_ptr().cast()).unwrap(),
        ))
    }

    pub fn raw_window_handle(&self) -> RawWindowHandle {
        let ptr = self.state.window.wl_surface().id().as_ptr();
        RawWindowHandle::Wayland(WaylandWindowHandle::new(
            NonNull::new(ptr as *mut _).unwrap(),
        ))
    }

    pub fn close(&mut self) {}
}

impl Drop for PlatformWindow {
    fn drop(&mut self) {
        self.close()
    }
}

fn convert_key(raw_key: xkeysym::RawKeysym) -> Key {
    match raw_key {
        xkeysym::key::A | xkeysym::key::a => Key::A,
        xkeysym::key::B | xkeysym::key::b => Key::B,
        xkeysym::key::C | xkeysym::key::c => Key::C,
        xkeysym::key::D | xkeysym::key::d => Key::D,
        xkeysym::key::E | xkeysym::key::e => Key::E,
        xkeysym::key::F | xkeysym::key::f => Key::F,
        xkeysym::key::G | xkeysym::key::g => Key::G,
        xkeysym::key::H | xkeysym::key::h => Key::H,
        xkeysym::key::I | xkeysym::key::i => Key::I,
        xkeysym::key::J | xkeysym::key::j => Key::J,
        xkeysym::key::K | xkeysym::key::k => Key::K,
        xkeysym::key::L | xkeysym::key::l => Key::L,
        xkeysym::key::M | xkeysym::key::m => Key::M,
        xkeysym::key::N | xkeysym::key::n => Key::N,
        xkeysym::key::O | xkeysym::key::o => Key::O,
        xkeysym::key::P | xkeysym::key::p => Key::P,
        xkeysym::key::Q | xkeysym::key::q => Key::Q,
        xkeysym::key::R | xkeysym::key::r => Key::R,
        xkeysym::key::S | xkeysym::key::s => Key::S,
        xkeysym::key::T | xkeysym::key::t => Key::T,
        xkeysym::key::U | xkeysym::key::u => Key::U,
        xkeysym::key::V | xkeysym::key::v => Key::V,
        xkeysym::key::W | xkeysym::key::w => Key::W,
        xkeysym::key::X | xkeysym::key::x => Key::X,
        xkeysym::key::Y | xkeysym::key::y => Key::Y,
        xkeysym::key::Z | xkeysym::key::z => Key::Z,

        xkeysym::key::space => Key::Space,
        xkeysym::key::Shift_L => Key::ShiftL,
        xkeysym::key::Shift_R => Key::ShiftR,
        xkeysym::key::leftarrow => Key::Leftarrow,
        xkeysym::key::uparrow => Key::Uparrow,
        xkeysym::key::rightarrow => Key::Rightarrow,
        xkeysym::key::downarrow => Key::Downarrow,

        _ => {
            log::warn!("Unknown key {}", raw_key);
            Key::Unknown
        }
    }
}
