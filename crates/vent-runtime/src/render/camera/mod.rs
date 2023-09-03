use downcast_rs::{impl_downcast, Downcast};
use glam::Vec3;

use super::{d3::UBO3D, Dimension};

pub mod camera_controller3d;

pub trait Camera: Downcast {
    fn new() -> Self
    where
        Self: Sized;
}
impl_downcast!(Camera);

pub fn from_dimension(dimension: Dimension) -> Box<dyn Camera> {
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

pub struct Camera3D {
    fovy: f32,
    znear: f32,
    zfar: f32,
    ubo: UBO3D,

    pub position: glam::Vec3,
    pub rotation: glam::Quat,
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
        let view =
            glam::Mat4::look_to_lh(self.position, self.direction_from_rotation(), glam::Vec3::Y);
        self.ubo.view_position = self.position.to_array();
        self.ubo.view = view.to_cols_array_2d();
    }

    pub fn ubo(&self) -> UBO3D {
        self.ubo
    }

    #[inline]
    #[must_use]
    fn direction_from_rotation(&self) -> glam::Vec3 {
        let (sin_pitch, cos_pitch) = self.rotation.y.sin_cos();
        let (sin_yaw, cos_yaw) = self.rotation.x.sin_cos();

        glam::vec3(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw)
    }
}
