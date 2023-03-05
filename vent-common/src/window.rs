use std::path::Path;
use winit::event_loop::EventLoop;
use winit::window::{Icon, Window, WindowBuilder};

pub struct VentWindow {
    pub window: Window,
    pub event_loop: EventLoop<()>,
}

impl VentWindow {
    pub fn new(builder: WindowBuilder) -> Self {
        let event_loop = EventLoop::new();
        let window = builder.build(&event_loop).unwrap();

        Self { window, event_loop }
    }

    pub fn load_icon(path: &Path) -> Icon {
        let (icon_rgba, icon_width, icon_height) = {
            let image = image::open(path)
                .expect("Failed to open icon path")
                .into_rgba8();
            let (width, height) = image.dimensions();
            let rgba = image.into_raw();
            (rgba, width, height)
        };
        Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
    }
}
