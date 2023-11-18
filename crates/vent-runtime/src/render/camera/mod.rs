use std::mem::size_of;

use ash::vk;
use downcast_rs::{impl_downcast, Downcast};
use glam::{Quat, Vec3};
use vent_rendering::{buffer::VulkanBuffer, instance::VulkanInstance};

use super::{d3::UBO3D, Dimension};

pub mod camera_controller3d;

pub trait Camera: Downcast {
    fn new(instance: &VulkanInstance) -> Self
    where
        Self: Sized;
}
impl_downcast!(Camera);

pub fn from_dimension(instance: &VulkanInstance, dimension: &Dimension) -> Box<dyn Camera> {
    match dimension {
        Dimension::D2 => Box::new(Camera2D::new(instance)),
        Dimension::D3 => Box::new(Camera3D::new(instance)),
    }
}

#[allow(dead_code)]
pub struct Camera2D {
    position: glam::Vec2,
}

impl Camera for Camera2D {
    #[inline]
    #[must_use]
    fn new(_instance: &VulkanInstance) -> Self
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
    ubo_buffers: Vec<VulkanBuffer>,

    position: glam::Vec3,
    rotation: glam::Quat,
}

impl Camera for Camera3D {
    #[inline]
    #[must_use]
    fn new(instance: &VulkanInstance) -> Self
    where
        Self: Sized,
    {
        let mut ubo_buffers = vec![];
        for _ in 0..instance.swapchain_images.len() {
            ubo_buffers.push(VulkanBuffer::new(
                &instance.device,
                &instance.memory_allocator,
                size_of::<UBO3D>() as vk::DeviceSize,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
            ))
        }

        Self {
            fovy: 60.0,
            znear: 0.1,
            zfar: 10000.0,
            rotation: glam::Quat::IDENTITY,
            position: Vec3::ZERO,
            ubo: Default::default(),
            ubo_buffers,
        }
    }
}

impl Camera3D {
    fn recreate_view(&mut self) {
        let view =
            glam::Mat4::look_to_lh(self.position, self.direction_from_rotation(), glam::Vec3::Y);
        self.ubo.view_position = self.position.to_array();
        self.ubo.view = view.to_cols_array_2d();
    }

    pub fn recreate_projection(&mut self, aspect_ratio: f32) {
        let projection =
            glam::Mat4::perspective_lh(self.fovy.to_radians(), aspect_ratio, self.znear, self.zfar);
        self.ubo.projection = projection.to_cols_array_2d();
    }

    pub fn write(&self, instance: &VulkanInstance, index: u32) {
        unsafe {
            self.ubo_buffers[index as usize].upload_type(
                &instance.device,
                &self.ubo,
                size_of::<UBO3D>() as vk::DeviceSize,
            )
        }
    }

    pub fn set_x(&mut self, x: f32) {
        self.position.x = x;
        self.recreate_view();
    }

    pub fn set_y(&mut self, y: f32) {
        self.position.y = y;
        self.recreate_view();
    }

    pub fn set_z(&mut self, z: f32) {
        self.position.z = z;
        self.recreate_view();
    }

    pub fn add_x(&mut self, x: f32) {
        self.position.x += x;
        self.recreate_view();
    }

    pub fn add_y(&mut self, y: f32) {
        self.position.y += y;
        self.recreate_view();
    }

    pub fn add_z(&mut self, z: f32) {
        self.position.z += z;
        self.recreate_view();
    }

    pub fn minus_x(&mut self, x: f32) {
        self.position.x -= x;
        self.recreate_view();
    }

    pub fn minus_y(&mut self, y: f32) {
        self.position.y -= y;
        self.recreate_view();
    }

    pub fn minus_z(&mut self, z: f32) {
        self.position.z -= z;
        self.recreate_view();
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn set_yaw(&mut self, yaw: f32) {
        self.rotation.x = yaw;
        self.recreate_view();
    }

    pub fn set_pitch(&mut self, pitch: f32) {
        self.rotation.y = pitch;
        self.recreate_view();
    }

    pub fn add_yaw(&mut self, yaw: f32) {
        self.rotation.x += yaw;
        self.recreate_view();
    }

    pub fn add_pitch(&mut self, pitch: f32) {
        self.rotation.y += pitch;
        self.recreate_view();
    }

    pub fn rotation(&self) -> Quat {
        self.rotation
    }

    pub fn ubo(&self) -> UBO3D {
        self.ubo
    }

    #[inline]
    #[must_use]
    fn direction_from_rotation(&self) -> Vec3 {
        let (sin_pitch, cos_pitch) = self.rotation.y.sin_cos();
        let (sin_yaw, cos_yaw) = self.rotation.x.sin_cos();

        glam::vec3(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw)
    }
}
