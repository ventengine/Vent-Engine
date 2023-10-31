use super::{camera::Camera, Renderer};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UBO2D {}

pub struct Renderer2D {}

impl Renderer for Renderer2D {
    fn init(instance: &vent_rendering::instance::VulkanInstance, camera: &mut dyn Camera) -> Self
    where
        Self: Sized,
    {
        todo!()
    }

    fn resize(
        &mut self,
        instance: &vent_rendering::instance::VulkanInstance,
        new_size: &winit::dpi::PhysicalSize<u32>,
        camera: &mut dyn Camera,
    ) {
        todo!()
    }

    fn render(
        &mut self,
        instance: &vent_rendering::instance::VulkanInstance,
        camera: &mut dyn Camera,
    ) {
        todo!()
    }
}
