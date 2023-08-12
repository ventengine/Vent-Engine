use winit::event::VirtualKeyCode;

use super::Camera3D;

pub struct CameraController3D {
    speed: f32,
    sensitivity_x: f32,
    sensitivity_y: f32,
}

impl CameraController3D {
    #[inline]
    #[must_use]
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            speed,
            sensitivity_x: sensitivity,
            sensitivity_y: sensitivity,
        }
    }

    pub fn process_keyboard(
        &mut self,
        camera: &mut Camera3D,
        key: &VirtualKeyCode,
        delta_time: f32,
    ) -> bool {
        let sin_pitch = camera.rotation.x.sin();
        let cos_pitch = camera.rotation.x.cos();
        match key {
            VirtualKeyCode::W | VirtualKeyCode::Up => {
                camera.position.x += sin_pitch * self.speed * delta_time;
                camera.position.z += cos_pitch * self.speed * delta_time;
                true
            }
            VirtualKeyCode::S | VirtualKeyCode::Down => {
                camera.position.x -= sin_pitch * self.speed * delta_time;
                camera.position.z -= cos_pitch * self.speed * delta_time;
                true
            }
            VirtualKeyCode::A | VirtualKeyCode::Left => {
                camera.position.x -= cos_pitch * self.speed * delta_time;
                camera.position.z += sin_pitch * self.speed * delta_time;
                true
            }
            VirtualKeyCode::D | VirtualKeyCode::Right => {
                camera.position.x += cos_pitch * self.speed * delta_time;
                camera.position.z -= sin_pitch * self.speed * delta_time;
                true
            }
            VirtualKeyCode::Space => {
                camera.position.y += self.speed * delta_time;
                true
            }
            VirtualKeyCode::LShift => {
                camera.position.y -= self.speed * delta_time;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(
        &mut self,
        camera: &mut Camera3D,
        mouse_dx: f64,
        mouse_dy: f64,
        delta_time: f32,
    ) {
        let deltaposition = glam::vec2(mouse_dx as f32, mouse_dy as f32);

        let moveposition =
            deltaposition * glam::vec2(self.sensitivity_x, self.sensitivity_y) * delta_time;
        let mut rotation = camera.rotation;
        rotation.x += moveposition.x;
        rotation.y += moveposition.y;
    }
}
