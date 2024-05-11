use std::ptr::NonNull;

use platform::wayland::WaylandWindow;
use rwh_06::{
    DisplayHandle, HasDisplayHandle, HasRawDisplayHandle, HasWindowHandle, RawDisplayHandle,
    WaylandDisplayHandle,
};
use wayland_client::Proxy;

pub mod platform;

#[derive(PartialEq, Clone)]
pub enum WindowEvent {
    Draw,
}

enum WindowError {}

pub struct WindowAttribs {
    title: String,
    width: u32,
    height: u32,
}

impl WindowAttribs {
    pub fn with_title(mut self, title: String) -> Self {
        self.title = title;
        self
    }
}

impl Default for WindowAttribs {
    fn default() -> Self {
        Self {
            title: "Vent Engine".to_string(),
            width: 800,
            height: 600,
        }
    }
}

pub struct EventLoop {
    windows: Vec<Window>,
}

impl EventLoop {
    pub fn new() -> Self {
        Self { windows: vec![] }
    }
    pub fn add_window(&mut self, window: Window) {
        self.windows.push(window);
    }
    pub fn poll<F>(self, mut event_handler: F)
    where
        F: FnMut(WindowEvent),
    {
        for window in self.windows {
            window.poll(&mut event_handler)
        }
    }
}

pub struct Window {
    attribs: WindowAttribs,
    window: WaylandWindow,
}

impl Window {
    pub fn new(attribs: WindowAttribs) -> Self {
        Self {
            window: WaylandWindow::create_window(&attribs),
            attribs,
        }
    }

    pub fn poll<F>(self, event_handler: F)
    where
        F: FnMut(WindowEvent),
    {
        self.window.poll(event_handler);
    }

    pub fn width(&self) -> u32 {
        self.attribs.width
    }

    pub fn height(&self) -> u32 {
        self.attribs.height
    }

    pub fn size(&self) -> (u32, u32) {
        (self.attribs.width, self.attribs.height)
    }
}

impl HasDisplayHandle for Window {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, rwh_06::HandleError> {
        let raw = self.window.raw_display_handle();
        Ok(unsafe { rwh_06::DisplayHandle::borrow_raw(raw) })
    }
}

impl HasWindowHandle for Window {
    fn window_handle(&self) -> Result<rwh_06::WindowHandle<'_>, rwh_06::HandleError> {
        let raw = self.window.raw_window_handle();
        Ok(unsafe { rwh_06::WindowHandle::borrow_raw(raw) })
    }
}
