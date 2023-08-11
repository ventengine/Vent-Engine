use std::path::Path;

use bytemuck::{Pod, Zeroable};
use vent_dev::utils::stopwatch::Stopwatch;
use wgpu::util::DeviceExt;
use wgpu::{BindGroupLayout, Device};

use crate::{Mesh3D, Model3D, Vertex3D};

use self::gltf::GLTFLoader;
use self::obj::OBJLoader;

mod gltf;
mod obj;

#[derive(Debug)]
pub enum ModelError {
    UnsupportedFormat,
    FileNotExists,
    LoadingError(String),
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Material {
    pub base_color: [f32; 4],
}

impl Model3D {
    #[inline]
    pub async fn load(
        device: &Device,
        queue: &wgpu::Queue,
        path: &Path,
        texture_bind_group_layout: &BindGroupLayout,
    ) -> Self {
        let sw = Stopwatch::new_and_start();
        let model = load_model_from_path(device, queue, path, texture_bind_group_layout)
            .await
            .expect("Failed to Load 3D Model");
        log::info!(
            "Model {} took {}ms to Load",
            path.to_str().unwrap(),
            sw.elapsed_ms()
        );
        model
    }

    pub fn draw<'rp>(&'rp self, rpass: &mut wgpu::RenderPass<'rp>) {
        self.meshes.iter().for_each(| mesh | {
            rpass.push_debug_group("Bind Mesh");
            mesh.bind_with_material(rpass, &self.materials[mesh.material_id]);
            rpass.pop_debug_group();
            rpass.insert_debug_marker("Draw!");
            mesh.draw(rpass);
        })
    }
}

async fn load_model_from_path(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    path: &Path,
    texture_bind_group_layout: &BindGroupLayout,
) -> Result<Model3D, ModelError> {
    if !path.exists() {
        return Err(ModelError::FileNotExists);
    }

    let extension = match path.extension() {
        Some(r) => r.to_str().unwrap(),
        None => return Err(ModelError::UnsupportedFormat),
    };

    // Very Pretty, I know
    match extension {
        "obj" => Ok(OBJLoader::load(device, queue, path, texture_bind_group_layout).await?),
        "gltf" => Ok(GLTFLoader::load(device, queue, path, texture_bind_group_layout).await?),
        _ => Err(ModelError::UnsupportedFormat),
    }
}

impl Mesh3D {
    pub fn new(
        device: &Device,
        vertices: &[Vertex3D],
        indices: &[u32],
        material_id: usize,
        name: Option<&str>,
    ) -> Self {
        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: name,
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: name,
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buf,
            index_buf,
            index_count: indices.len() as u32,
            material_id,
        }
    }

    pub fn bind<'rp>(&'rp self, rpass: &mut wgpu::RenderPass<'rp>) {
        rpass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint32);
        rpass.set_vertex_buffer(0, self.vertex_buf.slice(..));
    }

    pub fn bind_with_material<'rp>(
        &'rp self,
        rpass: &mut wgpu::RenderPass<'rp>,
        material_bind_group: &'rp wgpu::BindGroup,
    ) {
        rpass.set_bind_group(1, material_bind_group, &[]);
        Self::bind(self, rpass)
    }

    pub fn draw<'rp>(&'rp self, rpass: &mut wgpu::RenderPass<'rp>) {
        rpass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}
