use std::mem::align_of;

use ash::vk;

use crate::{allocator::MemoryAllocator, debug, instance::VulkanInstance};

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
        allocator: &MemoryAllocator,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        name: Option<&str>,
    ) -> Self {
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe { instance.device.create_buffer(&buffer_info, None) }.unwrap();

        let buffer_memory = allocator.allocate_buffer(
            &instance.device,
            buffer,
            vk::MemoryPropertyFlags::HOST_VISIBLE |vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );

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
     * Allocates new Image memory & binds them
     */
    pub fn new_image(
        device: &ash::Device,
        allocator: &MemoryAllocator,
        image: vk::Image,
    ) -> vk::DeviceMemory {
        let memory = allocator.allocate_image(device, image, vk::MemoryPropertyFlags::DEVICE_LOCAL);
        memory
    }

    /**
     * Allocates & Binds new initialized Memory based on Size and Usage
     */
    pub fn new_init<T: Copy>(
        instance: &VulkanInstance,
        allocator: &MemoryAllocator,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        data: &[T],
        name: Option<&str>,
    ) -> Self {
        let buffer = Self::new(instance, allocator, size, usage, name);
        buffer.upload_data(&instance.device, data, size);
        buffer
    }

    pub fn upload_data<T: Copy>(&self, device: &ash::Device, data: &[T], size: vk::DeviceSize) {
        unsafe {
            let memory = device
                .map_memory(self.buffer_memory, 0, size, vk::MemoryMapFlags::empty())
                .unwrap();
            let mut align = ash::util::Align::new(memory, align_of::<T>() as _, size);
            align.copy_from_slice(data);
            device.unmap_memory(self.buffer_memory);
        }
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
