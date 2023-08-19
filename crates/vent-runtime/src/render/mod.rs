use vent_common::render::DefaultRenderer;
use wgpu::SurfaceError;
use winit::dpi::PhysicalSize;
use winit::window::Window;

use self::camera::Camera;
use self::d2::Renderer2D;
use self::d3::Renderer3D;

pub mod camera;
pub mod model;
mod model_renderer;

mod d2;
mod d3;

pub struct RuntimeRenderer {
    default_renderer: DefaultRenderer,
    multi_renderer: Box<dyn Renderer>,
}

pub enum Dimension {
    D2,
    D3,
}

pub trait Renderer {
    fn init(
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera: &mut dyn Camera,
    ) -> Self
    where
        Self: Sized;

    fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera: &mut dyn Camera,
    );

    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        queue: &wgpu::Queue,
        camera: &mut dyn Camera,
        aspect_ratio: f32,
    );
}

impl RuntimeRenderer {
    pub fn new(
        dimension: Dimension,
        default_renderer: DefaultRenderer,
        camera: &mut dyn Camera,
    ) -> Self {
        Self {
            multi_renderer: match dimension {
                Dimension::D2 => Box::new(Renderer2D::init(
                    &default_renderer.config,
                    &default_renderer.device,
                    &default_renderer.queue,
                    camera,
                )),
                Dimension::D3 => Box::new(Renderer3D::init(
                    &default_renderer.config,
                    &default_renderer.device,
                    &default_renderer.queue,
                    camera,
                )),
            },
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
        self.multi_renderer.render(
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

    pub fn resize(&mut self, new_size: &PhysicalSize<u32>, camera: &mut dyn Camera) {
        // Its Important to resize Default Renderer first
        self.default_renderer.resize(new_size);
        self.multi_renderer.resize(
            &self.default_renderer.config,
            &self.default_renderer.device,
            &self.default_renderer.queue,
            camera,
        )
    }
}
