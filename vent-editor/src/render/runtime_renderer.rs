use egui::Window;
use vent_runtime::render::RuntimeRenderer;
use wgpu::{
    CommandEncoder, Device, Extent3d, SurfaceConfiguration, SurfaceError, Texture,
    TextureDimension, TextureFormat, TextureUsages, TextureView,
};
use winit::dpi::PhysicalSize;

pub struct EditorRuntimeRenderer {
    texture: Texture,
}

impl EditorRuntimeRenderer {
    pub fn new(device: &Device, config: &SurfaceConfiguration, extent: Extent3d) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: config.format,
            usage: config.usage,
            view_formats: &[],
        });

        Self { texture }
    }

    pub fn render(
        &self,
        window: &winit::window::Window,
        encoder: &mut CommandEncoder,
    ) -> Result<(), SurfaceError> {
        let view = self.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Runtime View"),
            ..Default::default()
        });

        RuntimeRenderer::render_from(window, encoder, &view)
    }

    pub fn resize(
        &mut self,
        device: &Device,
        config: &SurfaceConfiguration,
        new_size: &PhysicalSize<u32>,
    ) {
        self.texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: Extent3d {
                width: new_size.width,
                height: new_size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: config.format,
            usage: config.usage,
            view_formats: &[],
        });
    }
}
