use std::path::Path;

use bytemuck::{Pod, Zeroable};
use vent_sdk::utils::stopwatch::Stopwatch;
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
    pub async fn load<P: AsRef<Path>>(
        device: &Device,
        queue: &wgpu::Queue,
        path: P,
        texture_bind_group_layout: &BindGroupLayout,
    ) -> Self {
        let sw = Stopwatch::new_and_start();
        let model = load_model_from_path(device, queue, path.as_ref(), texture_bind_group_layout)
            .await
            .expect("Failed to Load 3D Model");
        log::info!(
            "Model {} took {}ms to Load, {} Meshes",
            path.as_ref().to_str().unwrap(),
            sw.elapsed_ms(),
            model.meshes.len(),
        );
        model
    }

    pub fn draw<'rp>(&'rp self, rpass: &mut wgpu::RenderPass<'rp>) {
        self.meshes.iter().for_each(|mesh| {
            rpass.push_debug_group("Bind Mesh");
            mesh.bind(rpass, true);
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

    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

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
        bind_group: Option<wgpu::BindGroup>,
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
            bind_group,
        }
    }

    pub fn bind<'rp>(&'rp self, rpass: &mut wgpu::RenderPass<'rp>, with_group: bool) {
        rpass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint32);
        rpass.set_vertex_buffer(0, self.vertex_buf.slice(..));
        if with_group {
            if let Some(bg) = self.bind_group.as_ref() {
                rpass.set_bind_group(1, bg, &[]);
            }
        }
    }

    pub fn draw(&self, rpass: &mut wgpu::RenderPass<'_>) {
        rpass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}
