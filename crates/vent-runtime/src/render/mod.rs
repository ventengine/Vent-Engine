use vent_common::render::DefaultRenderer;
use wgpu::SurfaceError;
use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::render::app_renderer::VentApplicationManager;

use self::camera::Camera;

pub mod app_renderer;
pub mod camera;
pub mod model;
mod model_renderer;

pub struct RuntimeRenderer {
    default_renderer: DefaultRenderer,
    app_renderer: VentApplicationManager,
}

pub enum Dimension {
    D2,
    D3,
}

impl RuntimeRenderer {
    pub fn new(
        dimension: Dimension,
        default_renderer: DefaultRenderer,
        camera: &mut dyn Camera,
    ) -> Self {
        Self {
            app_renderer: VentApplicationManager::new(dimension, &default_renderer, camera),
            default_renderer,
        }
    }

    pub fn render(
        &mut self,
        _window: &Window,
        camera: &mut dyn Camera,
    ) -> Result<(), SurfaceError> {
        let output = self.default_renderer.surface.get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Runtime View"),
            ..Default::default()
        });

        let mut encoder =
            self.default_renderer
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Runtime Render Encoder"),
                });
        self.app_renderer.render(
            &mut encoder,
            &view,
            &self.default_renderer.queue,
            camera,
            self.default_renderer.config.width as f32 / self.default_renderer.config.height as f32,
        );

        self.default_renderer.queue.submit(Some(encoder.finish()));
        output.present();

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(offscreen_canvas_setup) = &self.default_renderer.offscreen_canvas_setup {
                let image_bitmap = offscreen_canvas_setup
                    .offscreen_canvas
                    .transfer_to_image_bitmap()
                    .expect("couldn't transfer offscreen canvas to image bitmap.");
                offscreen_canvas_setup
                    .bitmap_renderer
                    .transfer_from_image_bitmap(&image_bitmap);

                log::info!("Transferring OffscreenCanvas to ImageBitmapRenderer");
            }
        }

        Ok(())
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>, camera: &mut dyn Camera) {
        DefaultRenderer::resize(&mut self.default_renderer, new_size);
        self.app_renderer.resize(
            &self.default_renderer.config,
            &self.default_renderer.device,
            &self.default_renderer.queue,
            camera,
        )
    }
}
