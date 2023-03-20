use std::path::Path;

pub struct VentWindow {
    pub window: winit::window::Window,
    pub event_loop: winit::event_loop::EventLoop<()>,
}

impl VentWindow {
    pub fn new(builder: winit::window::WindowBuilder) -> Self {
        let event_loop = winit::event_loop::EventLoop::new();
        let window = builder.build(&event_loop).unwrap();

        Self { window, event_loop }
    }

    #[must_use]
    pub fn load_icon(path: &Path) -> winit::window::Icon {
        let (icon_rgba, icon_width, icon_height) = {
            let image = image::open(path)
                .expect("Failed to open icon path")
                .into_rgba8();
            let (width, height) = image.dimensions();
            let rgba = image.into_raw();
            (rgba, width, height)
        };
        winit::window::Icon::from_rgba(icon_rgba, icon_width, icon_height)
            .expect("Failed to open icon")
    }
}
