use super::{camera::Camera, Renderer};

pub struct UBO2D {}

pub struct Renderer2D {}

impl Renderer for Renderer2D {
    fn init(_instance: &vent_rendering::instance::VulkanInstance, _camera: &mut dyn Camera) -> Self
    where
        Self: Sized,
    {
        todo!()
    }

    fn resize(
        &mut self,
        _instance: &mut vent_rendering::instance::VulkanInstance,
        _new_size: &winit::dpi::PhysicalSize<u32>,
        _camera: &mut dyn Camera,
    ) {
        todo!()
    }

    fn render(
        &mut self,
        _instance: &vent_rendering::instance::VulkanInstance,
        _image_index: u32,
        _camera: &mut dyn Camera,
    ) {
        todo!()
    }

    fn destroy(&mut self, _instance: &vent_rendering::instance::VulkanInstance) {
        todo!()
    }
}
