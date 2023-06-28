use crate::Asset;
use glam::{Quat, Vec3};
use vent_common::render::Vertex3D;
use wgpu::util::DeviceExt;
use wgpu::Device;

pub struct Mesh3D {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,

    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,

    index_count: usize,
}

impl Mesh3D {
    #[inline(always)]
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
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
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

impl Asset for Mesh3D {
    fn get_file_extensions() -> &'static str {
        ""
    }
}

use russimp::scene::PostProcess;

pub struct ModelLoader3D {
    pub vertices: Vec<Vertex3D>,
    pub indices: Vec<u32>,

    pub materials: Vec<russimp::material::Material>,
}

impl ModelLoader3D {
    #[inline]
    #[must_use]
    pub fn load(path: &str) -> Self {
        let scene = russimp::scene::Scene::from_file(
            path,
            vec![
                PostProcess::CalculateTangentSpace,
                PostProcess::Triangulate,
                PostProcess::JoinIdenticalVertices,
                PostProcess::SortByPrimitiveType,
                PostProcess::OptimizeMeshes,
                PostProcess::OptimizeGraph,
                PostProcess::ImproveCacheLocality,
            ],
        )
        .unwrap();

        let mut vertices = Vec::with_capacity(scene.meshes.iter().map(|m| m.vertices.len()).sum());
        let mut indices = Vec::new();
        indices.reserve(scene.meshes.iter().map(|m| m.faces.len() * 3).sum());

        for mesh in &scene.meshes {
            indices.extend(mesh.faces.iter().flat_map(|face| face.0.iter().copied()));

            vertices.extend(mesh.vertices.iter().map(|vertex| Vertex3D {
                pos: [vertex.x, vertex.y, vertex.z],
                tex_coord: [0.0, 0.0],
            }));
        }
        let mats = scene.materials;

        Self {
            vertices,
            indices,
            materials: mats,
        }
    }
}
