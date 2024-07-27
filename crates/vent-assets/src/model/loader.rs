use std::{collections::HashMap, ffi::CStr, fs::File, path::Path};

use ash::{
    util::read_spv,
    vk::{self, PipelineShaderStageCreateInfo},
};
use vent_rendering::{
    image::VulkanImage, instance::VulkanInstance, mesh::Mesh3D, vertex::Vertex3D,
    MaterialPipelineInfo, DEFAULT_TEXTURE_FILTER,
};

use crate::{Material, Model3D, ModelPipeline};

pub(crate) struct ModelLoader {}

impl ModelLoader {
    pub async fn load(
        instance: &mut VulkanInstance,
        vertex_shader: &Path,
        fragment_shader: &Path,
        pipeline_layout: vk::PipelineLayout,
        model: modelz::Model3D,
    ) -> crate::Model3D {
        // let mut matrix = None;

        // Do not load for every node, So we load it here
        let vertex_code =
            read_spv(&mut File::open(vertex_shader).expect("Failed to open Vertex File")).unwrap();
        let vertex_module_info = vk::ShaderModuleCreateInfo::default().code(&vertex_code);
        let fragment_code =
            read_spv(&mut File::open(fragment_shader).expect("Failed to open Fragment File"))
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

        let mut materials = Vec::new();
        for mat in model.materials {
            materials.push(Self::load_material(instance, mat));
        }

        let mut pipelines = Vec::new();
        Self::load_meshes(
            instance,
            model.meshes,
            &shader_stage_create_info,
            pipeline_layout,
            &materials,
            &mut pipelines,
        );

        unsafe {
            instance.device.destroy_shader_module(vertex_module, None);
            instance.device.destroy_shader_module(fragment_module, None);
        }

        // let matrix = matrix.unwrap_or_default();

        let descriptor_pool = Self::create_descriptor_pool(
            materials.len() as u32,
            instance.swapchain_images.len() as u32,
            &instance.device,
        );

        Model3D {
            descriptor_pool,
            materials,
            pipelines,
            position: [0.0, 0.0, 0.0], // TODO: matrix.0
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        }
    }

