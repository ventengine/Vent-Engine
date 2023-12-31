use std::mem::size_of;

use ash::vk;
use downcast_rs::{impl_downcast, Downcast};
use glam::Vec3;
use vent_rendering::{buffer::VulkanBuffer, instance::VulkanInstance};

use super::{d3::UBO3D, Dimension};

pub mod camera_controller3d;

pub trait Camera: Downcast {
    fn new(instance: &VulkanInstance, aspect_ratio: f32) -> Self
    where
        Self: Sized;

    fn recreate_projection(&mut self, aspect_ratio: f32);

    fn destroy(&mut self, pool: vk::DescriptorPool, device: &ash::Device);
}
impl_downcast!(Camera);

pub fn from_dimension(
    instance: &VulkanInstance,
    aspect_ratio: f32,
    dimension: &Dimension,
) -> Box<dyn Camera> {
    match dimension {
        Dimension::D2 => Box::new(Camera2D::new(instance, aspect_ratio)),
        Dimension::D3 => Box::new(Camera3D::new(instance, aspect_ratio)),
    }
}

#[allow(dead_code)]
pub struct Camera2D {
    position: glam::Vec2,
}

impl Camera for Camera2D {
    #[inline]
    #[must_use]
    fn new(_instance: &VulkanInstance, _aspect_ratio: f32) -> Self
    where
        Self: Sized,
    {
        Self {
            position: glam::Vec2::ZERO,
        }
    }

    fn destroy(&mut self, _pool: vk::DescriptorPool, _device: &ash::Device) {
        todo!()
    }

    fn recreate_projection(&mut self, _aspect_ratio: f32) {
        todo!()
    }
}

pub struct Camera3D {
    fovy: f32,
    znear: f32,
    zfar: f32,
    ubo: UBO3D,
    pub ubo_buffers: Vec<VulkanBuffer>,

    pub position: glam::Vec3,
    pub rotation: glam::Quat,
}

impl Camera for Camera3D {
    #[inline]
    #[must_use]
    fn new(instance: &VulkanInstance, aspect_ratio: f32) -> Self
    where
        Self: Sized,
    {
        let mut ubo_buffers = vec![];
        for _ in 0..instance.swapchain_images.len() {
            ubo_buffers.push(VulkanBuffer::new_init(
                &instance.device,
                &instance.memory_allocator,
                size_of::<UBO3D>() as vk::DeviceSize,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                &[UBO3D::default()],
            ))
        }

        let mut cam = Self {
            fovy: 60.0,
            znear: 0.1,
            zfar: 10000.0,
            rotation: glam::Quat::IDENTITY,
            position: Vec3::ZERO,
            ubo: Default::default(),
            ubo_buffers,
        };
        // we should configure
        cam.recreate_projection(aspect_ratio);
        cam.recreate_view();

        cam
    }

    fn destroy(&mut self, _pool: vk::DescriptorPool, device: &ash::Device) {
        self.ubo_buffers.iter_mut().for_each(|f| f.destroy(device))
    }

    fn recreate_projection(&mut self, aspect_ratio: f32) {
        let projection =
            glam::Mat4::perspective_rh(self.fovy.to_radians(), aspect_ratio, self.znear, self.zfar);
        self.ubo.projection = projection;
    }
}

impl Camera3D {
    pub fn update_set() {}

    pub fn recreate_view(&mut self) {
        let view =
            glam::Mat4::look_at_rh(self.position, self.direction_from_rotation(), glam::Vec3::Y);
        self.ubo.view_position = self.position;
        self.ubo.view = view;
    }

    pub fn write(&self, instance: &VulkanInstance, index: u32) {
        self.ubo_buffers[index as usize].upload_type(
            &instance.device,
            &self.ubo,
            size_of::<UBO3D>() as vk::DeviceSize,
        )
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
