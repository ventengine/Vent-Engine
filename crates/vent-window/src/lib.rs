use keyboard::{Key, KeyState};
use rwh_06::{DisplayHandle, HasDisplayHandle, HasWindowHandle};
pub mod keyboard;
pub mod mouse;
pub mod platform;

#[derive(PartialEq, Clone)]
pub enum WindowEvent {
    Close,
    Key {
        key: keyboard::Key,
        state: keyboard::KeyState,
    },
    Mouse {
        key: mouse::Key,
        state: mouse::ButtonState,
    },
    Draw,
}

enum WindowError {}

#[derive(PartialEq)]
pub enum WindowMode {
    Default,
    FullScreen,
    Maximized,
    Minimized,
}

pub struct WindowAttribs {
    title: String,
    width: u32,
    height: u32,
    mode: WindowMode,
    min_size: Option<(u32, u32)>,
    max_size: Option<(u32, u32)>,
}

impl WindowAttribs {
    pub fn with_title(mut self, title: String) -> Self {
        self.title = title;
        self
    }
    pub fn set_mode(mut self, mode: WindowMode) -> Self {
        self.mode = mode;
        self
    }
}

impl Default for WindowAttribs {
    fn default() -> Self {
        Self {
            title: "Vent Engine".to_string(),
            width: 800,
            height: 600,
            mode: WindowMode::Default,
            max_size: None,
            min_size: None,
        }
    }
}

pub struct EventLoop {
    windows: Vec<Window>,
}

impl Default for EventLoop {
    fn default() -> Self {
        Self::new()
    }
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

/**
 * Cross Platform window Wrapper
 */
pub struct Window {
    window: platform::PlatformWindow,
}

impl Window {
    pub fn new(inital_attribs: WindowAttribs) -> Self {
        Self {
            window: platform::PlatformWindow::create_window(&inital_attribs),
        }
    }

    pub fn poll<F>(self, event_handler: F)
    where
        F: FnMut(WindowEvent),
    {
        self.window.poll(event_handler);
    }

    pub fn close(&mut self) {
        self.window.close()
    }

    pub fn width(&self) -> u32 {
        self.window.width()
    }

    pub fn height(&self) -> u32 {
        self.window.height()
    }

    pub fn size(&self) -> (u32, u32) {
        (self.window.width(), self.window.height())
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
