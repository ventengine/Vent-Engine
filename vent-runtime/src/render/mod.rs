use vent_common::render::{DefaultRenderer, Renderer};
use wgpu::{
    include_wgsl, Adapter, CommandEncoder, Device, Queue, SurfaceConfiguration, SurfaceError,
    SurfaceTexture, TextureView,
};
use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::render::app_renderer::{AppRenderer, MultiDimensionRenderer, Renderer2D, Renderer3D};
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

pub mod app_renderer;

pub struct RuntimeRenderer {
    default_renderer: DefaultRenderer,
    app_renderer: AppRenderer,
}

pub enum Dimension {
    D2,
    D3,
}

impl RuntimeRenderer {
    pub fn new(dimension: Dimension, default_renderer: DefaultRenderer) -> Self {
        Self {
            app_renderer: AppRenderer::new(dimension, &default_renderer),
            default_renderer,
        }
    }

    pub fn render(&self, window: &Window) -> Result<(), SurfaceError> {
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
        self.app_renderer.render(&mut encoder, &view);

        self.default_renderer
            .queue
            .submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    pub fn resize(&mut self, window: &Window, new_size: PhysicalSize<u32>) {
        Renderer::resize(&mut self.default_renderer, window, new_size);
        self.app_renderer.resize(&self.default_renderer.config, &self.default_renderer.device, &self.default_renderer.queue)
    }
}
