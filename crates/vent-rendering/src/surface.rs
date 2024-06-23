// Mostly taken from ash-window: https://github.com/ash-rs/ash/blob/master/ash-window/src/lib.rs

use std::os::raw::c_char;

use ash::{
    khr::{surface},
    prelude::*,
    vk, Entry, Instance,
};
use raw_window_handle::{DisplayHandle, RawDisplayHandle, RawWindowHandle, WindowHandle};

pub unsafe fn create_surface(
    entry: &Entry,
    instance: &Instance,
    display_handle: DisplayHandle<'_>,
    window_handle: WindowHandle<'_>,
    allocation_callbacks: Option<&vk::AllocationCallbacks>,
) -> VkResult<vk::SurfaceKHR> {
    match (display_handle.as_raw(), window_handle.as_raw()) {
        #[cfg(target_os = "windows")]
        (RawDisplayHandle::Windows(_), RawWindowHandle::Win32(window)) => {
            let surface_desc = vk::Win32SurfaceCreateInfoKHR::default()
                .hwnd(window.hwnd.get())
                .hinstance(
                    window
                        .hinstance
                        .ok_or(vk::Result::ERROR_INITIALIZATION_FAILED)?
                        .get(),
                );
            let surface_fn = win32_surface::Instance::new(entry, instance);
            surface_fn.create_win32_surface(&surface_desc, allocation_callbacks)
        }

        #[cfg(target_os = "linux")]
        (RawDisplayHandle::Wayland(display), RawWindowHandle::Wayland(window)) => {
            use ash::khr::wayland_surface;

            let surface_desc = vk::WaylandSurfaceCreateInfoKHR::default()
                .display(display.display.as_ptr())
                .surface(window.surface.as_ptr());
            let surface_fn = wayland_surface::Instance::new(entry, instance);
            surface_fn.create_wayland_surface(&surface_desc, allocation_callbacks)
        }

        #[cfg(target_os = "linux")]
        (RawDisplayHandle::Xlib(display), RawWindowHandle::Xlib(window)) => {
            use ash::khr::xlib_surface;

            let surface_desc = vk::XlibSurfaceCreateInfoKHR::default()
                .dpy(
                    display
                        .display
                        .ok_or(vk::Result::ERROR_INITIALIZATION_FAILED)?
                        .as_ptr(),
                )
                .window(window.window);
            let surface_fn = xlib_surface::Instance::new(entry, instance);
            surface_fn.create_xlib_surface(&surface_desc, allocation_callbacks)
        }

        #[cfg(target_os = "linux")]
        (RawDisplayHandle::Xcb(display), RawWindowHandle::Xcb(window)) => {
            use ash::khr::xcb_surface;

            let surface_desc = vk::XcbSurfaceCreateInfoKHR::default()
                .connection(
                    display
                        .connection
                        .ok_or(vk::Result::ERROR_INITIALIZATION_FAILED)?
                        .as_ptr(),
                )
                .window(window.window.get());
            let surface_fn = xcb_surface::Instance::new(entry, instance);
            surface_fn.create_xcb_surface(&surface_desc, allocation_callbacks)
        }

        #[cfg(target_os = "android")]
        (RawDisplayHandle::Android(_), RawWindowHandle::AndroidNdk(window)) => {
            let surface_desc =
                vk::AndroidSurfaceCreateInfoKHR::default().window(window.a_native_window.as_ptr());
            let surface_fn = android_surface::Instance::new(entry, instance);
            surface_fn.create_android_surface(&surface_desc, allocation_callbacks)
        }

        #[cfg(target_os = "macos")]
        (RawDisplayHandle::AppKit(_), RawWindowHandle::AppKit(window)) => {
            use raw_window_handle::{appkit, Layer};

            let layer = match appkit::metal_layer_from_handle(window) {
                Layer::Existing(layer) | Layer::Allocated(layer) => layer.cast(),
            };

            let surface_desc = vk::MetalSurfaceCreateInfoEXT::default().layer(&*layer);
            let surface_fn = metal_surface::Instance::new(entry, instance);
            surface_fn.create_metal_surface(&surface_desc, allocation_callbacks)
        }

        #[cfg(target_os = "ios")]
        (RawDisplayHandle::UiKit(_), RawWindowHandle::UiKit(window)) => {
            use raw_window_metal::{uikit, Layer};

            let layer = match uikit::metal_layer_from_handle(window) {
                Layer::Existing(layer) | Layer::Allocated(layer) => layer.cast(),
            };

            let surface_desc = vk::MetalSurfaceCreateInfoEXT::default().layer(&*layer);
            let surface_fn = metal_surface::Instance::new(entry, instance);
            surface_fn.create_metal_surface(&surface_desc, allocation_callbacks)
        }

        _ => Err(vk::Result::ERROR_EXTENSION_NOT_PRESENT),
    }
}

pub fn enumerate_required_extensions(
    display_handle: RawDisplayHandle,
) -> VkResult<&'static [*const c_char]> {
    let extensions = match display_handle {
        #[cfg(target_os = "windows")]
        RawDisplayHandle::Windows(_) => {
            const WINDOWS_EXTS: [*const c_char; 2] =
                [surface::NAME.as_ptr(), win32_surface::NAME.as_ptr()];
            &WINDOWS_EXTS
        }

        #[cfg(target_os = "linux")]
        RawDisplayHandle::Wayland(_) => {
            use ash::khr::wayland_surface;

            const WAYLAND_EXTS: [*const c_char; 2] =
                [surface::NAME.as_ptr(), wayland_surface::NAME.as_ptr()];
            &WAYLAND_EXTS
        }

        #[cfg(target_os = "linux")]
        RawDisplayHandle::Xlib(_) => {
            use ash::khr::xlib_surface;

            const XLIB_EXTS: [*const c_char; 2] =
                [surface::NAME.as_ptr(), xlib_surface::NAME.as_ptr()];
            &XLIB_EXTS
        }

        #[cfg(target_os = "linux")]
        RawDisplayHandle::Xcb(_) => {
            use ash::khr::xcb_surface;

            const XCB_EXTS: [*const c_char; 2] =
                [surface::NAME.as_ptr(), xcb_surface::NAME.as_ptr()];
            &XCB_EXTS
        }

        #[cfg(target_os = "android")]
        RawDisplayHandle::Android(_) => {
            const ANDROID_EXTS: [*const c_char; 2] =
                [surface::NAME.as_ptr(), android_surface::NAME.as_ptr()];
            &ANDROID_EXTS
        }

        #[cfg(any(target_os = "macos", target_os = "ios"))]
        RawDisplayHandle::AppKit(_) | RawDisplayHandle::UiKit(_) => {
            const METAL_EXTS: [*const c_char; 2] =
                [surface::NAME.as_ptr(), metal_surface::NAME.as_ptr()];
            &METAL_EXTS
        }

        _ => return Err(vk::Result::ERROR_EXTENSION_NOT_PRESENT),
    };

    Ok(extensions)
}
