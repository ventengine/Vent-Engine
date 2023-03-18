use winit::event::{ElementState, VirtualKeyCode};

pub trait InputComponent {
    fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState);

    fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64);
}
