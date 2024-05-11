use std::{fs::File, os::fd::AsFd, ptr::NonNull};

use rwh_06::{
    DisplayHandle, RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle,
};
use wayland_client::{
    backend::protocol::WEnumError,
    delegate_noop,
    globals::{registry_queue_init, GlobalListContents},
    protocol::{
        wl_buffer, wl_compositor,
        wl_display::WlDisplay,
        wl_keyboard,
        wl_registry::{self, WlRegistry},
        wl_seat, wl_shm, wl_shm_pool, wl_surface,
    },
    Connection, Dispatch, EventQueue, Proxy, QueueHandle, WEnum,
};
use wayland_protocols::xdg::{decoration::zv1::client::zxdg_decoration_manager_v1::ZxdgDecorationManagerV1, shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base}};
use wayland_protocols_plasma::server_decoration;

use crate::{Window, WindowAttribs, WindowEvent};

pub struct PlatformWindow {
    pub display: WlDisplay,
    event_queue: EventQueue<State>,
    state: State,
}

struct State {
    running: bool,
    pub width: u32,
    pub height: u32,
    base_surface: Option<wl_surface::WlSurface>,
    buffer: Option<wl_buffer::WlBuffer>,
    wm_base: Option<xdg_wm_base::XdgWmBase>,
    xdg_surface: Option<(xdg_surface::XdgSurface, xdg_toplevel::XdgToplevel)>,
    configured: bool,
}

delegate_noop!(State: ignore wl_surface::WlSurface);
delegate_noop!(State: ignore wl_shm::WlShm);
delegate_noop!(State: ignore wl_shm_pool::WlShmPool);
delegate_noop!(State: ignore wl_buffer::WlBuffer);

impl State {
    fn init_xdg_surface(&mut self, qh: &QueueHandle<State>) {
        let wm_base = self.wm_base.as_ref().unwrap();
        let base_surface = self.base_surface.as_ref().unwrap();

        let xdg_surface = wm_base.get_xdg_surface(base_surface, qh, ());
        let toplevel = xdg_surface.get_toplevel(qh, ());
        toplevel.set_title("Vent Engine!".into());
        toplevel.set_app_id("com.ventengine.VentEngine".into());

        self.xdg_surface = Some((xdg_surface, toplevel));
    }
}

