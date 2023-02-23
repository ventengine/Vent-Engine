use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

pub struct VentWindow {
    pub window : Window,
    pub event_loop: EventLoop<()>,
}

impl VentWindow {

    pub fn new(title: &String) -> Self {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(title)
            .build(&event_loop).unwrap();

        Self {
            window,
            event_loop
        }
    }


}

