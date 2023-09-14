use crate::gui::EditorGUI;
use crate::render::runtime_renderer::EditorRuntimeRenderer;
use vent_common::render::WGPURenderer;
use vent_runtime::render::camera::Camera;
use vent_runtime::render::gui::debug_gui::RenderData;
use vent_runtime::render::gui::gui_renderer::EguiRenderer;
use vent_runtime::render::Dimension;

mod runtime_renderer;

pub struct EditorRenderer {
    default_renderer: WGPURenderer,
    pub egui: EguiRenderer,

    editor_runtime_renderer: EditorRuntimeRenderer,
}

impl EditorRenderer {
    pub fn new(
        window: &winit::window::Window,
        event_loop: &winit::event_loop::EventLoopWindowTarget<()>,
        camera: &mut dyn Camera,
    ) -> Self {
        let default_renderer = WGPURenderer::new(window);
        let egui = EguiRenderer::new(
            event_loop,
            &default_renderer.device,
            default_renderer.caps.formats[0],
        )
        .add_gui(Box::new(EditorGUI::new()));

        let editor_runtime_renderer = EditorRuntimeRenderer::new(
            &default_renderer,
            // TODO
            Dimension::D3,
            event_loop,
            wgpu::Extent3d {
                width: default_renderer.config.width,
                height: default_renderer.config.height,
                depth_or_array_layers: 1,
            },
            camera,
        );

        Self {
            default_renderer,
            egui,
            editor_runtime_renderer,
        }
    }

    pub fn render(
        &mut self,
        window: &winit::window::Window,
        camera: &mut dyn Camera,
    ) -> Result<(), wgpu::SurfaceError> {
        let output = self.default_renderer.surface.get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Editor View"),
            ..Default::default()
        });
        let mut encoder =
            self.default_renderer
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Editor Render Encoder"),
                });

        self.egui.render(
            &view,
            window,
            &self.default_renderer.device,
            &self.default_renderer.queue,
            &mut encoder,
            &RenderData::default(),
        );

        self.default_renderer.queue.submit(Some(encoder.finish()));

        self.editor_runtime_renderer
            .render(&self.default_renderer, window, camera)?;
        output.present();
        Ok(())
    }

    pub fn resize(&mut self, new_size: &winit::dpi::PhysicalSize<u32>, camera: &mut dyn Camera) {
        self.default_renderer.resize(new_size);
        // TODO
        self.editor_runtime_renderer.resize(
            &self.default_renderer.device,
            &self.default_renderer.queue,
            &self.default_renderer.config,
            new_size,
            camera,
        );
        // egui does resize using Window Events
    }

    pub fn resize_current(&mut self, camera: &mut dyn Camera) {
        let size = winit::dpi::PhysicalSize {
            width: self.default_renderer.config.width,
            height: self.default_renderer.config.height,
        };
        Self::resize(self, &size, camera)
    }
}
