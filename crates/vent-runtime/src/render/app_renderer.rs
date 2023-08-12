use crate::render::Dimension;

use crate::render::model_renderer::ModelRenderer3D;
use std::mem;
use std::path::Path;
use vent_assets::{Vertex, Vertex3D};
use vent_common::render::{DefaultRenderer, UBO3D};
use vent_ecs::world::World;
use wgpu::util::DeviceExt;

use super::{
    camera::{Camera, Camera3D},
    d2::Renderer2D,
    d3::Renderer3D,
    model::Model3D,
};

pub struct VentApplicationManager {
    multi_renderer: Box<dyn MultiDimensionRenderer>,
}

impl VentApplicationManager {
    pub fn new(
        dimension: Dimension,
        default_renderer: &DefaultRenderer,
        camera: &mut dyn Camera,
    ) -> Self {
        Self {
            multi_renderer: match dimension {
                Dimension::D2 => Box::new(Renderer2D::init(
                    &default_renderer.config,
                    &default_renderer.adapter,
                    &default_renderer.device,
                    &default_renderer.queue,
                    camera,
                )),
                Dimension::D3 => Box::new(Renderer3D::init(
                    &default_renderer.config,
                    &default_renderer.adapter,
                    &default_renderer.device,
                    &default_renderer.queue,
                    camera,
                )),
            },
        }
    }

    pub fn update(&self) {}

    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        queue: &wgpu::Queue,
        camera: &mut dyn Camera,
        aspect_ratio: f32,
    )  {
        self.multi_renderer
            .render(encoder, view, queue, camera, aspect_ratio)
    }

    pub fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera: &mut dyn Camera,
    ) {
        self.multi_renderer.resize(config, device, queue, camera);
    }
}

pub trait MultiDimensionRenderer {
    fn init<T>(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera: &mut T,
    ) -> Self where Self: Sized;

    fn resize<T>(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera: &mut T,
    ) ;

    fn render<T>(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        queue: &wgpu::Queue,
        camera: &mut T,
        aspect_ratio: f32,
    );
}
