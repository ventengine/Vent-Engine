use std::collections::HashMap;

use crate::image::Image;

mod file;

pub struct AssetsLoader {
    _image_cache: HashMap<String, Image>,
}

impl Default for AssetsLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetsLoader {
    pub fn new() -> Self {
        Self {
            _image_cache: HashMap::new(),
        }
    }

    // pub fn get_image(&mut self, device: &ash::Device, path: String) -> vk::Sampler {
    //     let vk = info.to_vk();
    //     *self.sampler_cache.entry(info).or_insert({

    //         unsafe { device.create_sampler(&vk, None) }.unwrap()
    //     })
    // }
}
