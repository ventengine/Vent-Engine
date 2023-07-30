use std::io;
use std::path::Path;

use cfg_if::cfg_if;
use glam::{Quat, Vec3};

use wgpu::util::DeviceExt;
use wgpu::{BindGroupLayout, Device};

use crate::{Mesh3D, Model3D, Vertex3D};

use self::obj::OBJLoader;

mod obj;

#[derive(Debug)]
pub enum ModelError {
    UnsupportedFormat,
    FileNotExists,
    LoadingError(String),
}

impl Model3D {
    #[inline(always)]
    pub fn new(device: &Device, queue: &wgpu::Queue, path: &Path, texture_bind_group_layout: BindGroupLayout) -> Self {
        let meshes = load_model_from_path(device, queue, path, texture_bind_group_layout)
            .expect("Failed to Load 3D Model");

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

fn load_model_from_path(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    path: &Path,
    texture_bind_group_layout: BindGroupLayout,
) -> Result<Vec<Mesh3D>, ModelError> {
    if !path.exists() {
        return Err(ModelError::FileNotExists);
    }

    // Very Pretty, I know
    match path.extension().unwrap().to_str().unwrap() {
        "obj" => Ok(OBJLoader::load(device, queue, path, texture_bind_group_layout)?),
        _ => Err(ModelError::UnsupportedFormat),
    }
}

impl Mesh3D {
    pub fn new(device: &Device, vertices: &[Vertex3D], indices: &[u32], name: &str) -> Self {
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

#[allow(dead_code)]
pub(crate) async fn load_binary(dir: &Path, file_name: &str) -> io::Result<Vec<u8>> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            let data = reqwest::get(url)
                .await?
                .bytes()
                .await?
                .to_vec();
        } else {
            let path = dir
                .join("res")
                .join(file_name);
            let data = std::fs::read(path)?;
        }
    }

    Ok(data)
}
