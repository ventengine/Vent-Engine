use ash::vk;

// TODO Write an own Efficent Vulkan Memory Allocator

pub struct MemoryAllocator {
    memory_props: vk::PhysicalDeviceMemoryProperties,
}

impl MemoryAllocator {
    pub fn new(memory_props: vk::PhysicalDeviceMemoryProperties) -> Self {
        Self { memory_props }
    }

    /// Allocates memory for an Buffer and binds it
    pub fn allocate_buffer(
        &self,
        device: &ash::Device,
        buffer: vk::Buffer,
        flags: vk::MemoryPropertyFlags,
    ) -> vk::DeviceMemory {
        let memory_req = unsafe { device.get_buffer_memory_requirements(buffer) };

        let memory_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(memory_req.size)
            .memory_type_index(
                self.find_memorytype_index(memory_req, flags)
                    .expect("Failed to find Memory Index"),
            );

        let memory = unsafe { device.allocate_memory(&memory_info, None) }.unwrap();
        unsafe {
            device
                .bind_buffer_memory(buffer, memory, 0)
                .expect("Failed to bind Buffer memory");
        }
        memory
    }

    /// Allocates memory for an Image and binds it
    pub fn allocate_image(
        &self,
        device: &ash::Device,
        image: vk::Image,
        flags: vk::MemoryPropertyFlags,
    ) -> vk::DeviceMemory {
        let memory_req = unsafe { device.get_image_memory_requirements(image) };

        let memory_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(memory_req.size)
            .memory_type_index(
                self.find_memorytype_index(memory_req, flags)
                    .expect("Failed to find Memory Index"),
            );

        let memory = unsafe { device.allocate_memory(&memory_info, None) }.unwrap();
        unsafe {
            device
                .bind_image_memory(image, memory, 0)
                .expect("Failed to bind Buffer memory");
        }
        memory
    }

    fn find_memorytype_index(
        &self,
        memory_req: vk::MemoryRequirements,
        flags: vk::MemoryPropertyFlags,
    ) -> Option<u32> {
        self.memory_props.memory_types[..self.memory_props.memory_type_count as _]
            .iter()
            .enumerate()
            .find(|(index, memory_type)| {
                (1 << index) & memory_req.memory_type_bits != 0
                    && memory_type.property_flags & flags == flags
            })
            .map(|(index, _memory_type)| index as _)
    }
}
