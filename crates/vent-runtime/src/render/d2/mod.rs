use ash::vk;

use super::{camera::Camera, Renderer};

#[allow(dead_code)]
pub struct UBO2D {}

pub struct Renderer2D {}

impl Renderer for Renderer2D {
    fn init(
        _instance: &mut vent_rendering::instance::VulkanInstance,
        _camera: &mut dyn Camera,
    ) -> Self
    where
        Self: Sized,
    {
        todo!()
    }

    fn resize(
        &mut self,
        _instance: &mut vent_rendering::instance::VulkanInstance,
        _new_size: (u32, u32),
        _camera: &mut dyn Camera,
    ) {
        todo!()
    }

    fn render(
        &mut self,
        _instance: &vent_rendering::instance::VulkanInstance,
        _image_index: u32,
        _command_buffer: vk::CommandBuffer,
        _camera: &mut dyn Camera,
    ) {
        todo!()
    }

    fn destroy(&mut self, _instance: &vent_rendering::instance::VulkanInstance) {
        todo!()
    }
}
