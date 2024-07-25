use std::{
    ffi::c_uint,
    mem::{self, transmute},
    rc::Rc,
};

use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, Win32WindowHandle, WindowsDisplayHandle,
};
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Input::KeyboardAndMouse::*,
            WindowsAndMessaging::{
                AdjustWindowRect, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect,
                GetWindowLongPtrW, LoadCursorW, PeekMessageW, PostQuitMessage, RegisterClassExW,
                SetWindowLongPtrW, ShowWindow, TranslateMessage, CREATESTRUCTW, CS_HREDRAW,
                CS_VREDRAW, CW_USEDEFAULT, GWLP_HINSTANCE, GWLP_USERDATA, IDC_ARROW, MSG,
                PM_REMOVE, SW_SHOW, WINDOW_EX_STYLE, WM_CREATE, WM_DESTROY, WM_KEYDOWN, WM_KEYUP,
                WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_PAINT, WM_QUIT, WM_RBUTTONDOWN,
                WM_RBUTTONUP, WM_SIZE, WM_SYSKEYDOWN, WM_SYSKEYUP, WNDCLASSEXW,
                WS_OVERLAPPEDWINDOW,
            },
        },
    },
};

use crate::{EventHandler, WindowAttribs, WindowEvent};

pub struct PlatformWindow {
    hwnd: HWND,
    data: Rc<WindowsWindow>,
}

pub struct WindowsWindow {
    pub attribs: WindowAttribs,
    event_handler: Option<Box<EventHandler>>,
}

impl PlatformWindow {
    pub fn create_window(attribs: crate::WindowAttribs) -> Self {
        let width = attribs.width;
        let height = attribs.height;
        let app_name = windows::core::w!("Placeholder"); // TODO use attribs

        let h_instance = unsafe { GetModuleHandleW(None) }.expect("failed to create module handle");

        let wnd_class = WNDCLASSEXW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            hInstance: h_instance.into(),
            lpszClassName: app_name,
            cbClsExtra: 0,
            cbWndExtra: 0,
            hIcon: Default::default(),
            hCursor: unsafe { LoadCursorW(None, IDC_ARROW) }.expect("Failed to load cursor"),
            hbrBackground: Default::default(),
            lpszMenuName: PCWSTR::null(),
            cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
            hIconSm: Default::default(),
        };

        let class_atom = unsafe { RegisterClassExW(&wnd_class) };
        if class_atom == 0 {
            panic!("Failed register class.");
        }

        let mut windows_window = WindowsWindow {
            attribs,
            event_handler: None,
        };

        let mut window_rect = RECT {
            left: 0,
            top: 0,
            right: width.get() as i32,
            bottom: height.get() as i32,
        };
        unsafe {
            AdjustWindowRect(&mut window_rect, WS_OVERLAPPEDWINDOW, None)
                .expect("Failed to adjust window rect")
        };

        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(), // Default style
                app_name,
                app_name, // Title
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                window_rect.right - window_rect.left,
                window_rect.bottom - window_rect.top,
                None, // Parent
                None, // Menu
                h_instance,
                Some(&mut windows_window as *mut _ as _),
            )
        }
        .expect("Failed to create window.");

        unsafe { _ = ShowWindow(hwnd, SW_SHOW) };

        Self {
            hwnd,
            data: windows_window.into(),
        }
    }

    pub fn poll<F>(&mut self, event_handler: F)
    where
        F: FnMut(WindowEvent) + 'static,
    {
        let data = Rc::get_mut(&mut self.data).unwrap();
        data.event_handler = Some(Box::new(event_handler));
        let input_ptr = Box::into_raw(Box::new(data));
        let create_struct: &CREATESTRUCTW = unsafe {
            &*(input_ptr as *const windows::Win32::UI::WindowsAndMessaging::CREATESTRUCTW)
        };
        unsafe { SetWindowLongPtrW(self.hwnd, GWLP_USERDATA, create_struct.lpCreateParams as _) };

        loop {
            let mut msg = MSG::default();

            if unsafe { PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE) }.into() {
                unsafe {
                    _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }

                // self.data.event_sender.send(WindowEvent::Draw).unwrap();

                #[cfg(debug_assertions)]
                {
                    // Currently we get error 87 all the time
                    /*     let s = unsafe { GetLastError() };
                        if s.0 != 0 {
                            let info = unsafe { GetErrorInfo(s.0) };
                            eprintln!("Error {}", s.0);
                            if let Ok(info) = info {
                                if let Ok(desc) = unsafe { info.GetDescription() } {
                                    eprintln!("Description {}", desc);
                                }
                            }
                        }
                    */
                }

                if msg.message == WM_QUIT {
                    break;
                }
            }
        }
    }

    pub fn set_cursor_visible(&mut self, visible: bool) {
        todo!()
    }

    pub fn width(&self) -> u32 {
        self.data.attribs.width.into()
    }

    pub fn height(&self) -> u32 {
        self.data.attribs.height.into()
    }

    pub fn raw_display_handle(&self) -> RawDisplayHandle {
        RawDisplayHandle::Windows(WindowsDisplayHandle::new())
    }

    pub fn raw_window_handle(&self) -> RawWindowHandle {
        let mut window_handle = Win32WindowHandle::new(unsafe {
            std::num::NonZeroIsize::new_unchecked((self.hwnd.0 as i32).try_into().unwrap())
        });
        let hinstance = unsafe {
            windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(self.hwnd, GWLP_HINSTANCE)
        };
        window_handle.hinstance = std::num::NonZeroIsize::new(hinstance);
        RawWindowHandle::Win32(window_handle)
    }

    pub fn close(&mut self) {
        unsafe { PostQuitMessage(0) };
    }
}

