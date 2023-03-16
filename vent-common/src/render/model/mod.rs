use crate::render::model::model_loader::ModelLoader3D;
use crate::render::Vertex3D;
use wgpu::util::DeviceExt;
use wgpu::Device;

pub mod model_loader;

pub struct Mesh3D {
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,

    index_count: usize,
}

impl Mesh3D {
    pub fn new(device: &Device, path: &str) -> Self {
        let model = ModelLoader3D::load(path);

        Self::new_from(device, model.vertices, model.indices)
    }

    pub fn new_from(device: &Device, vertices: Vec<Vertex3D>, indices: Vec<u32>) -> Self {
        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let index_count = indices.len();

        Self {
            vertex_buf,
            index_buf,
            index_count,
        }
    }

    pub fn bind<'rp>(&'rp self, rpass: &mut wgpu::RenderPass<'rp>) {
        rpass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint32);
        rpass.set_vertex_buffer(0, self.vertex_buf.slice(..));
    }

    pub fn draw<'rp>(&'rp self, rpass: &mut wgpu::RenderPass<'rp>) {
        rpass.draw_indexed(0..self.index_count as u32, 0, 0..1);
    }
}
