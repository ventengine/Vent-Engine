use ash::vk;
use downcast_rs::{impl_downcast, Downcast};
use vent_math::{
    scalar::{mat4::Mat4, quat::Quat},
    vec::{vec2::Vec2, vec3::Vec3},
};
use vent_rendering::{any_as_u8_slice, instance::VulkanInstance};

use super::{d3::Camera3DData, Dimension};

pub mod camera_controller3d;

pub trait Camera: Downcast {
    fn new(aspect_ratio: f32) -> Self
    where
        Self: Sized;

    fn recreate_projection(&mut self, aspect_ratio: f32);
}
impl_downcast!(Camera);

pub fn from_dimension(aspect_ratio: f32, dimension: &Dimension) -> Box<dyn Camera> {
    match dimension {
        Dimension::D2 => Box::new(Camera2D::new(aspect_ratio)),
        Dimension::D3 => Box::new(Camera3D::new(aspect_ratio)),
    }
}

#[allow(dead_code)]
pub struct Camera2D {
    position: Vec2,
}

impl Camera for Camera2D {
    #[inline]
    #[must_use]
    fn new(_aspect_ratio: f32) -> Self
    where
        Self: Sized,
    {
        Self {
            position: Vec2::ZERO,
        }
    }

    fn recreate_projection(&mut self, _aspect_ratio: f32) {
        todo!()
    }
}

pub struct Camera3D {
    fovy: f32,
    znear: f32,
    zfar: f32,
    pub ubo: Camera3DData,
    pub projection: Mat4,
    pub view: Mat4,
    pub transformation: Mat4,

    pub position: Vec3,
    pub direction: Vec3,
    pub rotation: Quat,
}

impl Camera for Camera3D {
    #[inline]
    #[must_use]
    fn new(aspect_ratio: f32) -> Self
    where
        Self: Sized,
    {
        let mut cam = Self {
            fovy: 60.0,
            znear: 0.1,
            zfar: 10000.0,
            rotation: Quat::IDENTITY,
            position: Vec3::ZERO,
            ubo: Default::default(),
            projection: Mat4::IDENTITY,
            direction: Vec3::ZERO,
            view: Mat4::IDENTITY,
            transformation: Mat4::IDENTITY,
        };
        // we should configure
        cam.recreate_projection(aspect_ratio);
        cam.recreate_direction();
        cam.calc_matrix();

        cam
    }

    // Only recreate when:
    // FOV is changed
    // Aspect ratio is changed
    // znear is changed
    // zfar is changed
    fn recreate_projection(&mut self, aspect_ratio: f32) {
        self.projection =
            Mat4::perspective_rh(self.fovy.to_radians(), aspect_ratio, self.znear, self.zfar);
        // Flip the cameras prospective upside down
        self.projection.y_axis.y *= -1.0;
    }
}

impl Camera3D {
    pub fn update_set() {}

    /// Call when position changed
    pub fn recreate_view(&mut self) {
        let view = Mat4::look_at_rh(self.position, self.position + self.direction, Vec3::Y);
        self.ubo.view_position = self.position;
        self.view = view;
    }

    pub fn calc_matrix(&mut self) {
        self.ubo.proj_view_trans = self.projection * self.view * self.transformation;
    }

    pub fn write(
        &self,
        instance: &VulkanInstance,
        layout: vk::PipelineLayout,
        command_buffer: vk::CommandBuffer,
    ) {
        unsafe {
            instance.device.cmd_push_constants(
                command_buffer,
                layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                any_as_u8_slice(&self.ubo),
            )
        }
    }

    #[inline]
    /// Call when rotation changed
    pub fn recreate_direction(&mut self) {
        let (sin_yaw, cos_yaw) = self.rotation.x.sin_cos();
        let (sin_pitch, cos_pitch) = self.rotation.y.sin_cos();

        self.direction = Vec3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize();
        self.recreate_view()
    }
}
