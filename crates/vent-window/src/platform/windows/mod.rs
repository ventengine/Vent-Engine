use std::{
    ffi::c_uint,
    mem::{self},
    sync::mpsc::{sync_channel, Receiver, SyncSender},
};

use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, Win32WindowHandle, WindowsDisplayHandle,
};
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            AdjustWindowRect, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect,
            LoadCursorW, PeekMessageW, PostQuitMessage, RegisterClassExW, ShowWindow,
            TranslateMessage, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, GWLP_HINSTANCE, IDC_ARROW,
            MSG, PM_REMOVE, SW_SHOW, WINDOW_EX_STYLE, WM_DESTROY, WM_PAINT, WM_QUIT, WM_SIZE,
            WNDCLASSEXW, WS_OVERLAPPEDWINDOW,
        },
    },
};

use crate::{WindowAttribs, WindowEvent};

pub struct PlatformWindow {
    hwnd: HWND,
    data: WindowsWindow,
}

pub struct WindowsWindow {
    pub attribs: WindowAttribs,
    event_sender: SyncSender<WindowEvent>,
    event_receiver: Receiver<WindowEvent>,
}

impl PlatformWindow {
    pub fn create_window(attribs: crate::WindowAttribs) -> Self {
        let width = attribs.width;
        let height = attribs.height;
        let app_name = windows::core::w!("Placeholder");

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

        let (event_sender, event_receiver) = sync_channel::<WindowEvent>(1);

        let windows_window = WindowsWindow {
            attribs,
            event_sender,
            event_receiver,
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
                None,
            )
        }
        .expect("Failed to create window.");

        unsafe { _ = ShowWindow(hwnd, SW_SHOW) };

        Self {
            hwnd,
            data: windows_window,
        }
    }

    pub fn poll<F>(mut self, mut event_handler: F)
    where
        F: FnMut(WindowEvent),
    {
        loop {
            let mut msg = MSG::default();

            if unsafe { PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE) }.into() {
                unsafe {
                    _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }

                while let Ok(event) = self.data.event_receiver.try_recv() {
                    event_handler(event);
                }

                if msg.message == WM_QUIT {
                    break;
                } else {
                    self.data
                        .progress_message(msg.hwnd, msg.message, msg.wParam, msg.lParam)
                }
                //  self.data.event_sender.send(WindowEvent::Draw).unwrap();
            }
        }
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

pub unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: c_uint,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    unsafe { DefWindowProcW(hwnd, msg, w_param, l_param) }
}

impl WindowsWindow {
    pub fn progress_message(
        &mut self,
        hwnd: HWND,
        message: c_uint,
        wparam: WPARAM,
        lparam: LPARAM,
    ) {
        match message {
            WM_DESTROY => {
                self.event_sender.send(WindowEvent::Close).unwrap();
                unsafe { PostQuitMessage(0) };
            }
            WM_PAINT => {
                self.event_sender.send(WindowEvent::Draw).unwrap();
            }
            WM_SIZE => {
                let mut rect = RECT::default();
                unsafe { GetClientRect(hwnd, &mut rect).expect("Failed to get Client rect area") };
                self.event_sender
                    .send(WindowEvent::Resize {
                        new_width: rect.right as u32,
                        new_height: rect.bottom as u32,
                    })
                    .unwrap();
            }
            _ => {}
        }
    }
}
