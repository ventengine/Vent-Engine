use std::{path::Path, rc::Rc};

use glam::{Quat, Vec3};
use vent_common::render::Vertex3D;
use vent_dev::utils::stopwatch::Stopwatch;
use wgpu::util::DeviceExt;
use wgpu::Device;

use self::obj::OBJLoader;

mod obj;

#[derive(Debug)]
pub enum ModelError {
    UnsupportedFormat,
    FileNotExists,
    LoadingError(String),
}

pub struct Model3D {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
    meshes: Vec<Mesh3D>,
}

impl Model3D {
    #[inline(always)]
    pub fn new(device: &Device, path: &Path) -> Self {
        let meshes = load_model_from_path(device, path).expect("Failed to Load 3D Model");

        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            meshes,
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

fn load_model_from_path(device: &wgpu::Device, path: &Path) -> Result<Vec<Mesh3D>, ModelError> {
    if !path.exists() {
        return Err(ModelError::FileNotExists);
    }

    // Very Pretty, I know
    match path.extension().unwrap().to_str().unwrap() {
        "obj" => Ok(OBJLoader::load(device, path)),
        _ => Err(ModelError::UnsupportedFormat),
    }
}

pub struct Mesh3D {
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,

    index_count: u32,
}

impl Mesh3D {
    pub fn new(device: &Device, vertices: &Vec<Vertex3D>, indices: &Vec<u32>, name: &str) -> Self {
        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Vertex Buffer", name)),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Index Buffer", name)),
            contents: bytemuck::cast_slice(indices),
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
