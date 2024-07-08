use std::{collections::HashMap, ffi::OsStr};

use ash::vk::{self};
use cosmic_text::{Attrs, FontSystem, Metrics};
use vent_rendering::instance::VulkanInstance;

use super::Font;

pub struct CosmicLoader {
    font_system: cosmic_text::FontSystem,
}

const CHARACTERS_SIZE: usize = 128;

impl Default for CosmicLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl CosmicLoader {
    pub fn new() -> Self {
        log::debug!(target: "ui", "initialising Cosmic-Text");
        let font_system = FontSystem::new();
        Self { font_system }
    }

    // Loads an new Font
    pub fn load<P>(&self, path: P, instance: &VulkanInstance, sampler: vk::Sampler) -> Font
    where
        P: AsRef<OsStr>,
    {
        log::debug!(target: "ui","Loading new Font using FreeType");
        const FONT_SIZE: f32 = 14.0;
        const LINE_HEIGHT: f32 = FONT_SIZE * 1.2;
        let metrics = Metrics::new(FONT_SIZE, LINE_HEIGHT);
        // unsafe { FT_Set_Pixel_Sizes(face, 0, 48) };
        // let mut buffer = Buffer::new(&mut self.font_system, metrics);

        // let mut buffer = buffer.borrow_with(&mut self.font_system);

        let width = 80.0;
        // The height is unbounded
        //        buffer.set_size(None, Some(48.0));

        let attrs = Attrs::new();

        let characters = Vec::with_capacity(CHARACTERS_SIZE);
        // for charr in 0..CHARACTERS_SIZE {
        //     let char = char::from_u32(charr as u32).unwrap();
        //     buffer.set_text(&char.to_string(), attrs, Shaping::Advanced);
        //     let image_size = Extent2D {
        //         width: bitmap.width() as u32,
        //         height: bitmap.rows() as u32,
        //     };

        //     dbg!(bitmap.buffer());
        //     // Reusing the sampler
        //     let texture = VulkanImage::new(
        //         instance,
        //         bitmap.buffer(),
        //         image_size,
        //         vk::Format::R8G8B8A8_UNORM,
        //         instance.command_pool,
        //         &instance.memory_allocator,
        //         instance.graphics_queue,
        //         Some(sampler),
        //     );

        //     let descriptor_sets = VulkanInstance::allocate_descriptor_sets(
        //         &instance.device,
        //         instance.descriptor_pool,
        //         instance.descriptor_set_layout,
        //         instance.swapchain_images.len(),
        //     );

        //     for &descriptor_set in descriptor_sets.iter() {
        //         let image_info = vk::DescriptorImageInfo::default()
        //             .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        //             .image_view(texture.image_view)
        //             .sampler(texture.sampler);

        //         let desc_sets = [vk::WriteDescriptorSet {
        //             dst_set: descriptor_set,
        //             dst_binding: 0, // From DescriptorSetLayoutBinding
        //             descriptor_count: 1,
        //             descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        //             p_image_info: &image_info,
        //             ..Default::default()
        //         }];
        //         unsafe {
        //             instance.device.update_descriptor_sets(&desc_sets, &[]);
        //         }
        //     }

        //     let character = Character {
        //         descriptor_sets,
        //         size: IVec2::new(bitmap.width(), bitmap.rows()),
        //         bearing: IVec2::new(glyph.bitmap_left(), glyph.bitmap_top()),
        //         advance: glyph.advance().x as u32,
        //     };
        //     characters.push(character);
        // }
        Font {
            characters,
            buffer_cache: HashMap::new(),
        }
    }
}
