use glam::Vec3;
use std::f32::consts;

pub trait Camera {
    fn new() -> Self
    where
        Self: Sized;
    fn build_view_projection_matrix(&mut self, aspect_ratio: f32) -> glam::Mat4;
}

pub struct BasicCamera {
    fovy: f32,
    znear: f32,
    zfar: f32,
    pub rotation: glam::Vec2,
}

impl Default for BasicCamera {
    fn default() -> Self {
        Self {
            fovy: 60.0,
            znear: 0.1,
            zfar: 1000.0,
            rotation: glam::Vec2::ZERO,
        }
    }
}

pub struct Camera2D {
    pub basic_cam: BasicCamera,
    pub position: glam::Vec2,
}

impl Camera for Camera2D {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            basic_cam: BasicCamera::default(),
            position: glam::Vec2::ZERO,
        }
    }

    fn build_view_projection_matrix(&mut self, aspect_ratio: f32) -> glam::Mat4 {
        todo!()
    }
}

pub struct Camera3D {
    pub basic_cam: BasicCamera,
    pub position: glam::Vec3,
}

impl Camera for Camera3D {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            basic_cam: BasicCamera::default(),
            position: glam::vec3(1.5, 1.0, -6.0),
        }
    }

    fn build_view_projection_matrix(&mut self, aspect_ratio: f32) -> glam::Mat4 {
        println!("{}", self.position);
        let projection = glam::Mat4::perspective_lh(
            self.basic_cam.fovy.to_radians(),
            aspect_ratio,
            self.basic_cam.znear,
            self.basic_cam.zfar,
        );

        let view = glam::Mat4::look_at_lh(
            self.position,
            self.position + self.direction_from_rotation(),
            glam::Vec3::Y,
        );

        projection * view
    }
}

impl Camera3D {
    fn direction_from_rotation(&self) -> glam::Vec3 {
        let rot = self.basic_cam.rotation;
        let cos_y = self.basic_cam.rotation.y.cos();

        Vec3::new(rot.x.sin() * cos_y, rot.y.sin(), rot.x.cos() * cos_y)
    }
}
