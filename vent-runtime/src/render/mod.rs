use vent_common::render::{DefaultRenderer, Renderer};
use wgpu::{CommandEncoder, SurfaceError, SurfaceTexture};
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub struct RuntimeRenderer {
    default_renderer: DefaultRenderer,
}

impl RuntimeRenderer {
    pub(crate) fn new(window: &Window) -> Self {
        let renderer = Renderer::new(window);
        Self {
            default_renderer: renderer,
        }
    }

    pub fn render(&self, _window: &Window) -> Result<(), SurfaceError> {
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
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
        self.default_renderer
            .queue
            .submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    pub fn render_from(
        _window: &Window,
        _encoder: &mut &CommandEncoder,
        _view: &SurfaceTexture,
    ) -> Result<(), SurfaceError> {
        Ok(())
    }

    pub fn resize(&mut self, window: &Window, new_size: PhysicalSize<u32>) {
        RuntimeRenderer::resize_from(&mut self.default_renderer, window, new_size)
    }

    pub fn resize_from(
        renderer: &mut DefaultRenderer,
        window: &Window,
        new_size: PhysicalSize<u32>,
    ) {
        Renderer::resize(renderer, window, new_size);
    }
}
