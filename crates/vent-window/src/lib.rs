use std::num::NonZeroU32;

use raw_window_handle::{DisplayHandle, HasDisplayHandle, HasWindowHandle};
use serde::{Deserialize, Serialize};
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
        button: mouse::Button,
        state: mouse::ButtonState,
    },
    Resize {
        new_width: u32,
        new_height: u32,
    },
    Draw,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub enum WindowMode {
    Default,
    FullScreen,
    Maximized,
    Minimized,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WindowAttribs {
    title: String,
    app_id: String,
    width: NonZeroU32,
    height: NonZeroU32,
    mode: WindowMode,
    min_size: Option<(u32, u32)>,
    max_size: Option<(u32, u32)>,
    resizable: bool,
    closeable: bool,
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
            app_id: "com.ventengine.VentEngine".to_string(),
            width: unsafe { NonZeroU32::new_unchecked(800) },
            height: unsafe { NonZeroU32::new_unchecked(600) },
            mode: WindowMode::Default,
            max_size: None,
            min_size: None,
            resizable: true,
            closeable: true,
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
            window: platform::PlatformWindow::create_window(inital_attribs),
        }
    }

    pub fn poll<F>(mut self, event_handler: F)
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

    pub fn set_cursor_visible() {}
}

impl HasDisplayHandle for Window {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, raw_window_handle::HandleError> {
        let raw = self.window.raw_display_handle();
        Ok(unsafe { raw_window_handle::DisplayHandle::borrow_raw(raw) })
    }
}

impl HasWindowHandle for Window {
    fn window_handle(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        let raw = self.window.raw_window_handle();
        Ok(unsafe { raw_window_handle::WindowHandle::borrow_raw(raw) })
    }
}
