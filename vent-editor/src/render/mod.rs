use crate::render::gui_renderer::EguiRenderer;
use crate::render::runtime_renderer::EditorRuntimeRenderer;
use vent_common::render::{DefaultRenderer, Renderer};
use vent_runtime::render::Dimension;
use wgpu::{Extent3d, SurfaceError};
use winit::dpi::PhysicalSize;
use winit::window::Window;
use vent_common::entities::camera::BasicCameraImpl;

mod gui_renderer;
mod runtime_renderer;

pub struct EditorRenderer {
    default_renderer: DefaultRenderer,
    pub egui: EguiRenderer,

    pub editor_runtime_renderer: EditorRuntimeRenderer,
}

impl EditorRenderer {
    pub fn new(window: &Window, camera: &mut dyn BasicCameraImpl) -> Self {
        let default_renderer: DefaultRenderer = Renderer::new(window);
        let egui = EguiRenderer::new(
            window,
            &default_renderer.device,
            default_renderer.caps.formats[0],
        );

        let editor_runtime_renderer = EditorRuntimeRenderer::new(
            &default_renderer,
            // TODO
            Dimension::D3,
            Extent3d {
                width: &default_renderer.config.width / 2,
                height: &default_renderer.config.height / 2,
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

    pub fn render(&mut self, window: &Window, camera: &mut dyn BasicCameraImpl) -> Result<(), SurfaceError> {
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

        {
            let mut _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Editor Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.4,
                            g: 0.1,
                            b: 0.6,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
        }

        self.egui.render(
            window,
            &self.default_renderer.device,
            &self.default_renderer.queue,
            &view,
            &mut encoder,
        );

        self.editor_runtime_renderer
            .render(window, &mut encoder, &self.default_renderer.queue, camera)
            .expect("Failed to Render Runtime inside Editor");

        self.default_renderer
            .queue
            .submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    pub fn resize(&mut self, window: &Window, new_size: PhysicalSize<u32>, camera: &mut dyn BasicCameraImpl) {
        Renderer::resize(&mut self.default_renderer, window, new_size);
        // TODO
        self.editor_runtime_renderer.resize(
            &self.default_renderer.device,
            &self.default_renderer.queue,
            &self.default_renderer.config,
            &new_size,
            camera,
        );
        // egui does Automatically resize
    }
}
