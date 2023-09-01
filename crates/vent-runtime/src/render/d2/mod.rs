use std::any::Any;

use super::{camera::Camera, Renderer};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UBO2D {}

pub struct Renderer2D {}

impl Renderer for Renderer2D {
    fn init(
        _config: &wgpu::SurfaceConfiguration,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _camera: &mut dyn Any,
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
    ) {
        todo!()
    }

    fn render(
        &mut self,
        _encoder: &mut wgpu::CommandEncoder,
        _view: &wgpu::TextureView,
        _depth_view: &wgpu::TextureView,
        _queue: &wgpu::Queue,
    ) {
        todo!()
    }
}
