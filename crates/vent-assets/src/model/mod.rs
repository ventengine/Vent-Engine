use std::path::Path;

use ash::vk;
use vent_rendering::allocator::MemoryAllocator;
use vent_rendering::buffer::VulkanBuffer;
use vent_rendering::instance::VulkanInstance;
use vent_rendering::{begin_single_time_command, end_single_time_command, Vertex3D};
use vent_sdk::utils::stopwatch::Stopwatch;

use crate::{Material, Mesh3D, Model3D};

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

impl Model3D {
    #[inline]
    pub async fn load<P: AsRef<Path>>(instance: &VulkanInstance, path: P) -> Self {
        let sw = Stopwatch::new_and_start();
        log::info!("Loading new Model...");
        let model = load_model_from_path(instance, path.as_ref())
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
        buffer_index: usize,
    ) {
        self.meshes.iter().for_each(|mesh| {
            // rpass.push_debug_group("Bind Mesh");
            mesh.bind(device, command_buffer, buffer_index, pipeline_layout, true);
            // rpass.pop_debug_group();
            // rpass.insert_debug_marker("Draw!");
            mesh.draw(device, command_buffer);
        })
    }

    pub fn destroy(&mut self, instance: &VulkanInstance) {
        self.meshes
            .drain(..)
            .for_each(|mut mesh| mesh.destroy(instance.descriptor_pool, &instance.device));
    }
}

async fn load_model_from_path(
    instance: &VulkanInstance,
    path: &Path,
) -> Result<Model3D, ModelError> {
    if !path.exists() {
        return Err(ModelError::FileNotExists);
    }

    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

    // Very Pretty, I know
    match extension {
        "obj" => Ok(OBJLoader::load(instance, path).await?),
        "gltf" => Ok(GLTFLoader::load(instance, path).await?),
        _ => Err(ModelError::UnsupportedFormat),
    }
}

impl Mesh3D {
    pub fn new(
        instance: &VulkanInstance,
        allocator: &MemoryAllocator,
        vertices: &[Vertex3D],
        indices: &[u32],
        material: Option<Material>,
        name: Option<&str>,
    ) -> Self {
        let vertex_size = std::mem::size_of_val(vertices) as vk::DeviceSize;
        let index_size = std::mem::size_of_val(indices) as vk::DeviceSize;

        let vertex_buf = VulkanBuffer::new(
            instance,
            allocator,
            vertex_size,
            vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            name,
        );

        let index_buf = VulkanBuffer::new(
            instance,
            allocator,
            index_size,
            vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            name,
        );

        let mut staging_buf = VulkanBuffer::new(
            instance,
            allocator,
            vertex_size + index_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            name,
        );

        let memory = staging_buf.map(&instance.device, vertex_size + index_size);

        // copy vertex buffer
        staging_buf.upload_data(memory, vertices, vertex_size);
        // copy index buffer
        staging_buf.upload_data(
            memory.wrapping_add(vertex_size as usize),
            indices,
            index_size,
        );

        let command_buffer = begin_single_time_command(&instance.device, instance.command_pool);

        unsafe {
            let buffer_info = vk::BufferCopy::builder().size(vertex_size);

            instance.device.cmd_copy_buffer(
                command_buffer,
                *staging_buf,
                *vertex_buf,
                &[*buffer_info],
            );
            let buffer_info = vk::BufferCopy::builder()
                .size(index_size)
                .src_offset(vertex_size);

            instance.device.cmd_copy_buffer(
                command_buffer,
                *staging_buf,
                *index_buf,
                &[*buffer_info],
            );
        };
        staging_buf.unmap(&instance.device);

        end_single_time_command(
            &instance.device,
            instance.command_pool,
            instance.graphics_queue,
            command_buffer,
        );

        staging_buf.destroy(&instance.device);

        Self {
            vertex_buf,
            index_buf,
            index_count: indices.len() as u32,
            material,
            descriptor_set: None,
        }
    }

    pub fn set_descriptor_set(&mut self, descriptor_set: Vec<vk::DescriptorSet>) {
        self.descriptor_set = Some(descriptor_set);
    }

    pub fn bind(
        &self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        buffer_index: usize,
        pipeline_layout: vk::PipelineLayout,
        with_descriptor_set: bool,
    ) {
        unsafe {
            device.cmd_bind_vertex_buffers(command_buffer, 0, &[*self.vertex_buf], &[0]);
            device.cmd_bind_index_buffer(command_buffer, *self.index_buf, 0, vk::IndexType::UINT32);
            if with_descriptor_set {
                if let Some(ds) = &self.descriptor_set {
                    device.cmd_bind_descriptor_sets(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        pipeline_layout,
                        0,
                        &ds[buffer_index..=buffer_index],
                        &[],
                    )
                }
            }
        }
    }

    pub fn draw(&self, device: &ash::Device, command_buffer: vk::CommandBuffer) {
        unsafe { device.cmd_draw_indexed(command_buffer, self.index_count, 1, 0, 0, 0) };
    }

    pub fn destroy(&mut self, _descriptor_pool: vk::DescriptorPool, device: &ash::Device) {
        self.vertex_buf.destroy(device);
        self.index_buf.destroy(device);
        // We are getting an Validation error when we try to free an descriptor set, They will all automatily freed when the Descriptor pool is destroyed
        // if let Some(descriptor_set) = &mut self.descriptor_set {
        //     unsafe {
        //         device
        //             .free_descriptor_sets(descriptor_pool, descriptor_set)
        //             .expect("Failed to free Model descriptor sets")
        //     };
        // }
        if let Some(material) = &mut self.material {
            material.diffuse_texture.destroy(device);
        }
    }
}
