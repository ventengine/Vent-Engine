use std::rc::Rc;

use crate::Asset;
use glam::{Quat, Vec3};
use russimp::mesh::Mesh;
use russimp::node::Node;
use vent_common::render::Vertex3D;
use vent_dev::utils::stopwatch::Stopwatch;
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
        let scene = RawMesh::load_full(path);

        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            meshes: RawMesh::to_meshes(device, scene),
        }
    }

    pub fn draw<'rp>(&'rp self, rpass: &mut wgpu::RenderPass<'rp>) {
        for mesh in &self.meshes {
            rpass.push_debug_group("Bind Mesh");
            mesh.bind(rpass);
            rpass.insert_debug_marker("Draw!");
            mesh.draw(rpass);
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
    pub fn new(device: &Device, vertices: Vec<Vertex3D>, indices: Vec<u32>, name: String) -> Self {
        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Vertex Buffer", name)),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Index Buffer", name)),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buf,
            index_buf,
            index_count: indices.len() as u32,
        }
    }

    pub fn bind<'rp>(&'rp self, rpass: &mut wgpu::RenderPass<'rp>) {
        rpass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint32);
        rpass.set_vertex_buffer(0, self.vertex_buf.slice(..));
    }

    pub fn draw<'rp>(&'rp self, rpass: &mut wgpu::RenderPass<'rp>) {
        rpass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}

use russimp::scene::{PostProcess, Scene};

struct RawMesh {
    pub name: String,
    vertices: Vec<Vertex3D>,
    indices: Vec<u32>,
}

impl RawMesh {
    #[inline]
    #[must_use]
    pub fn load_full(path: &str) -> Vec<Self> {
        let sw = Stopwatch::new_and_start();

        let scene = russimp::scene::Scene::from_file(
            path,
            vec![
                PostProcess::CalculateTangentSpace,
                PostProcess::Triangulate,
                PostProcess::JoinIdenticalVertices,
                PostProcess::SortByPrimitiveType,
            ],
        )
        .expect("Failed to Load Scene");

        let mut meshes = Vec::with_capacity(scene.meshes.len());
        Self::load_node(
            &scene.root.clone().expect("Failed to get Scene Root Node"),
            &scene,
            &mut meshes,
        );
        log::debug!("Scene {} took {}ms to load", path, sw.elapsed_ms());
        meshes
    }

    pub fn to_meshes(device: &Device, rawmeshes: Vec<Self>) -> Vec<Mesh3D> {
        rawmeshes
            .into_iter()
            .map(|mesh| Mesh3D::new(device, mesh.vertices, mesh.indices, mesh.name))
            .collect()
    }

    fn load_node(node: &Rc<Node>, scene: &Scene, meshes: &mut Vec<RawMesh>) {
        log::debug!("Loading Model Node: {}", node.name);
        for mesh in &node.meshes {
            meshes.push(Self::load_mesh(&scene.meshes[*mesh as usize]))
        }

        for node in node.children.borrow().iter() {
            Self::load_node(node, scene, meshes);
        }
    }

    fn load_mesh(mesh: &Mesh) -> Self {
        let mut vertices = Vec::with_capacity(mesh.vertices.len());

        for i in 0..mesh.vertices.len() {
            let mut vertex = Vertex3D::empty();

            vertex.pos = [mesh.vertices[i].x, mesh.vertices[i].y, mesh.vertices[i].z];

            if mesh.texture_coords[0].is_some() {
                let coord = &mesh.texture_coords[0]
                    .as_ref()
                    .expect("Failed to get texture coords")[i];
                vertex.tex_coord = [coord.x, coord.y]
            }
            vertices.push(vertex);
        }

        let indices: Vec<u32> = mesh
            .faces
            .iter()
            .flat_map(|face| face.0.iter().copied())
            .collect();

        Self {
            name: mesh.name.clone(),
            vertices,
            indices,
        }
    }
}
