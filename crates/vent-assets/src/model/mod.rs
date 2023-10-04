use std::mem::size_of;
use std::path::Path;

use ash::vk;
use bytemuck::{Pod, Zeroable};
use vent_rendering::allocator::MemoryAllocator;
use vent_rendering::buffer::VulkanBuffer;
use vent_rendering::instance::{self, Instance};
use vent_rendering::Vertex3D;
use vent_sdk::utils::stopwatch::Stopwatch;

use crate::{Mesh3D, Model3D};

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
        device: &ash::Device,
        path: P,
        allocator: &MemoryAllocator,
    ) -> Self {
        let sw = Stopwatch::new_and_start();
        let model = load_model_from_path(device, path.as_ref(), allocator)
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

    pub fn draw(
        &self,
        device: &ash::Device,
        pipeline_layout: vk::PipelineLayout,
        command_buffer: vk::CommandBuffer,
    ) {
        self.meshes.iter().for_each(|mesh| {
            // rpass.push_debug_group("Bind Mesh");
            mesh.bind(device, command_buffer, pipeline_layout, true);
            // rpass.pop_debug_group();
            // rpass.insert_debug_marker("Draw!");
            mesh.draw(device, command_buffer);
        })
    }
}

async fn load_model_from_path(
    device: &ash::Device,
    path: &Path,
    allocator: &MemoryAllocator,
) -> Result<Model3D, ModelError> {
    if !path.exists() {
        return Err(ModelError::FileNotExists);
    }

    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

    // Very Pretty, I know
    match extension {
        "obj" => Ok(OBJLoader::load(device, path, allocator).await?),
        "gltf" => Ok(GLTFLoader::load(device, path).await?),
        _ => Err(ModelError::UnsupportedFormat),
    }
}

impl Mesh3D {
    pub fn new(
        device: &ash::Device,
        allocator: &MemoryAllocator,
        vertices: &[Vertex3D],
        indices: &[u32],
        bind_group: Option<vk::DescriptorSet>,
        name: Option<&str>,
    ) -> Self {
        let vertex_buf = VulkanBuffer::new_init(
            device,
            allocator,
            (size_of::<Vertex3D>() * vertices.len()) as vk::DeviceSize,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            bytemuck::cast_slice(vertices),
        );

        let index_buf = VulkanBuffer::new_init(
            device,
            allocator,
            (size_of::<Vertex3D>() * indices.len()) as vk::DeviceSize,
            vk::BufferUsageFlags::INDEX_BUFFER,
            bytemuck::cast_slice(indices),
        );

        Self {
            vertex_buf,
            index_buf,
            index_count: indices.len() as u32,
            descriptor_set: bind_group,
        }
    }

    pub fn bind(
        &self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        pipeline_layout: vk::PipelineLayout,
        with_descriptor_set: bool,
    ) {
        unsafe {
            device.cmd_bind_vertex_buffers(command_buffer, 0, &[*self.vertex_buf], &[0]);
            device.cmd_bind_index_buffer(command_buffer, *self.index_buf, 0, vk::IndexType::UINT32);
            if with_descriptor_set {
                if let Some(ds) = self.descriptor_set {
                    device.cmd_bind_descriptor_sets(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        pipeline_layout,
                        0,
                        &[ds],
                        &[],
                    )
                }
            }
        }
    }

    pub fn draw(&self, device: &ash::Device, command_buffer: vk::CommandBuffer) {
        unsafe { device.cmd_draw_indexed(command_buffer, self.index_count, 1, 0, 0, 0) };
    }

    pub fn destroy(&self, device: &ash::Device) {
        self.vertex_buf.destroy(device);
        self.index_buf.destroy(device);
    }
}