impl Dispatch<wl_keyboard::WlKeyboard, ()> for State {
    fn event(
        state: &mut Self,
        _: &wl_keyboard::WlKeyboard,
        event: wl_keyboard::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_keyboard::Event::Key { key, .. } = event {
            if key == 1 {
                // ESC key
                state.running = false;
            }
        }
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for State {
    fn event(
        data: &mut Self,
        seat: &wl_seat::WlSeat,
        event: wl_seat::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_seat::Event::Capabilities {
            capabilities: WEnum::Value(capabilities),
        } = event
        {
            if capabilities.contains(wl_seat::Capability::Keyboard) {
                seat.get_keyboard(qh, ());
            }
        }
    }
}

impl Dispatch<xdg_wm_base::XdgWmBase, ()> for State {
    fn event(
        _: &mut Self,
        wm_base: &xdg_wm_base::XdgWmBase,
        event: xdg_wm_base::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_wm_base::Event::Ping { serial } = event {
            wm_base.pong(serial);
        }
    }
}

impl Dispatch<xdg_surface::XdgSurface, ()> for State {
    fn event(
        state: &mut Self,
        xdg_surface: &xdg_surface::XdgSurface,
        event: xdg_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_surface::Event::Configure { serial, .. } = event {
            xdg_surface.ack_configure(serial);
            state.configured = true;
            let surface = state.base_surface.as_ref().unwrap();
            if let Some(ref buffer) = state.buffer {
                surface.attach(Some(buffer), 0, 0);
                surface.commit();
            }
        }
    }
}

impl Dispatch<xdg_toplevel::XdgToplevel, ()> for State {
    fn event(
        state: &mut Self,
        _: &xdg_toplevel::XdgToplevel,
        event: xdg_toplevel::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_toplevel::Event::Close {} = event {
            state.running = false;
        }
        if let xdg_toplevel::Event::Configure {
            width,
            height,
            states,
        } = event
        {
            state.width = width as u32;
            state.height = height as u32;
        }
    }
}

impl wayland_client::Dispatch<wl_registry::WlRegistry, GlobalListContents> for State {
    fn event(
        state: &mut Self,
        proxy: &wl_registry::WlRegistry,
        event: <wl_registry::WlRegistry as Proxy>::Event,
        data: &GlobalListContents,
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_compositor::WlCompositor, ()> for State {
    fn event(
        state: &mut Self,
        proxy: &wl_compositor::WlCompositor,
        event: <wl_compositor::WlCompositor as Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZxdgDecorationManagerV1, ()> for State {
    fn event(
        state: &mut Self,
        proxy: &ZxdgDecorationManagerV1,
        event: <ZxdgDecorationManagerV1 as Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        todo!()
    }
}


impl PlatformWindow {
    pub fn create_window(attribs: &WindowAttribs) -> Self {
        let conn = wayland_client::Connection::connect_to_env().expect("Failed to get connection");
        println!("Connected to Wayland Server");

        let mut state = State {
            running: true,
            width: attribs.width,
            height: attribs.height,
            base_surface: None,
            buffer: None,
            wm_base: None,
            xdg_surface: None,
            configured: false,
        };

        let display = conn.display();

        let (globals, event_queue) = registry_queue_init::<State>(&conn).unwrap();
        let qhandle = event_queue.handle();

        let wm_base = globals.bind(&event_queue.handle(), 4..=5, ()).unwrap();
        state.wm_base = Some(wm_base);

        let compositor: wl_compositor::WlCompositor =
            globals.bind(&event_queue.handle(), 4..=5, ()).unwrap();
        let surface = compositor.create_surface(&qhandle, ());
        state.base_surface = Some(surface);

        if state.wm_base.is_some() && state.xdg_surface.is_none() {
            state.init_xdg_surface(&qhandle);
        }

        let wl_seat: wl_seat::WlSeat = globals.bind(&event_queue.handle(), 4..=5, ()).unwrap();
        state
            .xdg_surface
            .as_ref()
            .unwrap()
            .1
            .show_window_menu(&wl_seat, 0, 0, 0);
        state.base_surface.as_ref().unwrap().commit();
      //  let xdg_decoration_manager: ZxdgDecorationManagerV1  = globals.bind(&event_queue.handle(), 1..=1, ()).unwrap();


      PlatformWindow {
            display,
            state,
            event_queue,
        }
    }

    pub fn poll<F>(mut self, mut event_handler: F)
    where
        F: FnMut(WindowEvent),
    {
        while self.state.running {
            self.event_queue
                .dispatch_pending(&mut self.state)
                .expect("Failed to dispatch pending");

            event_handler(WindowEvent::Draw);
        }
    }

    pub fn width(&self) -> u32 {
        self.state.width
    }

    pub fn height(&self) -> u32 {
        self.state.height
    }

    pub fn raw_display_handle(&self) -> RawDisplayHandle {
        RawDisplayHandle::Wayland(
            WaylandDisplayHandle::new(NonNull::new(self.display.id().as_ptr().cast()).unwrap())
                .into(),
        )
    }

    pub fn raw_window_handle(&self) -> RawWindowHandle {
        let ptr = self.state.base_surface.as_ref().unwrap().id().as_ptr();
        RawWindowHandle::Wayland(
            WaylandWindowHandle::new(NonNull::new(ptr as *mut _).unwrap()).into(),
        )
    }
}

impl Drop for PlatformWindow {
    fn drop(&mut self) {
        self.event_queue
            .flush()
            .expect("Failed to flush Event Queue");
    }
}
