use crate::gui::EditorGUI;
use crate::render::runtime_renderer::EditorRuntimeRenderer;
use ash::vk;
use vent_rendering::instance::VulkanInstance;
use vent_runtime::render::camera::Camera;
use vent_runtime::render::gui::debug_gui::RenderData;
use vent_runtime::render::gui::gui_renderer::EguiRenderer;
use vent_runtime::render::Dimension;

mod runtime_renderer;

pub struct EditorRenderer {
    instance: VulkanInstance,
    pub egui: EguiRenderer,

    editor_runtime_renderer: EditorRuntimeRenderer,
}

impl EditorRenderer {
    pub fn new(
        window: &winit::window::Window,
        event_loop: &winit::event_loop::EventLoopWindowTarget<()>,
        camera: &mut dyn Camera,
    ) -> Self {
        let instance = VulkanInstance::new("Vent-Engine Editor", window);
        let egui = EguiRenderer::new().add_gui(Box::new(EditorGUI::new()));

        let editor_runtime_renderer = EditorRuntimeRenderer::new(
            &instance,
            Dimension::D3,
            // TODO
            event_loop,
            vk::Extent2D {
                width: 1,
                height: 1,
            }, // TODO
            camera,
        );

        Self {
            instance,
            egui,
            editor_runtime_renderer,
        }
    }

    pub fn render(&mut self, window: &winit::window::Window, camera: &mut dyn Camera) {
        self.egui.render(&RenderData::default());

        self.editor_runtime_renderer
            .render(&self.instance, window, camera);
    }

    pub fn resize(&mut self, new_size: &winit::dpi::PhysicalSize<u32>, camera: &mut dyn Camera) {
        // TODO
        self.editor_runtime_renderer
            .resize(&self.instance, new_size, camera);
        // egui does resize using Window Events
    }

    pub fn resize_current(&mut self, camera: &mut dyn Camera) {
        let size = winit::dpi::PhysicalSize {
            width: 1,
            height: 1,
        }; // TODO
        Self::resize(self, &size, camera)
    }
}
