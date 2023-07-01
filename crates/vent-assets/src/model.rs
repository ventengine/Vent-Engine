use std::rc::Rc;

use crate::Asset;
use glam::{Quat, Vec3};
use russimp::mesh::Mesh;
use russimp::node::Node;
use vent_common::render::Vertex3D;
use wgpu::util::DeviceExt;
use wgpu::Device;

pub struct Model3D {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
    meshes: Vec<Mesh3D>,
}

impl Model3D {
    #[inline(always)]
    pub fn new(device: &Device, path: &str) -> Self {
        let model = RawMesh::load_full(path);
        let mut meshes = Vec::new();
        println!("{}", model.len());
        for mesh in model {
            meshes.push(Mesh3D::new(device, mesh.vertices, mesh.indices));
        }

        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            meshes,
        }
    }

    pub fn bind<'rp>(&'rp self, rpass: &mut wgpu::RenderPass<'rp>) {
        for mesh in &self.meshes {
            mesh.bind(rpass)
        }
    }

    pub fn draw<'rp>(&'rp self, rpass: &mut wgpu::RenderPass<'rp>) {
        for mesh in &self.meshes {
            mesh.draw(rpass)
        }
    }
}

impl Asset for Model3D {
    fn get_file_extensions() -> &'static str {
        ""
    }
}

pub struct Mesh3D {
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,

    index_count: u32,
}

impl Mesh3D {
    pub fn new(device: &Device, vertices: Vec<Vertex3D>, indices: Vec<u32>) -> Self {
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

        let index_count = indices.len() as u32;

        Self {
            vertex_buf,
            index_buf,
            index_count,
        }
    }

    pub fn bind<'rp>(&'rp self, rpass: &mut wgpu::RenderPass<'rp>) {
        rpass.set_vertex_buffer(0, self.vertex_buf.slice(..));
        rpass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint32);
    }

    pub fn draw<'rp>(&'rp self, rpass: &mut wgpu::RenderPass<'rp>) {
        rpass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}

use russimp::scene::PostProcess;

pub struct RawMesh {
    pub vertices: Vec<Vertex3D>,
    pub indices: Vec<u32>,
}

impl RawMesh {
    #[inline]
    #[must_use]
    pub fn load_full(path: &str) -> Vec<Self> {
        let scene = russimp::scene::Scene::from_file(
            path,
            vec![
                PostProcess::GenerateSmoothNormals,
                PostProcess::JoinIdenticalVertices,
                PostProcess::Triangulate,
                PostProcess::FixInfacingNormals,
                PostProcess::SortByPrimitiveType,
                PostProcess::PreTransformVertices,
                PostProcess::OptimizeMeshes,
                PostProcess::OptimizeGraph,
                PostProcess::ImproveCacheLocality,
            ],
        )
        .unwrap();

        Self::load_node(
            scene.root.expect("Failed to get Scene Root Node"),
            &scene.meshes,
        )
    }

    fn load_node(node: Rc<Node>, scene_meshes: &Vec<Mesh>) -> Vec<Self> {
        let mut meshes = Vec::with_capacity(node.meshes.len());
        for i in 0..node.meshes.len() {
            meshes.push(Self::load_mesh(&scene_meshes[i]))
        }
        for i in 0..node.children.borrow().len() {
            meshes.extend(Self::load_node(
                node.children.borrow()[i].clone(),
                scene_meshes,
            ));
        }
        meshes
    }

    fn load_mesh(mesh: &Mesh) -> Self {
        let mut vertices = Vec::with_capacity(mesh.vertices.len());
        let indices = mesh
            .faces
            .iter()
            .flat_map(|face| face.0.iter().copied())
            .collect();

        for i in 0..mesh.vertices.len() {
            let mut vertex = Vertex3D::empty();

            let mesh_vertex = &mesh.vertices[i];
            vertex.pos = [mesh_vertex.x, mesh_vertex.y, mesh_vertex.z];

            if mesh.texture_coords[0].is_some() {
                let coord = mesh.texture_coords[0].as_ref().unwrap()[i];
                vertex.tex_coord = [coord.x, coord.y]
            }
            vertices.push(vertex);
        }

        Self { vertices, indices }
    }
}