/// # Safety
///
/// ---
pub unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: c_uint,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            unsafe {
                let create_struct: &CREATESTRUCTW = transmute(l_param);
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, create_struct.lpCreateParams as _);
            }
            LRESULT::default()
        }
        _ => {
            // Get the user data associated with the window
            let user_data_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
            let data = std::ptr::NonNull::<WindowsWindow>::new(user_data_ptr as _);
            // Dereference the pointer to access the actual data
            let handled = data.map_or(false, |mut s| {
                s.as_mut().progress_message(hwnd, msg, w_param, l_param)
            });
            if handled {
                LRESULT::default()
            } else {
                unsafe { DefWindowProcW(hwnd, msg, w_param, l_param) }
            }
        }
    }
}

impl WindowsWindow {
    pub fn progress_message(
        &mut self,
        hwnd: HWND,
        message: c_uint,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> bool {
        match message {
            WM_DESTROY => {
                if let Some(handler) = self.event_handler.as_mut() {
                    handler(WindowEvent::Close)
                }
                unsafe { PostQuitMessage(0) };
                true
            }
            WM_PAINT => {
                if let Some(handler) = self.event_handler.as_mut() {
                    handler(WindowEvent::Draw)
                }
                true
            }
            WM_SIZE => {
                let mut rect = RECT::default();
                unsafe { GetClientRect(hwnd, &mut rect).expect("Failed to get Client rect area") };
                if let Some(handler) = self.event_handler.as_mut() {
                    handler(WindowEvent::Resize {
                        new_width: rect.right as u32,
                        new_height: rect.bottom as u32,
                    });
                }
                true
            }
            WM_SYSKEYDOWN | WM_KEYDOWN => {
                let vkcode = loword(wparam.0 as u32);
                if let Some(handler) = self.event_handler.as_mut() {
                    handler(WindowEvent::Key {
                        key: convert_key(VIRTUAL_KEY(vkcode)),
                        state: crate::keyboard::KeyState::Pressed,
                    });
                }
                true
            }
            WM_SYSKEYUP | WM_KEYUP => {
                let vkcode = loword(wparam.0 as u32);
                if let Some(handler) = self.event_handler.as_mut() {
                    handler(WindowEvent::Key {
                        key: convert_key(VIRTUAL_KEY(vkcode)),
                        state: crate::keyboard::KeyState::Released,
                    });
                }
                true
            }
            WM_RBUTTONDOWN => {
                if let Some(handler) = self.event_handler.as_mut() {
                    handler(WindowEvent::MouseButton {
                        button: crate::mouse::Button::RIGHT,
                        state: crate::mouse::ButtonState::Pressed,
                    });
                }
                true
            }
            WM_RBUTTONUP => {
                if let Some(handler) = self.event_handler.as_mut() {
                    handler(WindowEvent::MouseButton {
                        button: crate::mouse::Button::RIGHT,
                        state: crate::mouse::ButtonState::Released,
                    });
                }
                true
            }
            WM_LBUTTONUP => {
                if let Some(handler) = self.event_handler.as_mut() {
                    handler(WindowEvent::MouseButton {
                        button: crate::mouse::Button::LEFT,
                        state: crate::mouse::ButtonState::Released,
                    });
                }
                true
            }
            WM_LBUTTONDOWN => {
                if let Some(handler) = self.event_handler.as_mut() {
                    handler(WindowEvent::MouseButton {
                        button: crate::mouse::Button::LEFT,
                        state: crate::mouse::ButtonState::Pressed,
                    });
                }
                true
            }
            WM_MOUSEMOVE => {
                let x = get_x_lparam(lparam.0 as u32) as i32;
                let y = get_y_lparam(lparam.0 as u32) as i32;
                if let Some(handler) = self.event_handler.as_mut() {
                    handler(WindowEvent::MouseMotion {
                        x: x as f64,
                        y: y as f64,
                    });
                }
                true
            }
            _ => false,
        }
    }
}
use crate::keyboard::Key;

fn convert_key(key: VIRTUAL_KEY) -> Key {
    match key {
        VK_A => Key::A,
        VK_B => Key::B,
        VK_C => Key::C,
        VK_D => Key::D,
        VK_E => Key::E,
        VK_F => Key::F,
        VK_G => Key::G,
        VK_H => Key::H,
        VK_I => Key::I,
        VK_J => Key::J,
        VK_K => Key::K,
        VK_L => Key::L,
        VK_M => Key::M,
        VK_N => Key::N,
        VK_O => Key::O,
        VK_P => Key::P,
        VK_Q => Key::Q,
        VK_R => Key::R,
        VK_S => Key::S,
        VK_T => Key::T,
        VK_U => Key::U,
        VK_V => Key::V,
        VK_W => Key::W,
        VK_X => Key::X,
        VK_Y => Key::Y,
        VK_Z => Key::Z,
        VK_SPACE => Key::Space,
        VK_LSHIFT => Key::ShiftL,
        VK_RSHIFT => Key::ShiftR,
        VK_LEFT => Key::Leftarrow,
        VK_RIGHT => Key::Rightarrow,
        VK_UP => Key::Uparrow,
        VK_DOWN => Key::Downarrow,
        _ => Key::Unknown,
    }
}

#[inline(always)]
pub(crate) const fn loword(x: u32) -> u16 {
    (x & 0xffff) as u16
}

#[inline(always)]
const fn hiword(x: u32) -> u16 {
    ((x >> 16) & 0xffff) as u16
}

#[inline(always)]
const fn get_x_lparam(x: u32) -> i16 {
    loword(x) as _
}

#[inline(always)]
const fn get_y_lparam(x: u32) -> i16 {
    hiword(x) as _
}
