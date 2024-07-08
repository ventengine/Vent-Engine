use std::{collections::HashMap, ffi::OsStr};

use ash::vk::{self, Extent2D};
use vent_math::vec::i32::ivec2::IVec2;
use vent_rendering::{image::VulkanImage, instance::VulkanInstance, SamplerInfo};

use super::{Character, Font};

pub struct FreeTypeLoader {
    library: freetype::Library,
}

const CHARACTERS_SIZE: usize = 30;

impl Default for FreeTypeLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl FreeTypeLoader {
    pub fn new() -> Self {
        log::debug!(target: "ui", "initialising FreeType");
        let library = freetype::Library::init().unwrap();
        Self { library }
    }

    // Loads an new Font
    pub fn load<P>(&self, path: P, instance: &mut VulkanInstance) -> Font
    where
        P: AsRef<OsStr>,
    {
        log::debug!(target: "ui","Loading new Font using FreeType");
        let face = self.library.new_face(path, 0).unwrap();
        face.set_pixel_sizes(0, 48).unwrap();
        // unsafe { FT_Set_Pixel_Sizes(face, 0, 48) };
        let mut characters = Vec::with_capacity(CHARACTERS_SIZE);
        for char in 0..CHARACTERS_SIZE {
            face.load_char(char, freetype::face::LoadFlag::RENDER)
                .unwrap();
            let glyph = face.glyph();
            let bitmap = glyph.bitmap();

            let image_size = Extent2D {
                width: bitmap.width() as u32,
                height: bitmap.rows() as u32,
            };

            // Reusing the sampler
            let texture = VulkanImage::new(
                instance,
                bitmap.buffer(),
                image_size,
                vk::Format::R8G8B8A8_UNORM,
                Some(SamplerInfo::default()),
            );

            let descriptor_sets = VulkanInstance::allocate_descriptor_sets(
                &instance.device,
                instance.descriptor_pool,
                instance.descriptor_set_layout,
                instance.swapchain_images.len(),
            );

            for &descriptor_set in descriptor_sets.iter() {
                let image_info = vk::DescriptorImageInfo::default()
                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .image_view(texture.image_view)
                    .sampler(texture.sampler);

                let desc_sets = [vk::WriteDescriptorSet {
                    dst_set: descriptor_set,
                    dst_binding: 0, // From DescriptorSetLayoutBinding
                    descriptor_count: 1,
                    descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                    p_image_info: &image_info,
                    ..Default::default()
                }];
                unsafe {
                    instance.device.update_descriptor_sets(&desc_sets, &[]);
                }
            }

            let character = Character {
                descriptor_sets,
                size: IVec2::new(bitmap.width(), bitmap.rows()),
                bearing: IVec2::new(glyph.bitmap_left(), glyph.bitmap_top()),
                advance: glyph.advance().x as u32,
            };
            characters.push(character);
        }
        Font {
            characters,
            buffer_cache: HashMap::new(),
        }
    }
}
