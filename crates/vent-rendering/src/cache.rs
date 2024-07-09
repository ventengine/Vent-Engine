use std::collections::HashMap;

use ash::vk;

use crate::SamplerInfo;

pub struct VulkanCache {
    sampler_cache: HashMap<SamplerInfo, vk::Sampler>,
}

impl Default for VulkanCache {
    fn default() -> Self {
        Self::new()
    }
}

impl VulkanCache {
    pub fn new() -> Self {
        Self {
            sampler_cache: HashMap::new(),
        }
    }

    pub fn get_sampler(&mut self, device: &ash::Device, info: SamplerInfo) -> vk::Sampler {
        let vk = info.to_vk();
        *self
            .sampler_cache
            .entry(info)
            .or_insert(unsafe { device.create_sampler(&vk, None) }.unwrap())
    }

    pub fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            for sampler in self.sampler_cache.drain() {
                device.destroy_sampler(sampler.1, None)
            }
        }
    }
}
