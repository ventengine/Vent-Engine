use crate::render::{UBO2D, UBO3D};
use glam::{Mat4, Vec3};

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

pub struct BasicCamera {
    fovy: f32,
    znear: f32,
    zfar: f32,
    pub rotation: glam::Vec2,
}

impl Default for BasicCamera {
    #[inline]
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
    #[inline]
    #[must_use]
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            basic_cam: BasicCamera::default(),
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
    pub basic_cam: BasicCamera,
    pub position: glam::Vec3,
}

impl Camera for Camera3D {
    #[inline]
    #[must_use]
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            basic_cam: BasicCamera::default(),
            position: Vec3::ZERO,
        }
    }

    #[must_use]
    fn build_view_matrix_2d(&mut self, _aspect_ratio: f32) -> UBO2D {
        todo!()
    }

    #[must_use]
    fn build_view_matrix_3d(&mut self, aspect_ratio: f32) -> UBO3D {
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
        let rot = self.basic_cam.rotation;
        let cos_y = self.basic_cam.rotation.y.cos();

        glam::vec3(rot.x.sin() * cos_y, rot.y.sin(), rot.x.cos() * cos_y)
    }
}
