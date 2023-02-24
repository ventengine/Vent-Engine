use crate::render::gui_renderer::{EguiRenderer};
use vent_common::render::{DefaultRenderer, Renderer};
use wgpu::SurfaceError;
use winit::dpi::PhysicalSize;
use winit::window::Window;

mod gui_renderer;
mod runtime_renderer;

pub struct EditorRenderer {
    default_renderer: DefaultRenderer,
    pub egui: EguiRenderer,
}

impl EditorRenderer {
    pub(crate) fn new(window: &Window) -> Self {
        let default_renderer: DefaultRenderer = Renderer::new(window);
        let egui = EguiRenderer::new(
            window,
            &default_renderer.device,
            default_renderer.caps.formats[0],
        );

        Self {
            default_renderer,
            egui,
        }
    }

    pub fn render(&mut self, window: &Window) -> Result<(), SurfaceError> {
        let output = self.default_renderer.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());


        let mut encoder =
            self.default_renderer
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            let mut _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
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

        self.egui
            .render(
                window,
                &self.default_renderer.device,
                &self.default_renderer.queue,
                &view,
                &mut encoder,
            );

        self.default_renderer
            .queue
            .submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    pub fn resize(&mut self, window: &Window, new_size: PhysicalSize<u32>) {
        Renderer::resize(&mut self.default_renderer, window, new_size);
    }
}
