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
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .build();

        let buffer = unsafe { instance.device.create_buffer(&buffer_info, None) }.unwrap();

        let requirements = unsafe { instance.device.get_buffer_memory_requirements(buffer) };

        let buffer_memory = allocator.allocate(
            &instance.device,
            requirements,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        );

        unsafe {
            instance
                .device
                .bind_buffer_memory(buffer, buffer_memory, 0)
                .expect("Failed to bind Buffer memory");
        }

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
        let requirements = unsafe { device.get_image_memory_requirements(image) };

        let memory =
            allocator.allocate(device, requirements, vk::MemoryPropertyFlags::DEVICE_LOCAL);
        unsafe {
            device
                .bind_image_memory(image, memory, 0)
                .expect("Unable to bind Image memory")
        };
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

    pub fn new_init_type<T>(
        instance: &VulkanInstance,
        allocator: &MemoryAllocator,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        data: *const T,
        name: Option<&str>,
    ) -> Self {
        let buffer = Self::new(instance, allocator, size, usage, name);
        buffer.upload_type(&instance.device, data, size);
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

    pub fn upload_type<T>(&self, device: &ash::Device, data: *const T, size: vk::DeviceSize) {
        unsafe {
            let memory = device
                .map_memory(self.buffer_memory, 0, size, vk::MemoryMapFlags::empty())
                .unwrap();
            let mut align = ash::util::Align::new(memory, align_of::<T>() as _, size);
            align.copy_from_slice(&[data]);
            device.unmap_memory(self.buffer_memory);
        }
    }

    pub fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            device.free_memory(self.buffer_memory, None);
            device.destroy_buffer(self.buffer, None);
        }
    }
}

impl ::std::ops::Deref for VulkanBuffer {
    type Target = vk::Buffer;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}
