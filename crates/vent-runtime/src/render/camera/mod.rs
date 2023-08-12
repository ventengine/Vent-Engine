use glam::{Mat4, Vec3};
use vent_common::render::{UBO2D, UBO3D};

pub mod camera_controller3d;

pub trait Camera {
    fn new() -> Self
    where
        Self: Sized;
}

pub struct Camera2D {
    pub position: glam::Vec2,
    pub rotation: glam::Quat,
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
            rotation: glam::Quat::IDENTITY,
        }
    }
}

impl Camera2D {
    #[must_use]
    pub fn build_view_matrix_2d(&mut self, _aspect_ratio: f32) -> UBO2D {
        todo!()
    }
}

pub struct Camera3D {
    fovy: f32,
    znear: f32,
    zfar: f32,
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
        }
    }
}

impl Camera3D {
    #[must_use]
    pub fn build_view_matrix_3d(&mut self, aspect_ratio: f32) -> UBO3D {
        let projection =
            glam::Mat4::perspective_lh(self.fovy.to_radians(), aspect_ratio, self.znear, self.zfar);

        let view =
            glam::Mat4::look_at_lh(self.position, self.direction_from_rotation(), glam::Vec3::Y);
        UBO3D {
            projection: projection.to_cols_array_2d(),
            view: view.to_cols_array_2d(),
            transformation: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    #[inline]
    #[must_use]
    fn direction_from_rotation(&self) -> glam::Vec3 {
        let (sin_pitch, cos_pitch) = self.rotation.y.sin_cos();
        let (sin_yaw, cos_yaw) = self.rotation.x.sin_cos();

        glam::vec3(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize()
    }
}
