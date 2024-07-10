use ash::vk;

use crate::{
    begin_single_time_command, buffer::VulkanBuffer, end_single_time_command,
    instance::VulkanInstance, Indices, Vertex3D,
};

/// This is a simple mesh that consists of vertices and indices. It is useful when you need to hard-code 3D data into your application.

/// By using this simple mesh, you can easily define custom shapes or provide default objects for your application. It is particularly handy when you want to avoid loading external model files and instead directly embed the 3D data within your code.

/// Note that this simple mesh implementation does not support advanced features such as normal mapping, skeletal animation, or material properties. It serves as a basic foundation for representing 3D geometry and can be extended or customized according to your specific requirements.

pub struct Mesh3D {
    // Basic
    vertex_buf: VulkanBuffer,
    index_buf: VulkanBuffer,
    index_type: vk::IndexType,
    index_count: u32,
}

impl Mesh3D {
    pub fn new(
        instance: &VulkanInstance,
        vertices: &[Vertex3D],
        indices: Indices,
        name: Option<&str>,
    ) -> Self {
        let vertex_size = std::mem::size_of_val(vertices) as vk::DeviceSize;
        let index_size = std::mem::size_of_val(indices.get_slice()) as vk::DeviceSize;

        let vertex_buf = VulkanBuffer::new(
            instance,
            vertex_size,
            vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            name,
        );

        let index_buf = VulkanBuffer::new(
            instance,
            index_size,
            vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            name,
        );

        let mut staging_buf = VulkanBuffer::new(
            instance,
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
            indices.upload(
                &staging_buf,
                memory.wrapping_add(vertex_size as usize),
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
            index_type: indices.vk_type(),
            index_count: indices.len() as u32,
        }
    }

    pub fn bind(&self, device: &ash::Device, command_buffer: vk::CommandBuffer) {
        unsafe {
            device.cmd_bind_vertex_buffers2(
                command_buffer,
                0,
                &[*self.vertex_buf],
                &[0],
                None,
                None,
            );
            device.cmd_bind_index_buffer(command_buffer, *self.index_buf, 0, self.index_type);
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
