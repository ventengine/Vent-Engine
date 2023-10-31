use ash::vk;
use vent_rendering::instance::VulkanInstance;
use vent_runtime::render::Dimension;
use vent_runtime::render::{camera::Camera, RawRuntimeRenderer};

pub struct EditorRuntimeRenderer {
    runtime_renderer: RawRuntimeRenderer,
}

impl EditorRuntimeRenderer {
    pub fn new(
        instance: &VulkanInstance,
        dimension: Dimension,
        event_loop: &winit::event_loop::EventLoopWindowTarget<()>,
        _extent: vk::Extent2D,
        camera: &mut dyn Camera,
    ) -> Self {
        let runtime_renderer = RawRuntimeRenderer::new(dimension, instance, event_loop, camera);
        Self {
            runtime_renderer,
            // extent,
        }
    }

    pub fn render(
        &mut self,
        instance: &VulkanInstance,
        window: &winit::window::Window,
        camera: &mut dyn Camera,
    ) {
        // TODO: Get new image
        self.runtime_renderer.render(instance, window, camera);
    }

    pub fn resize(
        &mut self,
        instance: &VulkanInstance,
        new_size: &winit::dpi::PhysicalSize<u32>,
        _camera: &mut dyn Camera,
    ) {

        // TODO
        // self.runtime_renderer.resize(config, device, queue, camera);
    }
}
