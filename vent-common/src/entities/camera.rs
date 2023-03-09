use std::f32::consts;

pub trait Camera {}

#[allow(dead_code)]
pub struct BasicCamera {
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,

    pub rotation: glam::Vec2,
}

impl BasicCamera {
    fn build_view_projection_matrix(&self, aspect_ratio: f32) -> glam::Mat4 {
        // 1.
        let projection = glam::Mat4::perspective_rh(consts::FRAC_PI_4, aspect_ratio, self.znear, self.zfar);

        let view = glam::Mat4::look_at_rh(
            glam::Vec3::new(1.5f32, -5.0, 3.0),
            glam::Vec3::ZERO,
            glam::Vec3::Y,
        );

        projection * view
    }
}

pub struct Camera2D {
    pub basic_cam: BasicCamera,
    pub position: glam::Vec2,
}

impl Camera for Camera2D {}


pub struct Camera3D {
    pub basic_cam: BasicCamera,
    pub position: glam::Vec3,
}

impl Camera for Camera3D {}

