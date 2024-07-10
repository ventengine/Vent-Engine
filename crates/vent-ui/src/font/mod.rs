use std::collections::HashMap;

use ash::vk;
use vent_math::vec::i32::ivec2::IVec2;
use vent_rendering::{buffer::VulkanBuffer, instance::VulkanInstance, Vertex2D};

pub mod cosmic;
pub mod freetype;

pub struct Character {
    descriptor_sets: Vec<vk::DescriptorSet>, // Image
    size: IVec2,
    bearing: IVec2,
    advance: u32,
}

pub struct Font {
    buffer_cache: HashMap<usize, VulkanBuffer>,
    characters: Vec<Character>,
}

impl Default for Font {
    fn default() -> Self {
        Self::new()
    }
}

impl Font {
    pub fn new() -> Self {
        Self {
            buffer_cache: HashMap::new(),
            characters: Vec::new(),
        }
    }

    // Bind Pipeline before
    #[allow(clippy::too_many_arguments)]
    pub fn render_text(
        &mut self,
        instance: &VulkanInstance,
        command_buffer: vk::CommandBuffer,
        pipeline_layout: vk::PipelineLayout,
        buffer_index: usize,
        text: &str,
        x: f32,
        y: f32,
        scale: f32,
        color: u32,
    ) {
        let mut offset_x = x;
        let characters = &self.characters;

        // Loop through each character in the text
        for character in text.chars() {
            let character_index = character as usize;

            // Check if the character is within the loaded characters
            if character_index < characters.len() {
                let character = &characters[character_index];

                // Calculate vertex positions and texture coordinates based on scale
                let xpos = offset_x + character.bearing.x as f32 * scale;
                let ypos = y - (character.size.y - character.bearing.y) as f32 * scale;
                let width = character.size.x as f32 * scale;
                let height = character.size.y as f32 * scale;

                // Check if buffer contains in the cache, When not create a new vertex buffer
                if !self.buffer_cache.contains_key(&character_index) {
                    let color = [color as f32, color as f32, color as f32, 1.0];
                    let vertices: [Vertex2D; 6] = [
                        Vertex2D {
                            position: [xpos, ypos + height],
                            tex_coord: [0.0, 0.0],
                            color,
                        },
                        Vertex2D {
                            position: [xpos, ypos],
                            tex_coord: [0.0, 1.0],
                            color,
                        },
                        Vertex2D {
                            position: [xpos + width, ypos],
                            tex_coord: [1.0, 1.0],
                            color,
                        },
                        Vertex2D {
                            position: [xpos, ypos + height],
                            tex_coord: [0.0, 0.0],
                            color,
                        },
                        Vertex2D {
                            position: [xpos + width, ypos],
                            tex_coord: [1.0, 1.0],
                            color,
                        },
                        Vertex2D {
                            position: [xpos + width, ypos + height],
                            tex_coord: [1.0, 0.0],
                            color,
                        },
                    ];
                    Self::create_buffer(
                        &mut self.buffer_cache,
                        instance,
                        character_index,
                        &vertices,
                    );
                }
                let buffer = &self.buffer_cache[&character_index];

                let render_area = vk::Rect2D::default()
                    .offset(vk::Offset2D::default())
                    .extent(instance.surface_resolution);

                unsafe {
                    instance
                        .device
                        .cmd_set_scissor(command_buffer, 0, &[render_area]);
                    instance.device.cmd_bind_descriptor_sets(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        pipeline_layout,
                        0,
                        &character.descriptor_sets[buffer_index..=buffer_index],
                        &[],
                    );
                    instance.device.cmd_bind_vertex_buffers2(
                        command_buffer,
                        0,
                        &[**buffer],
                        &[0],
                        None,
                        None,
                    );
                    instance.device.cmd_draw(command_buffer, 6, 1, 0, 0)
                }

                // Update offset for the next character
                offset_x += (character.advance >> 6) as f32 * scale;
            }
        }
    }

    fn create_buffer(
        buffer_cache: &mut HashMap<usize, VulkanBuffer>,
        instance: &VulkanInstance,
        index: usize,
        vertices: &[Vertex2D],
    ) {
        let vertex_size = std::mem::size_of_val(vertices) as vk::DeviceSize;

        let vulkan_buffer = VulkanBuffer::cpu_to_gpu(
            instance,
            vertices,
            vertex_size,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            Some("Font Buffer"),
        );
        buffer_cache.insert(index, vulkan_buffer);
    }
}
