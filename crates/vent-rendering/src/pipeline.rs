use std::{ffi::CStr, fs::File};

use ash::{util::read_spv, vk};

use crate::{instance::VulkanInstance, Vertex, Vertex3D};

pub fn create_pipeline(
    instance: VulkanInstance,
    vertex_file: String,
    fragment_file: String,
    surface_resolution: vk::Extent2D,
) -> vk::Pipeline {
    let vertex_code = read_spv(&mut File::open(vertex_file).unwrap()).unwrap();
    let vertex_module_info = vk::ShaderModuleCreateInfo::builder()
        .code(&vertex_code)
        .build();
    let fragment_code = read_spv(&mut File::open(fragment_file).unwrap()).unwrap();
    let fragment_module_info = vk::ShaderModuleCreateInfo::builder()
        .code(&fragment_code)
        .build();

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
        .vertex_attribute_descriptions(&Vertex3D::create_input_descriptions())
        .vertex_binding_descriptions(&[Vertex3D::BINDING_DESCRIPTION])
        .build();
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
        .viewports(&viewports)
        .build();

    let rasterization_info = vk::PipelineRasterizationStateCreateInfo {
        front_face: vk::FrontFace::COUNTER_CLOCKWISE,
        line_width: 1.0,
        polygon_mode: vk::PolygonMode::FILL,
        ..Default::default()
    };
    let multisample_state_info = vk::PipelineMultisampleStateCreateInfo {
        rasterization_samples: vk::SampleCountFlags::TYPE_1,
        ..Default::default()
    };
    let noop_stencil_state = vk::StencilOpState {
        fail_op: vk::StencilOp::KEEP,
        pass_op: vk::StencilOp::KEEP,
        depth_fail_op: vk::StencilOp::KEEP,
        compare_op: vk::CompareOp::ALWAYS,
        ..Default::default()
    };
    let depth_state_info = vk::PipelineDepthStencilStateCreateInfo {
        depth_test_enable: 1,
        depth_write_enable: 1,
        depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
        front: noop_stencil_state,
        back: noop_stencil_state,
        max_depth_bounds: 1.0,
        ..Default::default()
    };
    let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
        blend_enable: 0,
        src_color_blend_factor: vk::BlendFactor::SRC_COLOR,
        dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_DST_COLOR,
        color_blend_op: vk::BlendOp::ADD,
        src_alpha_blend_factor: vk::BlendFactor::ZERO,
        dst_alpha_blend_factor: vk::BlendFactor::ZERO,
        alpha_blend_op: vk::BlendOp::ADD,
        color_write_mask: vk::ColorComponentFlags::RGBA,
    }];
    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op(vk::LogicOp::CLEAR)
        .attachments(&color_blend_attachment_states)
        .build();

    let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::builder()
        .dynamic_states(&dynamic_state)
        .build();

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
        .layout(instance.pipeline_layout)
        .render_pass(instance.render_pass)
        .build();

    let graphics_pipelines = unsafe {
        instance.device.create_graphics_pipelines(
            vk::PipelineCache::null(),
            &[graphic_pipeline_info],
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
