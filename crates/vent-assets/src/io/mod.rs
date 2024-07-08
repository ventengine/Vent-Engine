use std::collections::HashMap;

use crate::image::Image;

mod file;

pub struct AssetsLoader {
    image_cache: HashMap<String, Image>,
}

impl AssetsLoader {
    pub fn new() -> Self {
        Self {
            image_cache: HashMap::new(),
        }
    }

    // pub fn get_image(&mut self, device: &ash::Device, path: String) -> vk::Sampler {
    //     let vk = info.to_vk();
    //     *self.sampler_cache.entry(info).or_insert({

    //         unsafe { device.create_sampler(&vk, None) }.unwrap()
    //     })
    // }

    pub fn destroy(&mut self, device: &ash::Device) {
        unsafe { for imageö in self.image_cache.drain() {} }
    }
}
