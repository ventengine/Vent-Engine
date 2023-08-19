use vent_common::render::DefaultRenderer;
use vent_runtime::render::camera::Camera;
use vent_runtime::render::Dimension;
use wgpu::{
    CommandEncoder, Device, Extent3d, Queue, SurfaceConfiguration, SurfaceError, Texture,
    TextureDimension,
};

pub struct EditorRuntimeRenderer {
    texture: Texture,
    // runtime_renderer: RuntimeRenderer,
}

impl EditorRuntimeRenderer {
    pub fn new(
        default_renderer: &DefaultRenderer,
        _dimension: Dimension,
        _extent: Extent3d,
        _camera: &mut dyn Camera,
    ) -> Self {
        let texture = default_renderer
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Editor Runtime Texture"),
                size: Extent3d::default(),
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: default_renderer.config.format,
                usage: default_renderer.config.usage,
                view_formats: &[],
            });
        // let runtime_renderer = RuntimeRenderer::new(dimension, default_renderer, camera);
        Self {
            texture,
            // runtime_renderer,
            // extent,
        }
    }

    pub fn render(
        &mut self,
        _window: &winit::window::Window,
        _encoder: &mut CommandEncoder,
        _queue: &Queue,
        _camera: &mut dyn Camera,
    ) -> Result<(), SurfaceError> {
        let _view = self.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Runtime View"),
            ..Default::default()
        });
        // TODO
        // self.runtime_renderer.render(
        //     encoder,
        //     &view,
        //     queue,
        //     camera,
        //     self.extent.width as f32 / self.extent.height as f32,
        // );
        Ok(())
    }

    pub fn resize(
        &mut self,
        device: &Device,
        _queue: &Queue,
        config: &SurfaceConfiguration,
        new_size: &winit::dpi::PhysicalSize<u32>,
        _camera: &mut dyn Camera,
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
        // TODO
        // self.runtime_renderer.resize(config, device, queue, camera);
    }
}
