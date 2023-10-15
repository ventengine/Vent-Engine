use std::ptr::copy_nonoverlapping;

use ash::vk;

use crate::allocator::MemoryAllocator;

pub struct VulkanBuffer {
    pub buffer: vk::Buffer,
    pub buffer_memory: vk::DeviceMemory,
}

impl VulkanBuffer {
    // Size = (size_of::<Vertex>() * SIZE)
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

        Self {
            buffer,
            buffer_memory,
        }
    }

    pub fn new_init(
        device: &ash::Device,
        allocator: &MemoryAllocator,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        data: &[u8],
    ) -> Self {
        let buffer = Self::new(device, allocator, size, usage);
        buffer.upload_data(device, data, size);
        buffer
    }

    pub fn upload_data(&self, device: &ash::Device, data: &[u8], size: vk::DeviceSize) {
        unsafe {
            let memory = device
                .map_memory(self.buffer_memory, 0, size, vk::MemoryMapFlags::empty())
                .unwrap();
            copy_nonoverlapping(data.as_ptr(), memory.cast(), data.len());
            device.unmap_memory(self.buffer_memory);
        }
    }

    pub fn destroy(self, device: &ash::Device) {
        unsafe {
            device.destroy_buffer(self.buffer, None);
            device.free_memory(self.buffer_memory, None);
        }
    }
}

impl ::std::ops::Deref for VulkanBuffer {
    type Target = vk::Buffer;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}
