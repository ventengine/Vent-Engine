use vent_common::render::WGPURenderer;
use vent_runtime::render::Dimension;
use vent_runtime::render::{camera::Camera, RawRuntimeRenderer};
use wgpu::{
    Device, Extent3d, Queue, SurfaceConfiguration, SurfaceError, Texture, TextureDimension,
};

pub struct EditorRuntimeRenderer {
    texture: Texture,
    runtime_renderer: RawRuntimeRenderer,
}

impl EditorRuntimeRenderer {
    pub fn new(
        default_renderer: &WGPURenderer,
        dimension: Dimension,
        event_loop: &winit::event_loop::EventLoopWindowTarget<()>,
        _extent: Extent3d,
        camera: &mut dyn Camera,
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
        let runtime_renderer = RawRuntimeRenderer::new(
            dimension,
            &default_renderer.device,
            &default_renderer.queue,
            &default_renderer.config,
            default_renderer.caps.formats[0],
            &default_renderer.adapter,
            event_loop,
            camera,
        );
        Self {
            texture,
            runtime_renderer,
            // extent,
        }
    }

    pub fn render(
        &mut self,
        default_renderer: &WGPURenderer,
        window: &winit::window::Window,
        camera: &mut dyn Camera,
    ) -> Result<(), SurfaceError> {
        let view = self.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Runtime View"),
            ..Default::default()
        });
        self.runtime_renderer.render(
            &view,
            &default_renderer.device,
            &default_renderer.queue,
            window,
            camera,
        );
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
