pub trait InputComponent {
    // fn process_keyboard(&mut self, key: keyboard::Key, state: ElementState);

    fn process_mouse_motion(&mut self, mouse_dx: f64, mouse_dy: f64);
}
