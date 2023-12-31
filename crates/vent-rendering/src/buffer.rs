use std::mem::align_of;

use ash::vk;

use crate::allocator::MemoryAllocator;

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
        device: &ash::Device,
        allocator: &MemoryAllocator,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
    ) -> Self {
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .build();

        let buffer = unsafe { device.create_buffer(&buffer_info, None) }.unwrap();

        let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

        let buffer_memory = allocator.allocate(
            device,
            requirements,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        );

        unsafe {
            device
                .bind_buffer_memory(buffer, buffer_memory, 0)
                .expect("Failed to bind Buffer memory");
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
        device: &ash::Device,
        allocator: &MemoryAllocator,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        data: &[T],
    ) -> Self {
        let buffer = Self::new(device, allocator, size, usage);
        buffer.upload_data(device, data, size);
        buffer
    }

    pub fn new_init_type<T>(
        device: &ash::Device,
        allocator: &MemoryAllocator,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        data: *const T,
    ) -> Self {
        let buffer = Self::new(device, allocator, size, usage);
        buffer.upload_type(device, data, size);
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
            let mut align = ash::util::Align::new(memory, align_of::<f32>() as _, size);
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
