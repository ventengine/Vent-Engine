use crate::entity::camera::Camera3D;
use glam::Vec3;
use std::time::Duration;
use winit::event::{ElementState, VirtualKeyCode};

#[derive(Debug)]
pub struct CameraController3D {
    speed: f32,
    sensitivity_x: f32,
    sensitivity_y: f32,
}

impl CameraController3D {
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
        let sin_pitch = camera.basic_cam.rotation.x.sin();
        let cos_pitch = camera.basic_cam.rotation.x.cos();
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
                camera.position.x += cos_pitch * self.speed * delta_time;
                camera.position.z -= sin_pitch * self.speed * delta_time;
                true
            }
            VirtualKeyCode::D | VirtualKeyCode::Right => {
                camera.position.x -= cos_pitch * self.speed * delta_time;
                camera.position.z += sin_pitch * self.speed * delta_time;
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

    pub fn process_mouse(&mut self, camera: &mut Camera3D, mouse_dx: f64, mouse_dy: f64) {
        let deltaposition = glam::vec2(mouse_dx as f32, mouse_dy as f32);

        let moveposition =
            deltaposition * glam::vec2(self.sensitivity_x, self.sensitivity_y) * 0.0073;
        let mut rotation = camera.basic_cam.rotation;

        rotation.x -= moveposition.x;
        rotation.y -= moveposition.y;
    }
}
