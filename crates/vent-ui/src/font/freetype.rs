use std::ffi::{CStr, CString};

use ash::vk::{self, Extent2D};
use vent_math::vec::{i32::ivec2::IVec2, vec2::Vec2};
use vent_rendering::{image::VulkanImage, instance::VulkanInstance};

use super::{Character, Font};

pub struct FreeTypeLoader {
    library: freetype::Library,
}

const CHARACTERS_SIZE: usize = 128;

impl FreeTypeLoader {
    pub fn new() -> Self {
        let library = freetype::Library::init().unwrap();
        Self { library }
    }

    // Loads an new Font
    pub fn load(&self, path: String, instance: &VulkanInstance) -> Font {
        let face = self.library.new_face(path, 0).unwrap();
        face.set_char_size(40 * 64, 0, 50, 0).unwrap();
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
            let texture = VulkanImage::new(
                instance,
                bitmap.buffer(),
                image_size,
                vk::Format::R32_SINT,
                instance.command_pool,
                &instance.memory_allocator,
                instance.graphics_queue,
                None,
            );
            // TODO, Use only one sampeler
            let character = Character { texture, size: IVec2::new(bitmap.width(), bitmap.rows()), bearing: IVec2::new(glyph.bitmap_left(), glyph.bitmap_top()), advance: glyph.advance().x as u32 };
            characters.push(character);
        }
        Font { characters }
    }
}
