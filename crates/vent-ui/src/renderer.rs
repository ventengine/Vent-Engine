use std::mem::size_of;

use ash::vk::{self, Pipeline};
use vent_math::vec::vec2::Vec2;
use vent_rendering::{instance::VulkanInstance, Vertex2D};

use super::GUI;

#[allow(dead_code)]
pub struct GuiRenderer {
    descriptor_set_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,

    sampler: vk::Sampler,

    guis: Vec<Box<dyn GUI>>,
}

#[repr(C)]
pub struct PushConstant {
    scale: Vec2,
    translate: Vec2,
}

impl GuiRenderer {
    pub const DEFAULT_TEXTURE_FILTER: vk::Filter = vk::Filter::LINEAR;

    fn new(&mut self, instance: &VulkanInstance) -> Self {
        // Create sampler
        let sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(Self::DEFAULT_TEXTURE_FILTER)
            .min_filter(Self::DEFAULT_TEXTURE_FILTER)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR);
        let sampler = unsafe { instance.device.create_sampler(&sampler_info, None) }.unwrap();

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

        Self {
            descriptor_set_layout,
            pipeline_layout,
            pipeline,
            sampler,
            guis: Vec::new(),
        }
    }

    fn create_pipeline(pipeline_layout: vk::PipelineLayout, instance: &VulkanInstance) -> Pipeline {
        let vertex_shader = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/shaders/shader.vert.spv"
        );
        let fragment_shader = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/shaders/shader.frag.spv"
        );

        vent_rendering::pipeline::create_simple_pipeline(
            instance,
            vertex_shader.as_ref(),
            fragment_shader.as_ref(),
            &[Vertex2D::binding_description()],
            pipeline_layout,
            &Vertex2D::input_descriptions(),
            instance.surface_resolution,
        )
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
}
