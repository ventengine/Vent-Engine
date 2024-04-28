pub struct VentWindow {
    pub window: winit::window::Window,
    pub event_loop: winit::event_loop::EventLoop<()>,
}




impl VentWindow {
    #[must_use]
    pub fn new(attributes: winit::window::WindowAttributes) -> Self {
        let event_loop = winit::event_loop::EventLoop::new().unwrap();
        let window = event_loop.create_window(attributes).expect("Failed to Create Window");

        Self { window, event_loop }
    }

    #[must_use]
    pub fn load_icon(path: &str) -> winit::window::Icon {
        let (icon_rgba, icon_width, icon_height) = {
            let image = image::open(path)
                .expect("Failed to open icon path")
                .into_rgba8();
            let (width, height) = image.dimensions();
            let rgba = image.into_raw();
            (rgba, width, height)
        };
        winit::window::Icon::from_rgba(icon_rgba, icon_width, icon_height)
            .expect("Failed to load icon")
    }
}
