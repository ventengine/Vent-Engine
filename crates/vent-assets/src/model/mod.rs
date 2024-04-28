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
mod optimizer;

#[derive(Debug)]
pub enum ModelError {
    UnsupportedFormat,
    FileNotExists,
    LoadingError(String),
}

impl Model3D {
    #[inline]
    pub async fn load<P: AsRef<Path>>(
        instance: &VulkanInstance,
        vertex_shader: P,
        fragment_shader: P,
        pipeline_layout: vk::PipelineLayout,
        path: P,
    ) -> Self {
        let sw = Stopwatch::new_and_start();
        log::info!("Loading new Model...");
        let model = load_model_from_path(
            instance,
            vertex_shader.as_ref(),
            fragment_shader.as_ref(),
            pipeline_layout,
            path.as_ref(),
        )
        .await
        .expect("Failed to Load 3D Model");
        log::info!(
            "Model {} took {}ms to Load, {} Pipelines",
            path.as_ref().to_str().unwrap(),
            sw.elapsed_ms(),
            model.pipelines.len(), // TODO
        );
        model
    }

    /// So your ideal render loop would be

    /// For each pipeline
    ///  Set pipeline
    ///   For each material that uses pipeline
    ///      Set material bind group
    ///       For each primitive that uses material with pipeline
    ///        Draw primitive
    pub fn draw(
        &self,
        device: &ash::Device,
        pipeline_layout: vk::PipelineLayout,
        command_buffer: vk::CommandBuffer,
        buffer_index: usize,
        with_descriptor_set: bool,
    ) {
        self.pipelines.iter().for_each(|pipeline| {
            unsafe {
                device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    pipeline.pipeline,
                )
            }
            pipeline.materials.iter().for_each(|material| {
                if with_descriptor_set {
                    if let Some(ds) = &material.descriptor_set {
                        unsafe {
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
                material.meshes.iter().for_each(|mesh| {
                    // rpass.push_debug_group("Bind Mesh");
                    mesh.bind(device, command_buffer);
                    // rpass.pop_debug_group();
                    // rpass.insert_debug_marker("Draw!");
                    mesh.draw(device, command_buffer);
                });
            });
        })
    }

    pub fn destroy(&mut self, instance: &VulkanInstance) {
        self.pipelines.drain(..).for_each(|mut pipline| {
            unsafe { instance.device.destroy_pipeline(pipline.pipeline, None) };
            pipline.materials.drain(..).for_each(|mut model_material| {
                model_material
                    .material
                    .diffuse_texture
                    .destroy(&instance.device);
                model_material.meshes.drain(..).for_each(|mut mesh| {
                    mesh.destroy(&instance.device);
                });
                // We are getting an Validation error when we try to free an descriptor set, They will all automatily freed when the Descriptor pool is destroyed
            });
        });
    }
}

async fn load_model_from_path(
    instance: &VulkanInstance,
    vertex_shader: &Path,
    fragment_shader: &Path,
    pipline_layout: vk::PipelineLayout,
    path: &Path,
) -> Result<Model3D, ModelError> {
    if !path.exists() {
        return Err(ModelError::FileNotExists);
    }

    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

    // Very Pretty, I know
    match extension {
        "obj" => Ok(OBJLoader::load(instance, path).await?),
        "gltf" => Ok(GLTFLoader::load(
            instance,
            vertex_shader,
            fragment_shader,
            pipline_layout,
            path,
        )
        .await?),
        _ => Err(ModelError::UnsupportedFormat),
    }
}

impl Mesh3D {
    pub fn new(
        instance: &VulkanInstance,
        allocator: &MemoryAllocator,
        vertices: &[Vertex3D],
        indices: &[u32],
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
        unsafe { staging_buf.upload_data(memory, vertices, vertex_size) };
        // copy index buffer
        unsafe {
            staging_buf.upload_data(
                memory.wrapping_add(vertex_size as usize),
                indices,
                index_size,
            )
        };

        let command_buffer = begin_single_time_command(&instance.device, instance.command_pool);

        unsafe {
            let buffer_info = vk::BufferCopy::default().size(vertex_size);

            instance.device.cmd_copy_buffer(
                command_buffer,
                *staging_buf,
                *vertex_buf,
                &[buffer_info],
            );
            let buffer_info = vk::BufferCopy::default()
                .size(index_size)
                .src_offset(vertex_size);

            instance.device.cmd_copy_buffer(
                command_buffer,
                *staging_buf,
                *index_buf,
                &[buffer_info],
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
        }
    }

    pub fn bind(&self, device: &ash::Device, command_buffer: vk::CommandBuffer) {
        unsafe {
            device.cmd_bind_vertex_buffers(command_buffer, 0, &[*self.vertex_buf], &[0]);
            device.cmd_bind_index_buffer(command_buffer, *self.index_buf, 0, vk::IndexType::UINT32);
        }
    }

    pub fn draw(&self, device: &ash::Device, command_buffer: vk::CommandBuffer) {
        unsafe { device.cmd_draw_indexed(command_buffer, self.index_count, 1, 0, 0, 0) };
    }

    pub fn destroy(&mut self, device: &ash::Device) {
        self.vertex_buf.destroy(device);
        self.index_buf.destroy(device);
    }
}
