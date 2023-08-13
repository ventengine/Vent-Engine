use glam::{Mat4, Vec3};

use super::{d2::UBO2D, d3::UBO3D};

pub mod camera_controller3d;

pub trait Camera {
    fn new() -> Self
    where
        Self: Sized;
    // ugly i know :c
    #[must_use]
    fn build_view_matrix_2d(&mut self, aspect_ratio: f32) -> UBO2D;

    #[must_use]
    fn build_view_matrix_3d(&mut self, aspect_ratio: f32) -> UBO3D;
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

    #[must_use]
    fn build_view_matrix_2d(&mut self, _aspect_ratio: f32) -> UBO2D {
        todo!()
    }

    #[must_use]
    fn build_view_matrix_3d(&mut self, _aspect_ratio: f32) -> UBO3D {
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

    #[must_use]
    fn build_view_matrix_2d(&mut self, _aspect_ratio: f32) -> UBO2D {
        todo!()
    }

    #[must_use]
    fn build_view_matrix_3d(&mut self, aspect_ratio: f32) -> UBO3D {
        let projection =
            glam::Mat4::perspective_lh(self.fovy.to_radians(), aspect_ratio, self.znear, self.zfar);

        let view = glam::Mat4::look_to_lh(
            self.position,
            self.position + self.direction_from_rotation(),
            glam::Vec3::Y,
        );
        UBO3D {
            projection: projection.to_cols_array_2d(),
            view: view.to_cols_array_2d(),
            transformation: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }
}

impl Camera3D {
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
