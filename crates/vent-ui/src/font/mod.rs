use std::collections::HashMap;

use ash::vk;
use vent_math::vec::vec2::Vec2;
use vent_rendering::{buffer::VulkanBuffer, instance::VulkanInstance, Vertex2D};

pub mod ab_glyph;

pub struct Character {
    size: Vec2,
    bearing: Vec2,
    v_advance: f32,
}

#[allow(dead_code)]
pub struct Font {
    buffer_cache: HashMap<String, (u32, VulkanBuffer)>,
    font_atlas: Vec<vk::DescriptorSet>,
    atlas_width: u32,
    atlas_height: u32,
    characters: Vec<Character>,
}

impl Font {
    // Bind Pipeline before
    #[allow(clippy::too_many_arguments)]
    pub fn render_text(
        &mut self,
        instance: &VulkanInstance,
        command_buffer: vk::CommandBuffer,
        pipeline_layout: vk::PipelineLayout,
        buffer_index: usize,
        text: String,
        x: f32,
        y: f32,
        scale: f32,
        color: u32,
    ) {
        unsafe {
            instance.device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline_layout,
                0,
                &self.font_atlas[buffer_index..=buffer_index],
                &[],
            )
        };

        let characters = &self.characters;

        if !self.buffer_cache.contains_key(&text) {
            let mut batched_vertices = Vec::new();
            // Loop through each character in the text
            let mut current_x = x;
            for character in text.chars() {
                let character_index = character as usize;

                // Check if the character is within the loaded characters
                if character_index < characters.len() {
                    let character = &characters[character_index];

                    let xpos = current_x + character.bearing.x * scale;
                    let ypos = y - (character.size.y - character.bearing.y) * scale;
                    let width = character.size.x * scale;
                    let height = character.size.y * scale;

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
                    for vertex in vertices {
                        batched_vertices.push(vertex);
                    }
                    current_x += character.v_advance * scale;
                } else {
                    log::warn!("Text Character is too big {}", character_index)
                }
            }
            Self::create_buffer(
                &mut self.buffer_cache,
                instance,
                text.clone(),
                &batched_vertices,
            );
        }
        let (vertex_count, buffer) = &self.buffer_cache[&text];

        let render_area = vk::Rect2D::default()
            .offset(vk::Offset2D::default())
            .extent(instance.surface_resolution);

        unsafe {
            instance
                .device
                .cmd_set_scissor(command_buffer, 0, &[render_area]);
            instance.device.cmd_bind_vertex_buffers2(
                command_buffer,
                0,
                &[**buffer],
                &[0],
                None,
                None,
            );
            instance
                .device
                .cmd_draw(command_buffer, *vertex_count, 1, 0, 0)
        }

        // Update offset for the next character
    }

    fn create_buffer(
        buffer_cache: &mut HashMap<String, (u32, VulkanBuffer)>,
        instance: &VulkanInstance,
        text: String,
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
        buffer_cache.insert(text, (vertices.len() as u32, vulkan_buffer));
    }

    pub fn destroy(&mut self, device: &ash::Device) {
        for mut buffer in self.buffer_cache.drain() {
            buffer.1 .1.destroy(device);
        }
    }
}
