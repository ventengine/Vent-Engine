use std::{mem::align_of, os::raw::c_void};

use ash::vk;

use crate::{
    allocator::MemoryAllocator, begin_single_time_command, debug, end_single_time_command,
    instance::VulkanInstance,
};

pub struct VulkanBuffer {
    pub buffer: vk::Buffer,
    pub buffer_memory: vk::DeviceMemory,
}

impl VulkanBuffer {
    // Size = (size_of::<Vertex>() * SIZE)
    /**
     * Allocates & Binds new uninitialized Memory based on Size and Usage
     */
    pub fn new(
        instance: &VulkanInstance,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        flags: vk::MemoryPropertyFlags,
        name: Option<&str>,
    ) -> Self {
        let buffer_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe { instance.device.create_buffer(&buffer_info, None) }.unwrap();

        let buffer_memory =
            instance
                .memory_allocator
                .allocate_buffer(&instance.device, buffer, flags);

        #[cfg(debug_assertions)]
        if let Some(name) = name {
            debug::set_object_name(instance, buffer, name)
        }

        Self {
            buffer,
            buffer_memory,
        }
    }

    /**
     * Allocates & Binds an Staging buffer for CPU and then uploading it to the GPU
     */
    pub fn cpu_to_gpu<T: Copy>(
        instance: &VulkanInstance,
        data: &[T],
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        name: Option<&str>,
    ) -> Self {
        let buffer = VulkanBuffer::new(
            instance,
            size,
            usage,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            name,
        );
        let mut staging_buf = VulkanBuffer::new(
            instance,
            size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            name,
        );

        let memory = staging_buf.map(&instance.device, size);

        unsafe { staging_buf.upload_data(memory, data, size) };

        let command_buffer = begin_single_time_command(&instance.device, instance.command_pool);

        unsafe {
            let buffer_info = vk::BufferCopy::default().size(size);

            instance
                .device
                .cmd_copy_buffer(command_buffer, *staging_buf, *buffer, &[buffer_info]);
        }
        staging_buf.unmap(&instance.device);

        end_single_time_command(
            &instance.device,
            instance.command_pool,
            instance.graphics_queue,
            command_buffer,
        );

        staging_buf.destroy(&instance.device);
        buffer
    }

    /**
     * Allocates new Image memory & binds them
     */
    pub fn new_image(
        device: &ash::Device,
        allocator: &MemoryAllocator,
        image: vk::Image,
    ) -> vk::DeviceMemory {
        allocator.allocate_image(device, image, vk::MemoryPropertyFlags::DEVICE_LOCAL)
    }

    /**
     * Allocates & Binds new initialized Memory based on Size and Usage
     */
    pub fn new_init<T: Copy>(
        instance: &VulkanInstance,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        data: &[T],
        flags: vk::MemoryPropertyFlags,
        name: Option<&str>,
    ) -> Self {
        let buffer = Self::new(instance, size, usage, flags, name);
        let memory = buffer.map(&instance.device, size);
        unsafe { buffer.upload_data(memory, data, size) };
        buffer.unmap(&instance.device);
        buffer
    }

    /// # Safety
    ///
    /// Do not give an bad memory pointer
    pub unsafe fn upload_data<T: Copy>(
        &self,
        memory: *mut c_void,
        data: &[T],
        size: vk::DeviceSize,
    ) {
        let mut align = ash::util::Align::new(memory, align_of::<T>() as _, size);
        align.copy_from_slice(data);
    }

    pub fn map(&self, device: &ash::Device, size: vk::DeviceSize) -> *mut c_void {
        unsafe {
            device
                .map_memory(self.buffer_memory, 0, size, vk::MemoryMapFlags::empty())
                .unwrap()
        }
    }

    pub fn unmap(&self, device: &ash::Device) {
        unsafe { device.unmap_memory(self.buffer_memory) };
    }

    pub fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            device.destroy_buffer(self.buffer, None);
            device.free_memory(self.buffer_memory, None); // Free Memory after buffer destruction!
        }
    }
}

impl ::std::ops::Deref for VulkanBuffer {
    type Target = vk::Buffer;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}
