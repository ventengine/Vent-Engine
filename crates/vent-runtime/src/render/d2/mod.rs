use super::{app_renderer::MultiDimensionRenderer, camera::Camera2D};

pub struct Renderer2D {}

impl MultiDimensionRenderer for Renderer2D {
    fn init(
        _config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _camera: &mut Camera2D,
    ) -> Self
    where
        Self: Sized,
    {
        Self {}
    }

    fn resize(
        &mut self,
        _config: &wgpu::SurfaceConfiguration,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _camera: &mut Camera2D,
    ) {
        todo!()
    }

    fn render(
        &mut self,
        _encoder: &mut wgpu::CommandEncoder,
        _view: &wgpu::TextureView,
        _queue: &wgpu::Queue,
        _camera: &mut Camera2D,
        _aspect_ratio: f32,
    ) {
        todo!()
    }
}
