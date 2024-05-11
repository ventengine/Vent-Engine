use winit::{
    event::{ElementState, KeyEvent},
    keyboard::{Key, NamedKey},
};

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
        event: &KeyEvent,
        delta_time: f32,
    ) -> bool {
        log::info!("{}", camera.position);
        if event.state == ElementState::Pressed {
            let (sin_pitch, cos_pitch) = camera.rotation.x.sin_cos();
            match event.logical_key.as_ref() {
                // Arrow keys works but WASD not :C
                Key::Character("W") | Key::Named(NamedKey::ArrowUp) => {
                    camera.position.x += sin_pitch * self.speed * delta_time;
                    camera.position.z += cos_pitch * self.speed * delta_time;
                    return true;
                }
                Key::Character("S") | Key::Named(NamedKey::ArrowDown) => {
                    camera.position.x -= sin_pitch * self.speed * delta_time;
                    camera.position.z -= cos_pitch * self.speed * delta_time;
                    return true;
                }
                Key::Character("A") | Key::Named(NamedKey::ArrowLeft) => {
                    camera.position.x -= cos_pitch * self.speed * delta_time;
                    camera.position.x += sin_pitch * self.speed * delta_time;
                    return true;
                }
                Key::Character("D") | Key::Named(NamedKey::ArrowRight) => {
                    camera.position.x += cos_pitch * self.speed * delta_time;
                    camera.position.z -= sin_pitch * self.speed * delta_time;
                    return true;
                }
                Key::Named(NamedKey::Space) => {
                    camera.position.y += self.speed * delta_time;
                    return true;
                }
                Key::Named(NamedKey::Shift) => {
                    camera.position.y -= self.speed * delta_time;
                    return true;
                }
                _ => return false,
            }
        }
        false
    }

    // pub fn process_mouse_input(
    //     &mut self,
    //     window: &winit::window::Window,
    //     button: &winit::event::MouseButton,
    //     state: &winit::event::ElementState,
    // ) {
    //     if button == &winit::event::MouseButton::Left {
    //         self.mouse_left_down = state == &winit::event::ElementState::Pressed;
    //         window.set_cursor_visible(!self.mouse_left_down);
    //     }
    // }

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
            camera.rotation.x += moveposition.x;
            camera.rotation.y += moveposition.y;
        }
    }
}
