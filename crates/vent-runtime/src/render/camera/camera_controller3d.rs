use vent_math::vec::{vec2::Vec2, vec3::Vec3};
use vent_window::keyboard::Key;

use crate::util::input_handler::InputHandler;

use super::Camera3D;

pub struct CameraController3D {
    speed: f32,
    sensitivity_x: f32,
    sensitivity_y: f32,

    old_x: f64,
    old_y: f64,
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
            old_x: 0.0,
            old_y: 0.0,
        }
    }

    pub fn process_keyboard(
        &self,
        camera: &mut Camera3D,
        input_handler: &InputHandler,
        delta_time: f32,
    ) {
        let (sin_pitch, cos_pitch) = camera.rotation.x.sin_cos();
        let (sin_yaw, cos_yaw) = camera.rotation.y.sin_cos();
        let mut moved = false;

        if input_handler.is_pressed(Key::W) | input_handler.is_pressed(Key::Uparrow) {
            camera.position += Vec3::new(cos_pitch * cos_yaw, sin_pitch, -cos_pitch * sin_yaw)
                * self.speed
                * delta_time;
            moved = true;
        }
        if input_handler.is_pressed(Key::S) | input_handler.is_pressed(Key::Downarrow) {
            camera.position -= Vec3::new(cos_pitch * cos_yaw, sin_pitch, -cos_pitch * sin_yaw)
                * self.speed
                * delta_time;
            moved = true;
        }
        if input_handler.is_pressed(Key::A) | input_handler.is_pressed(Key::Leftarrow) {
            camera.position -= Vec3::new(sin_yaw, 0.0, cos_yaw) * self.speed * delta_time;
            moved = true;
        }
        if input_handler.is_pressed(Key::D) | input_handler.is_pressed(Key::Rightarrow) {
            camera.position += Vec3::new(sin_yaw, 0.0, cos_yaw) * self.speed * delta_time;
            moved = true;
        }
        if input_handler.is_pressed(Key::Space) {
            camera.position.y += self.speed * delta_time;
            moved = true;
        }
        if input_handler.is_pressed(Key::ShiftL) {
            camera.position.y -= self.speed * delta_time;
            moved = true;
        }
        if moved {
            camera.recreate_view();
        }
    }

    pub fn process_mouse_input(
        &mut self,
        button: &vent_window::mouse::Button,
        state: &vent_window::mouse::ButtonState,
    ) {
        if button == &vent_window::mouse::Button::LEFT {
            self.mouse_left_down = state == &vent_window::mouse::ButtonState::Pressed;
        }
    }

    pub fn process_mouse_movement(
        &mut self,
        camera: &mut Camera3D,
        mouse_x: f64,
        mouse_y: f64,
        delta_time: f32,
    ) {
        if self.mouse_left_down {
            let deltaposition =
                Vec2::new((mouse_x - self.old_x) as f32, (mouse_y - self.old_y) as f32);
            self.old_x = mouse_x;
            self.old_y = mouse_y;

            let moveposition =
                deltaposition * Vec2::new(self.sensitivity_x, self.sensitivity_y) * delta_time;
            camera.rotation.x += moveposition.x.to_radians();
            camera.rotation.y += moveposition.y.to_radians();
            camera.recreate_direction();
        }
    }
}