    fn create_descriptor_pool(
        material_count: u32,
        swapchain_count: u32,
        device: &ash::Device,
    ) -> vk::DescriptorPool {
        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: material_count * swapchain_count,
            },
            // Material UBO
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: material_count * swapchain_count,
            },
        ];

        let create_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(material_count * swapchain_count);

        log::debug!(
            "Creating Description Pool, Size {}",
            material_count * swapchain_count
        );

        unsafe { device.create_descriptor_pool(&create_info, None) }.unwrap()
    }

    fn load_meshes(
        instance: &mut VulkanInstance,
        meshes: Vec<modelz::Mesh>,
        shader_stage_create_info: &[PipelineShaderStageCreateInfo],
        pipeline_layout: vk::PipelineLayout,
        loaded_materials: &[Material],
        pipelines: &mut Vec<ModelPipeline>,
    ) {
        let surface_resolution = instance.surface_resolution;

        let binding = [Vertex3D::binding_description()];
        let attrib = Vertex3D::input_descriptions();
        let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_attribute_descriptions(&attrib)
            .vertex_binding_descriptions(&binding);

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

        let mut cached_pipeline: HashMap<MaterialPipelineInfo, usize> = HashMap::new(); // We just need to store the pipelines vec index

        for mesh in meshes {
            log::debug!("      Loading Mesh {:?}", &mesh.name);

            let mut all_meshes = vec![];

            let material_index = mesh.material_index.unwrap(); // TODO
            let material = &loaded_materials[material_index];
            {
                let loaded_mesh = Mesh3D::new(
                    instance,
                    &Self::convert_vertices(&mesh.vertices),
                    Self::convert_indices(mesh.indices.unwrap()),
                    mesh.name.as_deref(),
                );
                all_meshes.push(loaded_mesh);
            }
            let pipeline_info = MaterialPipelineInfo {
                mode: vk::PrimitiveTopology::TRIANGLE_LIST, // TODO
                alpha_cut: Some(ordered_float::OrderedFloat(material.alpha_cut)),
                double_sided: material.double_sided,
            };

            let model_material = crate::ModelMaterial {
                material_index,
                meshes: all_meshes,
            };

            if let Some(pipeline_index) = cached_pipeline.get(&pipeline_info) {
                pipelines[*pipeline_index].materials.push(model_material);
            } else {
                let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
                    topology: pipeline_info.mode,
                    ..Default::default()
                };
                let rasterization_info = vk::PipelineRasterizationStateCreateInfo {
                    front_face: vk::FrontFace::COUNTER_CLOCKWISE,
                    line_width: 1.0,
                    polygon_mode: vk::PolygonMode::FILL,
                    cull_mode: if pipeline_info.double_sided {
                        vk::CullModeFlags::NONE
                    } else {
                        vk::CullModeFlags::BACK
                    },
                    ..Default::default()
                };

                {
                    let multisample_state_info = vk::PipelineMultisampleStateCreateInfo {
                        rasterization_samples: vk::SampleCountFlags::TYPE_1,
                        ..Default::default()
                    };

                    let depth_state_info = vk::PipelineDepthStencilStateCreateInfo::default()
                        .depth_test_enable(true)
                        .depth_write_enable(true)
                        .depth_compare_op(vk::CompareOp::LESS)
                        .max_depth_bounds(1.0);
                    let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
                        color_write_mask: vk::ColorComponentFlags::RGBA,
                        ..Default::default()
                    }];
                    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::default()
                        .logic_op(vk::LogicOp::COPY)
                        .attachments(&color_blend_attachment_states);

                    let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR]; // TODO
                    let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::default()
                        .dynamic_states(&dynamic_state);

                    let graphic_pipeline_info = vk::GraphicsPipelineCreateInfo::default()
                        .stages(shader_stage_create_info)
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

                    cached_pipeline.insert(pipeline_info, pipelines.len());

                    pipelines.push(ModelPipeline {
                        pipeline: graphics_pipelines[0],
                        materials: vec![model_material], // TODO
                    });
                }
            }
        }
    }

    /**
     *  Creates an VulkanImage from Material Data, We want to do this Single threaded
     *  RAM -> VRAM
     */
    fn load_material(instance: &mut VulkanInstance, data: modelz::Material) -> Material {
        let diffuse_texture = if let Some(diffuse_texture) = data.diffuse_texture {
            VulkanImage::from_image(
                instance,
                diffuse_texture.image,
                true,
                Some(Self::convert_sampler(diffuse_texture.sampler)),
                data.name.as_deref(), // TODO: use texture name not material name
            )
        } else {
            VulkanImage::from_color(
                instance,
                [255, 255, 255, 255],
                vk::Extent2D {
                    width: 128,
                    height: 128,
                },
                data.name.as_deref(), // TODO: use texture name not material name
            )
        };

        Material {
            diffuse_texture,
            descriptor_set: None,
            alpha_mode: data.alpha_mode,
            alpha_cut: data.alpha_cutoff.unwrap_or(0.5),
            double_sided: data.double_sided,
            base_color: data.base_color.unwrap_or([1.0, 1.0, 1.0, 1.0]),
        }
    }

    /// Converts an gltf Texture Sampler into Vulkan Sampler Info
    fn convert_sampler(sampler: modelz::Sampler) -> vk::SamplerCreateInfo<'static> {
        let mag_filter = sampler
            .mag_filter
            .map_or(DEFAULT_TEXTURE_FILTER, |filter| match filter {
                modelz::MagFilter::Nearest => vk::Filter::NEAREST,
                modelz::MagFilter::Linear => vk::Filter::LINEAR,
            });

        let (min_filter, mipmap_filter) = sampler.min_filter.map_or(
            (DEFAULT_TEXTURE_FILTER, vk::SamplerMipmapMode::LINEAR),
            |filter| match filter {
                modelz::MinFilter::Nearest => (vk::Filter::NEAREST, vk::SamplerMipmapMode::NEAREST),
                modelz::MinFilter::Linear => (vk::Filter::LINEAR, vk::SamplerMipmapMode::NEAREST),
                modelz::MinFilter::NearestMipmapNearest => {
                    (vk::Filter::NEAREST, vk::SamplerMipmapMode::NEAREST)
                }
                modelz::MinFilter::LinearMipmapNearest => {
                    (vk::Filter::LINEAR, vk::SamplerMipmapMode::NEAREST)
                }
                modelz::MinFilter::NearestMipmapLinear => {
                    (vk::Filter::LINEAR, vk::SamplerMipmapMode::LINEAR)
                }
                modelz::MinFilter::LinearMipmapLinear => {
                    (vk::Filter::LINEAR, vk::SamplerMipmapMode::LINEAR)
                }
            },
        );

        let address_mode_u = Self::convert_wrapping_mode(&sampler.wrap_s);
        let address_mode_v = Self::convert_wrapping_mode(&sampler.wrap_t);

        vk::SamplerCreateInfo {
            mag_filter,
            min_filter,
            mipmap_mode: mipmap_filter,
            address_mode_u,
            address_mode_v,
            ..Default::default()
        }
    }

    #[must_use]
    const fn convert_wrapping_mode(mode: &modelz::WrappingMode) -> vk::SamplerAddressMode {
        match mode {
            modelz::WrappingMode::ClampToEdge => vk::SamplerAddressMode::CLAMP_TO_EDGE,
            modelz::WrappingMode::MirroredRepeat => vk::SamplerAddressMode::MIRRORED_REPEAT,
            modelz::WrappingMode::Repeat => vk::SamplerAddressMode::REPEAT,
        }
    }

    fn convert_indices(indices: modelz::Indices) -> vent_rendering::Indices {
        match indices {
            modelz::Indices::U8(d) => vent_rendering::Indices::U8(d),
            modelz::Indices::U16(d) => vent_rendering::Indices::U16(d),
            modelz::Indices::U32(d) => vent_rendering::Indices::U32(d),
        }
    }

    fn convert_vertices(verticies: &[modelz::Vertex]) -> Vec<Vertex3D> {
        verticies
            .iter()
            .map(|vertex| Vertex3D {
                position: vertex.position,
                tex_coord: vertex.tex_coord.unwrap_or_default(), // TODO
                normal: vertex.normal.unwrap_or_default(),       // TODO
            })
            .collect()
    }

    // #[must_use]
    // #[allow(dead_code)]
    // const fn conv_primitive_mode(mode: Mode) -> vk::PrimitiveTopology {
    //     match mode {
    //         Mode::Points => vk::PrimitiveTopology::POINT_LIST,
    //         Mode::Lines => vk::PrimitiveTopology::LINE_LIST,
    //         Mode::LineLoop => vk::PrimitiveTopology::LINE_LIST,
    //         Mode::LineStrip => vk::PrimitiveTopology::LINE_STRIP,
    //         Mode::Triangles => vk::PrimitiveTopology::TRIANGLE_LIST,
    //         Mode::TriangleStrip => vk::PrimitiveTopology::TRIANGLE_STRIP,
    //         Mode::TriangleFan => vk::PrimitiveTopology::TRIANGLE_FAN,
    //     }
    // }
}
