use std::{mem::size_of, path::PathBuf};

use ash::vk::{self};
use vent_assets::io::file::FileAsset;
use vent_math::vec::vec2::Vec2;
use vent_rendering::{
    any_as_u8_slice, instance::VulkanInstance, pipeline::VulkanPipeline, vertex::Vertex2D,
};

use crate::font::{ab_glyph::AbGlyphLoader, Font};

use super::GUI;

#[allow(dead_code)]
pub struct GuiRenderer {
    descriptor_pool: vk::DescriptorPool,
    pipeline: VulkanPipeline,

    // Font
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

        let pipeline = Self::create_pipeline(instance);

        let push_constant = PushConstant {
            scale: Vec2::ONE,
            translate: Vec2::ZERO,
        };

        let descriptor_pool = Self::create_descriptor_pool(
            1, // 1 Font
            instance.swapchain_images.len() as u32,
            &instance.device,
        );

        let mut renderer = Self {
            descriptor_pool,
            pipeline,
            push_constant,
            font: None,
            guis: Vec::new(),
        };
        let path = FileAsset::new("assets/fonts/Arial.ttf");
        // Load default font
        renderer.load_font(instance, path.root_path());
        renderer
    }

    fn create_pipeline(instance: &VulkanInstance) -> VulkanPipeline {
        let vertex_shader = FileAsset::new("assets/shaders/app/2D/gui.vert.spv");
        let fragment_shader = FileAsset::new("assets/shaders/app/2D/gui.frag.spv");

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

        let push_constant_range = vk::PushConstantRange::default()
            .size(size_of::<PushConstant>() as u32)
            .stage_flags(vk::ShaderStageFlags::VERTEX);

        VulkanPipeline::create_simple_pipeline(
            instance,
            vertex_shader.root_path(),
            fragment_shader.root_path(),
            &[Vertex2D::binding_description()],
            &Vertex2D::input_descriptions(),
            instance.surface_resolution,
            &[push_constant_range],
            &desc_layout_bindings,
        )
    }

    pub fn create_descriptor_pool(
        material_count: u32,
        swapchain_count: u32,
        device: &ash::Device,
    ) -> vk::DescriptorPool {
        let pool_sizes = [vk::DescriptorPoolSize {
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: material_count * swapchain_count,
        }];

        let create_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(material_count * swapchain_count);

        unsafe { device.create_descriptor_pool(&create_info, None) }.unwrap()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render_text(
        &mut self,
        instance: &VulkanInstance,
        command_buffer: vk::CommandBuffer,
        buffer_index: usize,
        text: String,
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
            let p_scale = Vec2::new(2.0, 2.0);
            let translate = Vec2::new(-1.0 - 0.0 * p_scale.x, -1.0 - 0.0 * p_scale.y);

            self.push_constant = PushConstant {
                scale: p_scale,
                translate,
            };

            unsafe {
                instance.device.cmd_push_constants(
                    command_buffer,
                    self.pipeline.pipeline_layout,
                    vk::ShaderStageFlags::VERTEX,
                    0,
                    any_as_u8_slice(&self.push_constant),
                )
            }
            font.render_text(
                instance,
                command_buffer,
                self.pipeline.pipeline_layout,
                buffer_index,
                text,
                x,
                y,
                scale,
                color,
            );
        }
    }

    pub fn load_font(&mut self, instance: &mut VulkanInstance, path: &PathBuf) {
        self.font = Some(AbGlyphLoader::load(
            path,
            self.pipeline.descriptor_set_layout,
            self.descriptor_pool,
            instance,
        ));
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

    pub fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            if let Some(font) = &mut self.font {
                font.destroy(device);
                self.font = None
            }
            self.pipeline.destroy(device);
            device.destroy_descriptor_pool(self.descriptor_pool, None);
        }
    }
}
