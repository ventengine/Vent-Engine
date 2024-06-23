use std::{ffi::c_uint, ptr::null_mut};

use rwh_06::{RawDisplayHandle, RawWindowHandle, Win32WindowHandle, WindowsDisplayHandle};
use windows_sys::Win32::{
    Foundation::{self, HWND, LPARAM, LRESULT, WPARAM},
    System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, LoadCursorW,
        PostQuitMessage, RegisterClassW, ShowWindow, TranslateMessage, CS_HREDRAW, CS_OWNDC,
        CS_VREDRAW, CW_USEDEFAULT, GWLP_HINSTANCE, IDC_ARROW, MSG, SW_SHOW, WM_DESTROY, WNDCLASSW,
        WS_OVERLAPPEDWINDOW, WS_VISIBLE,
    },
};

use crate::WindowEvent;

pub struct PlatformWindow {
    hwnd: isize,
    width: u32,
    height: u32,
}

impl PlatformWindow {
    pub fn create_window(attribs: crate::WindowAttribs) -> Self {
        let app_name = windows_sys::core::w!("WinAPIApp");

        let h_instance = unsafe { GetModuleHandleW(null_mut()) };

        let wnd_class = WNDCLASSW {
            style: CS_OWNDC | CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            hInstance: h_instance,
            lpszClassName: app_name,
            cbClsExtra: 0,
            cbWndExtra: 0,
            hIcon: 0,
            hCursor: unsafe { LoadCursorW(0, IDC_ARROW) },
            hbrBackground: 0,
            lpszMenuName: null_mut(),
        };

        let class_atom = unsafe { RegisterClassW(&wnd_class) };

        let hwnd = unsafe {
            CreateWindowExW(
                0,
                class_atom as *const u16,
                app_name,
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                0,
                0,
                h_instance,
                null_mut(),
            )
        };

        if hwnd == 0 {
            panic!("Failed to create window.");
        }

        unsafe { ShowWindow(hwnd, SW_SHOW) };

        Self {
            hwnd,
            width: attribs.width.into(),
            height: attribs.height.into(),
        }
    }

    pub fn poll<F>(self, event_handler: F)
    where
        F: FnMut(WindowEvent),
    {
        let mut msg = MSG {
            hwnd: 0,
            message: 0,
            wParam: 0,
            lParam: 0,
            time: 0,
            pt: Foundation::POINT { x: 0, y: 0 },
        };

        while unsafe { GetMessageW(&mut msg, 0, 0, 0) } != 0 {
            unsafe {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn raw_display_handle(&self) -> RawDisplayHandle {
        RawDisplayHandle::Windows(WindowsDisplayHandle::new())
    }

    pub fn raw_window_handle(&self) -> RawWindowHandle {
        let mut window_handle =
            Win32WindowHandle::new(unsafe { std::num::NonZeroIsize::new_unchecked(self.hwnd) });
        let hinstance = unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                self.hwnd,
                GWLP_HINSTANCE,
            )
        };
        window_handle.hinstance = std::num::NonZeroIsize::new(hinstance);
        RawWindowHandle::Win32(window_handle)
    }

    pub fn close(&mut self) {
        todo!()
    }
}

pub unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: c_uint,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match msg {
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, w_param, l_param),
    }
}
