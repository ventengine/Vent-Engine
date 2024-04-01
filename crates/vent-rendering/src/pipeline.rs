use std::{ffi::CStr, fs::File, path::Path};

use ash::{util::read_spv, vk};

use crate::instance::VulkanInstance;

///
/// Creates an Simple Pipeline from an Vertex & Fragment Shader
///
/// Depth: Enabled (LESS),
/// Cull: Enabled (BACK),
/// Front Face: CC,
/// Polygon Mode: Fill
///
pub fn create_simple_pipeline(
    instance: &VulkanInstance,
    vertex_file: &Path,
    fragment_file: &Path,
    binding_desc: &[vk::VertexInputBindingDescription],
    pipeline_layout: vk::PipelineLayout,
    attrib_desc: &[vk::VertexInputAttributeDescription],
    surface_resolution: vk::Extent2D,
) -> vk::Pipeline {
    let vertex_code = read_spv(
        &mut File::open(vertex_file.with_extension("spv")).expect("Failed to open Vertex File"),
    )
    .unwrap();
    let vertex_module_info = vk::ShaderModuleCreateInfo::builder().code(&vertex_code);
    let fragment_code = read_spv(
        &mut File::open(fragment_file.with_extension("spv")).expect("Failed to open Fragment File"),
    )
    .unwrap();
    let fragment_module_info = vk::ShaderModuleCreateInfo::builder().code(&fragment_code);

    let vertex_module = unsafe {
        instance
            .device
            .create_shader_module(&vertex_module_info, None)
    }
    .unwrap();
    let fragment_module = unsafe {
        instance
            .device
            .create_shader_module(&fragment_module_info, None)
    }
    .unwrap();

    let shader_entry_name = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };
    let shader_stage_create_info = [
        vk::PipelineShaderStageCreateInfo {
            module: vertex_module,
            p_name: shader_entry_name.as_ptr(),
            stage: vk::ShaderStageFlags::VERTEX,
            ..Default::default()
        },
        vk::PipelineShaderStageCreateInfo {
            module: fragment_module,
            p_name: shader_entry_name.as_ptr(),
            stage: vk::ShaderStageFlags::FRAGMENT,
            ..Default::default()
        },
    ];

    let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_attribute_descriptions(attrib_desc)
        .vertex_binding_descriptions(binding_desc);
    let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
        topology: vk::PrimitiveTopology::TRIANGLE_LIST,
        ..Default::default()
    };
    let viewports = [vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: surface_resolution.width as f32,
        height: surface_resolution.height as f32,
        min_depth: 0.0,
        max_depth: 1.0,
    }];
    let scissors = [surface_resolution.into()];
    let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
        .scissors(&scissors)
        .viewports(&viewports);

    let rasterization_info = vk::PipelineRasterizationStateCreateInfo {
        front_face: vk::FrontFace::COUNTER_CLOCKWISE,
        line_width: 1.0,
        polygon_mode: vk::PolygonMode::FILL,
        cull_mode: vk::CullModeFlags::BACK,
        ..Default::default()
    };
    let multisample_state_info = vk::PipelineMultisampleStateCreateInfo {
        rasterization_samples: vk::SampleCountFlags::TYPE_1,
        ..Default::default()
    };

    let depth_state_info = vk::PipelineDepthStencilStateCreateInfo::builder()
        .depth_test_enable(true)
        .depth_write_enable(true)
        .depth_compare_op(vk::CompareOp::LESS)
        .max_depth_bounds(1.0);
    let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
        color_write_mask: vk::ColorComponentFlags::RGBA,
        ..Default::default()
    }];
    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op(vk::LogicOp::COPY)
        .attachments(&color_blend_attachment_states);

    let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR]; // TODO
    let dynamic_state_info =
        vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_state);

    let graphic_pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(&shader_stage_create_info)
        .vertex_input_state(&vertex_input_state_info)
        .input_assembly_state(&vertex_input_assembly_state_info)
        .viewport_state(&viewport_state_info)
        .rasterization_state(&rasterization_info)
        .multisample_state(&multisample_state_info)
        .depth_stencil_state(&depth_state_info)
        .color_blend_state(&color_blend_state)
        .dynamic_state(&dynamic_state_info)
        .layout(pipeline_layout)
        .render_pass(instance.render_pass);

    let graphics_pipelines = unsafe {
        instance.device.create_graphics_pipelines(
            vk::PipelineCache::null(),
            &[*graphic_pipeline_info],
            None,
        )
    }
    .expect("Unable to create graphics pipeline");

    unsafe {
        instance.device.destroy_shader_module(vertex_module, None);
        instance.device.destroy_shader_module(fragment_module, None);
    }

    graphics_pipelines[0]
}
