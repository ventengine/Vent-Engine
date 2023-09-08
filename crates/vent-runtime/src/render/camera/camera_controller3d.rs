use winit::event::VirtualKeyCode;

use super::Camera3D;

pub struct CameraController3D {
    speed: f32,
    sensitivity_x: f32,
    sensitivity_y: f32,

    mouse_left_down: bool,
}

impl CameraController3D {
    #[inline]
    #[must_use]
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            speed,
            sensitivity_x: sensitivity,
            sensitivity_y: sensitivity,
            mouse_left_down: false,
        }
    }

    pub fn process_keyboard(
        &self,
        camera: &mut Camera3D,
        key: &VirtualKeyCode,
        delta_time: f32,
    ) -> bool {
        let (sin_pitch, cos_pitch) = camera.rotation.x.sin_cos();
        match key {
            VirtualKeyCode::W | VirtualKeyCode::Up => {
                camera.add_x(sin_pitch * self.speed * delta_time);
                camera.add_z(cos_pitch * self.speed * delta_time);
                true
            }
            VirtualKeyCode::S | VirtualKeyCode::Down => {
                camera.minus_x(sin_pitch * self.speed * delta_time);
                camera.minus_z(cos_pitch * self.speed * delta_time);
                true
            }
            VirtualKeyCode::A | VirtualKeyCode::Left => {
                camera.minus_x(cos_pitch * self.speed * delta_time);
                camera.add_z(sin_pitch * self.speed * delta_time);
                true
            }
            VirtualKeyCode::D | VirtualKeyCode::Right => {
                camera.add_x(cos_pitch * self.speed * delta_time);
                camera.minus_z(sin_pitch * self.speed * delta_time);
                true
            }
            VirtualKeyCode::Space => {
                camera.add_y(self.speed * delta_time);
                true
            }
            VirtualKeyCode::LShift => {
                camera.minus_y(self.speed * delta_time);
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse_input(
        &mut self,
        window: &winit::window::Window,
        button: &winit::event::MouseButton,
        state: &winit::event::ElementState,
    ) {
        if button == &winit::event::MouseButton::Left {
            self.mouse_left_down = state == &winit::event::ElementState::Pressed;
            window.set_cursor_visible(!self.mouse_left_down);
        }
    }

    pub fn process_mouse_movement(
        &self,
        camera: &mut Camera3D,
        mouse_dx: f64,
        mouse_dy: f64,
        delta_time: f32,
    ) {
        if self.mouse_left_down {
            let deltaposition = glam::vec2(mouse_dx as f32, mouse_dy as f32);

            let moveposition =
                deltaposition * glam::vec2(self.sensitivity_x, self.sensitivity_y) * delta_time;
            camera.add_yaw(moveposition.x);
            camera.add_pitch(moveposition.y);
        }
    }
}
