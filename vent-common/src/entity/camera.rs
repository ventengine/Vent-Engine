use std::f32::consts;

pub trait Camera {
    fn new() -> Self
    where
        Self: Sized;
}

pub struct BasicCamera {
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
    projection: glam::Mat4,
    view: glam::Mat4,

    pub rotation: glam::Vec2,
}

pub trait BasicCameraImpl: Camera {
    fn build_view_projection_matrix(&mut self, aspect_ratio: f32) -> glam::Mat4;
}

impl BasicCameraImpl for Camera3D {
    fn build_view_projection_matrix(&mut self, aspect_ratio: f32) -> glam::Mat4 {
        self.basic_cam.build_view_projection_matrix(aspect_ratio)
    }
}

impl BasicCameraImpl for Camera2D {
    fn build_view_projection_matrix(&mut self, aspect_ratio: f32) -> glam::Mat4 {
        self.basic_cam.build_view_projection_matrix(aspect_ratio)
    }
}

impl BasicCamera {
    pub fn build_view_projection_matrix(&mut self, aspect_ratio: f32) -> glam::Mat4 {
        self.aspect = aspect_ratio;

        self.projection =
            glam::Mat4::perspective_rh(self.fovy.to_radians(), aspect_ratio, self.znear, self.zfar);

        let view = glam::Mat4::look_at_rh(
            glam::Vec3::new(1.5f32, -5.0, 3.0),
            glam::Vec3::ZERO,
            glam::Vec3::Y,
        );

        self.view = view;

        self.projection * self.view
    }
}

impl Default for BasicCamera {
    fn default() -> Self {
        let aspect = 0.0;
        let _fovy = 60.0;
        let znear = 0.1;
        let zfar = 100.0;

        let projection = glam::Mat4::perspective_rh(consts::FRAC_PI_4, aspect, znear, zfar);

        let view = glam::Mat4::look_at_rh(
            glam::vec3(1.5, -5.0, 3.0),
            glam::vec3(0.0, 0.0, 0.0),
            glam::Vec3::Y,
        );

        Self {
            aspect: 0.0,
            fovy: 60.0,
            znear: 0.1,
            zfar: 100.0,
            projection,
            view,
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
            position: glam::Vec3::ZERO,
        }
    }
}
