use ash::vk;
use vent_math::vec::{i32::ivec2::IVec2, vec3::Vec3};
use vent_rendering::{image::VulkanImage, instance::VulkanInstance};

mod freetype;

pub struct Character {
    texture: VulkanImage,
    size: IVec2,
    bearing: IVec2,
    advance: u32,
}

pub struct Font {
    characters: Vec<Character>,
}

impl Font {
    pub fn render_text(
        instance: &VulkanInstance,
        command_buffer: vk::CommandBuffer,
        pipeline_layout: vk::PipelineLayout,
        text: &str,
        x: i32,
        y: i32,
        scale: i32,
        color: u32,
        font: Font,
    ) {
        // for char in text.chars().enumerate() {
        //     let character = &font.characters[char.0];

        //     let xpos = x + character.bearing.x * scale;
        //     let ypos = y - (character.size.y - character.bearing.y) * scale;

        //     let w = character.size.x * scale;
        //     let h = character.size.y * scale;

        //     let vertices: [Vertex; 6] = [
        //         Vertex { x: xpos, y: ypos + h, u: 0.0, v: 0.0 },
        //         Vertex { x: xpos, y: ypos, u: 0.0, v: 1.0 },
        //         Vertex { x: xpos + w, y: ypos, u: 1.0, v: 1.0 },
        //         Vertex { x: xpos, y: ypos + h, u: 0.0, v: 0.0 },
        //         Vertex { x: xpos + w, y: ypos, u: 1.0, v: 1.0 },
        //         Vertex { x: xpos + w, y: ypos + h, u: 1.0, v: 0.0 },
        //     ];
    
        // }
    }
}
