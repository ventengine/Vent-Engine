use std::any::Any;

use glam::{Mat4, Vec3};

use super::{d2::UBO2D, d3::UBO3D, Dimension};

pub mod camera_controller3d;

pub trait Camera {
    fn new() -> Self
    where
        Self: Sized;
}

pub fn from_dimension(dimension: Dimension) -> Box<dyn Any> {
    match dimension {
        Dimension::D2 => Box::new(Camera2D::new()),
        Dimension::D3 => Box::new(Camera3D::new()),
    }
}


pub struct Camera2D {
    pub position: glam::Vec2,
}

impl Camera for Camera2D {
    #[inline]
    #[must_use]
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            position: glam::Vec2::ZERO,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Camera3D {
    fovy: f32,
    znear: f32,
    zfar: f32,

    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub ubo: UBO3D,
}

impl Camera for Camera3D {
    #[inline]
    #[must_use]
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            fovy: 60.0,
            znear: 0.1,
            zfar: 10000.0,
            rotation: glam::Quat::IDENTITY,
            position: Vec3::ZERO,
            ubo: Default::default(),
        }
    }
}

impl Camera3D {
    pub fn recreate_projection(&mut self, aspect_ratio: f32) {
        let projection =
            glam::Mat4::perspective_lh(self.fovy.to_radians(), aspect_ratio, self.znear, self.zfar);
        self.ubo.projection = projection.to_cols_array_2d();
    }

    pub fn recreate_view(&mut self) {
        let view = glam::Mat4::look_to_lh(
            self.position,
            self.position + self.direction_from_rotation(),
            glam::Vec3::Y,
        );
        self.ubo.view_position = self.position.to_array();
        self.ubo.view = view.to_cols_array_2d();
    }

    #[inline]
    #[must_use]
    fn direction_from_rotation(&self) -> glam::Vec3 {
        let cos_y = self.rotation.y.cos();

        glam::vec3(
            self.rotation.x.sin() * cos_y,
            self.rotation.y.sin(),
            self.rotation.x.cos() * cos_y,
        )
    }
}
