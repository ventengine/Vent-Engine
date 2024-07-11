use std::{ffi::CStr, fs::File, path::Path};

use ash::{util::read_spv, vk};

use crate::instance::VulkanInstance;

pub struct VulkanPipeline {
    pub pipeline: vk::Pipeline,
    pub vertex_module: vk::ShaderModule,
    pub fragment_module: vk::ShaderModule,
}

impl VulkanPipeline {
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
    ) -> Self {
        let vertex_code =
            read_spv(&mut File::open(vertex_file).expect("Failed to open Vertex File")).unwrap();
        let vertex_module_info = vk::ShaderModuleCreateInfo::default().code(&vertex_code);
        let fragment_code =
            read_spv(&mut File::open(fragment_file).expect("Failed to open Fragment File"))
                .unwrap();
        let fragment_module_info = vk::ShaderModuleCreateInfo::default().code(&fragment_code);

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

        let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::default()
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
        let viewport_state_info = vk::PipelineViewportStateCreateInfo::default()
            .scissors(&scissors)
            .viewports(&viewports);

        let rasterization_info = vk::PipelineRasterizationStateCreateInfo {
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            line_width: 1.0,
            polygon_mode: vk::PolygonMode::FILL,
            cull_mode: vk::CullModeFlags::NONE, // TODO
            ..Default::default()
        };
        let multisample_state_info = vk::PipelineMultisampleStateCreateInfo {
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            ..Default::default()
        };

        let depth_state_info = vk::PipelineDepthStencilStateCreateInfo::default();
        // .depth_test_enable(true)
        // .depth_write_enable(true)
        // .depth_compare_op(vk::CompareOp::LESS)
        // .max_depth_bounds(1.0);
        let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
            color_write_mask: vk::ColorComponentFlags::RGBA,
            ..Default::default()
        }];
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op(vk::LogicOp::COPY)
            .attachments(&color_blend_attachment_states);

        let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR]; // TODO
        let dynamic_state_info =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_state);

        let graphic_pipeline_info = vk::GraphicsPipelineCreateInfo::default()
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
                &[graphic_pipeline_info],
                None,
            )
        }
        .expect("Unable to create graphics pipeline");

        Self {
            pipeline: graphics_pipelines[0],
            vertex_module,
            fragment_module,
        }
    }

    pub fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            device.destroy_shader_module(self.vertex_module, None);
            device.destroy_shader_module(self.fragment_module, None);
            device.destroy_pipeline(self.pipeline, None)
        }
    }
}

#[allow(dead_code)]
fn conv_shader_stage(model: spirv::ExecutionModel) -> vk::ShaderStageFlags {
    match model {
        spirv::ExecutionModel::Vertex => vk::ShaderStageFlags::VERTEX,
        spirv::ExecutionModel::TessellationControl => vk::ShaderStageFlags::TESSELLATION_CONTROL,
        spirv::ExecutionModel::TessellationEvaluation => {
            vk::ShaderStageFlags::TESSELLATION_EVALUATION
        }
        spirv::ExecutionModel::Geometry => vk::ShaderStageFlags::GEOMETRY,
        spirv::ExecutionModel::Fragment => vk::ShaderStageFlags::FRAGMENT,
        spirv::ExecutionModel::GLCompute => vk::ShaderStageFlags::COMPUTE,
        spirv::ExecutionModel::Kernel => todo!(),
        spirv::ExecutionModel::TaskNV => vk::ShaderStageFlags::TASK_NV,
        spirv::ExecutionModel::MeshNV => vk::ShaderStageFlags::MESH_NV,
        spirv::ExecutionModel::RayGenerationNV => vk::ShaderStageFlags::RAYGEN_NV,
        spirv::ExecutionModel::IntersectionNV => vk::ShaderStageFlags::INTERSECTION_NV,
        spirv::ExecutionModel::AnyHitNV => vk::ShaderStageFlags::ANY_HIT_NV,
        spirv::ExecutionModel::ClosestHitNV => vk::ShaderStageFlags::CLOSEST_HIT_NV,
        spirv::ExecutionModel::MissNV => vk::ShaderStageFlags::MISS_NV,
        spirv::ExecutionModel::CallableNV => vk::ShaderStageFlags::CALLABLE_NV,
        spirv::ExecutionModel::TaskEXT => vk::ShaderStageFlags::TASK_EXT,
        spirv::ExecutionModel::MeshEXT => vk::ShaderStageFlags::MESH_EXT,
    }
}
