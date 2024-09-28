use std::{collections::HashMap, path::Path};

use ab_glyph::{point, Font, FontVec, Glyph, PxScale, ScaleFont};
use ash::vk::{self};
use image::{DynamicImage, Rgba};
use vent_math::vec::vec2::Vec2;
use vent_rendering::{image::VulkanImage, instance::VulkanInstance};

pub struct AbGlyphLoader {}

pub const CHARACTERS_SIZE: u32 = 256; // UTF-8

impl AbGlyphLoader {
    // Loads an new Font
    pub fn load<P>(
        path: P,
        descriptor_set_layout: vk::DescriptorSetLayout,
        descriptor_pool: vk::DescriptorPool,
        instance: &mut VulkanInstance,
    ) -> super::Font
    where
        P: AsRef<Path>,
    {
        log::debug!(target: "ui","Loading new Font using Ab-Glyph");
        let font = FontVec::try_from_vec(std::fs::read(path).unwrap()).unwrap();
        let scale = PxScale::from(48.0);
        let scaled_font = font.as_scaled(scale);

        let mut characters = Vec::with_capacity(CHARACTERS_SIZE as usize);
        let mut glyphs = Vec::new();
        let position = point(20.0, 20.0);
        let max_width = 9999_f32;

        let v_advance = scaled_font.height() + scaled_font.line_gap();
        let mut caret = position + point(0.0, scaled_font.ascent());
        let mut last_glyph: Option<Glyph> = None;
        for charr in 0..CHARACTERS_SIZE {
            let char = char::from_u32(charr).unwrap();

            if char.is_control() {
                if char.is_whitespace() {
                    caret = point(position.x, caret.y + v_advance);
                    last_glyph = None;
                }
                continue;
            }
            let mut glyph = scaled_font.scaled_glyph(char);
            if let Some(previous) = last_glyph.take() {
                caret.x += scaled_font.kern(previous.id, glyph.id);
            }
            glyph.position = caret;

            last_glyph = Some(glyph.clone());
            caret.x += scaled_font.h_advance(glyph.id);

            if !char.is_whitespace() && caret.x > position.x + max_width {
                caret = point(position.x, caret.y + v_advance);
                glyph.position = caret;
                last_glyph = None;
            }

            let character = Character {
                size: Vec2::new(scaled_font.h_advance(glyph.id), v_advance),
                bearing: Vec2::new(
                    scaled_font.h_side_bearing(glyph.id),
                    scaled_font.v_side_bearing(glyph.id),
                ),
                v_advance,
            };
            glyphs.push(glyph);
            characters.push(character);
        }
        log::debug!("Loaded Charaters: {}", characters.len());

        let outlined: Vec<_> = glyphs
            .into_iter()
            // Note: not all layout glyphs have outlines (e.g. " ")
            .filter_map(|g| font.outline_glyph(g))
            .collect();

        let Some(all_px_bounds) = outlined
            .iter()
            .map(|g| g.px_bounds())
            .reduce(|mut b, next| {
                b.min.x = b.min.x.min(next.min.x);
                b.max.x = b.max.x.max(next.max.x);
                b.min.y = b.min.y.min(next.min.y);
                b.max.y = b.max.y.max(next.max.y);
                b
            })
        else {
            panic!("No outlined glyphs?");
        };
        let mut image =
            DynamicImage::new_rgba8(all_px_bounds.width() as _, all_px_bounds.height() as _)
                .to_rgba8();
        const COLOUR: (u8, u8, u8) = (255, 255, 255);
        // Loop through the glyphs in the text, positing each one on a line
        for glyph in outlined {
            let bounds = glyph.px_bounds();
            // calc top/left ords in "image space"
            // image-x=0 means the *left most pixel*, equivalent to
            // px_bounds.min.x which *may be non-zero* (and similarly with y)
            // so `- px_bounds.min` converts the left-most/top-most to 0
            let img_left = bounds.min.x as u32 - all_px_bounds.min.x as u32;
            let img_top = bounds.min.y as u32 - all_px_bounds.min.y as u32;
            // Draw the glyph into the image per-pixel by using the draw closure
            glyph.draw(|x, y, v| {
                // Offset the position by the glyph bounding box
                let px = image.get_pixel_mut(img_left + x, img_top + y);
                // Turn the coverage into an alpha value (blended with any previous)
                *px = Rgba([
                    COLOUR.0,
                    COLOUR.1,
                    COLOUR.2,
                    px.0[3].saturating_add((v * 255.0) as u8),
                ]);
            });
        }
        // image.save("test.png").unwrap();

        // let image_size = Extent2D {
        //     width: all_px_bounds.width() as u32,
        //     height: all_px_bounds.height() as u32,
        // };

        let dimensions = image.dimensions();
        let sampler = vk::SamplerCreateInfo::default()
            .max_lod(1.0);
        let texture =
            VulkanImage::from_image(instance, DynamicImage::ImageRgba8(image), false, Some(sampler), Some("Font Atlas"));

        // TODO: store everything in an Texture Atlas
        let descriptor_sets = VulkanInstance::allocate_descriptor_sets(
            &instance.device,
            descriptor_pool,
            descriptor_set_layout,
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
        super::Font {
            font_atlas: descriptor_sets,
            characters,
            font_texture: texture,
            buffer_cache: HashMap::new(),
            atlas_width: dimensions.0,
            atlas_height: dimensions.1,
            scale_in_pixels: 1
        }
    }
}
