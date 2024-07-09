use std::mem::size_of;

use ash::vk::{self};
use vent_math::vec::vec2::Vec2;
use vent_rendering::{
    any_as_u8_slice, instance::VulkanInstance, pipeline::VulkanPipeline, Vertex2D,
};

use crate::font::{freetype::FreeTypeLoader, Font};

use super::GUI;

#[allow(dead_code)]
pub struct GuiRenderer {
    descriptor_set_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    pipeline: VulkanPipeline,

    // Font
    font_loader: FreeTypeLoader,
    push_constant: PushConstant,
    font: Option<Font>,

    guis: Vec<Box<dyn GUI>>,
}

#[repr(C)]
pub struct PushConstant {
    scale: Vec2,
    translate: Vec2,
}

impl GuiRenderer {
    pub const DEFAULT_TEXTURE_FILTER: vk::Filter = vk::Filter::LINEAR;

    pub fn new(instance: &mut VulkanInstance) -> Self {
        log::debug!(target: "ui", "initialising UI Renderer");

        // Create descripotr set layout
        let desc_layout_bindings = [
            // Fragment
            vk::DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            },
        ];
        let info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&desc_layout_bindings);
        let descriptor_set_layout =
            unsafe { instance.device.create_descriptor_set_layout(&info, None) }.unwrap();

        let push_constant_range = vk::PushConstantRange::default()
            .size(size_of::<PushConstant>() as u32)
            .stage_flags(vk::ShaderStageFlags::VERTEX);

        let constant_ranges = [push_constant_range];
        let binding = [descriptor_set_layout];
        let create_info = vk::PipelineLayoutCreateInfo::default()
            .push_constant_ranges(&constant_ranges)
            .set_layouts(&binding);

        let pipeline_layout =
            unsafe { instance.device.create_pipeline_layout(&create_info, None) }.unwrap();
        let pipeline = Self::create_pipeline(pipeline_layout, instance);

        let font_loader = FreeTypeLoader::new();

        let push_constant = PushConstant {
            scale: Vec2::ONE,
            translate: Vec2::ZERO,
        };

        let mut renderer = Self {
            descriptor_set_layout,
            pipeline_layout,
            pipeline,
            font_loader,
            push_constant,
            font: None,
            guis: Vec::new(),
        };
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fonts/Arial.ttf");
        // Load default font
        renderer.load_font(instance, path);
        renderer
    }

    fn create_pipeline(
        pipeline_layout: vk::PipelineLayout,
        instance: &VulkanInstance,
    ) -> VulkanPipeline {
        let vertex_shader = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/shaders/shader.vert.spv"
        );
        let fragment_shader = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/shaders/shader.frag.spv"
        );

        VulkanPipeline::create_simple_pipeline(
            instance,
            vertex_shader.as_ref(),
            fragment_shader.as_ref(),
            &[Vertex2D::binding_description()],
            pipeline_layout,
            &Vertex2D::input_descriptions(),
            instance.surface_resolution,
        )
    }

    pub fn render_text(
        &mut self,
        instance: &VulkanInstance,
        command_buffer: vk::CommandBuffer,
        buffer_index: usize,
        text: &str,
        x: f32,
        y: f32,
        scale: f32,
        color: u32,
    ) {
        if let Some(font) = &mut self.font {
            unsafe {
                instance.device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.pipeline.pipeline,
                )
            };
            let viewport = vk::Viewport::default()
                .width(instance.surface_resolution.width as f32)
                .height(instance.surface_resolution.height as f32)
                .max_depth(1.0);
            unsafe {
                instance
                    .device
                    .cmd_set_viewport(command_buffer, 0, &[viewport]);
            }
            let p_scale = Vec2::new(2.0 / x, 2.0 / y);
            let translate = Vec2::new(-1.0 - x * p_scale.x, -1.0 - y * p_scale.y);

            self.push_constant = PushConstant {
                scale: p_scale,
                translate,
            };

            unsafe {
                instance.device.cmd_push_constants(
                    command_buffer,
                    self.pipeline_layout,
                    vk::ShaderStageFlags::VERTEX,
                    0,
                    any_as_u8_slice(&self.push_constant),
                )
            }
            font.render_text(
                instance,
                command_buffer,
                self.pipeline_layout,
                buffer_index,
                text,
                x,
                y,
                scale,
                color,
            );
        }
    }

    pub fn load_font(&mut self, instance: &mut VulkanInstance, path: &str) {
        self.font = Some(
            self.font_loader
                .load(path, self.descriptor_set_layout, instance),
        );
    }

    pub fn render(&mut self) {
        for gui in self.guis.iter_mut() {
            gui.update()
        }
    }

    pub fn add_gui(mut self, gui: Box<dyn GUI>) -> Self {
        self.guis.push(gui);
        self
    }

    #[inline]
    #[allow(dead_code)]
    pub fn register_texture(&mut self) {}

    pub fn destroy(&mut self, instance: &VulkanInstance) {
        unsafe {
            self.pipeline.destroy(&instance.device);
            instance
                .device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            instance
                .device
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None);
        }
    }
}
