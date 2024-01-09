use ash::vk;

// TODO Write an own Efficent Vulkan Memory Allocator

pub struct MemoryAllocator {
    memory_props: vk::PhysicalDeviceMemoryProperties,
}

impl MemoryAllocator {
    pub fn new(memory_props: vk::PhysicalDeviceMemoryProperties) -> Self {
        Self { memory_props }
    }

    pub fn allocate(
        &self,
        device: &ash::Device,
        memory_req: vk::MemoryRequirements,
        flags: vk::MemoryPropertyFlags,
    ) -> vk::DeviceMemory {
        let memory_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(memory_req.size)
            .memory_type_index(
                self.find_memorytype_index(memory_req, flags)
                    .expect("Failed to find Memory Index"),
            );

        unsafe { device.allocate_memory(&memory_info, None) }.unwrap()
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
