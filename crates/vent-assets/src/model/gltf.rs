use std::{
    collections::HashMap,
    ffi::CStr,
    fs::{self, File},
    io::BufReader,
    path::Path,
};

use ash::{
    util::read_spv,
    vk::{self},
};
use gltf::{material::AlphaMode, mesh::Mode, texture::Sampler};
use image::DynamicImage;
use vent_rendering::{
    image::VulkanImage, instance::VulkanInstance, MaterialPipelineInfo, Vertex, Vertex3D,
};

use crate::{Material, Model3D, ModelPipeline};

use super::{optimizer, Mesh3D, ModelError};

pub(crate) struct GLTFLoader {}

struct MaterialData<'a> {
    image: DynamicImage,
    sampler: Option<Sampler<'a>>,
    base_color: [f32; 4],
    // Pipeline
    alpha_mode: AlphaMode,
    alpha_cut: Option<f32>,
    double_sided: bool,
}

impl GLTFLoader {
    pub async fn load(
        instance: &VulkanInstance,
        vertex_shader: &Path,
        fragment_shader: &Path,
        pipeline_layout: vk::PipelineLayout,
        path: &Path,
    ) -> Result<Model3D, ModelError> {
        let gltf = gltf::Gltf::from_reader(fs::File::open(path).unwrap()).unwrap();

        let path = path.parent().unwrap_or_else(|| Path::new("./"));

        let buffer_data = gltf::import_buffers(&gltf.document, Some(path), gltf.blob)
            .expect("Failed to Load glTF Buffers");

        let mut pipelines = Vec::new();
        let mut matrix = None;
        for scene in gltf.document.scenes() {
            for node in scene.nodes() {
                matrix = Some(node.transform().decomposed());
                Self::load_node(
                    instance,
                    vertex_shader,
                    fragment_shader,
                    pipeline_layout,
                    path,
                    node,
                    &buffer_data,
                    &gltf.document,
                    &mut pipelines,
                );
            }
        }

        let matrix = matrix.unwrap_or_default();

        Ok(Model3D {
            pipelines,
            position: matrix.0,
            rotation: matrix.1,
            scale: matrix.2,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn load_node(
        instance: &VulkanInstance,
        vertex_shader: &Path,
        fragment_shader: &Path,
        pipeline_layout: vk::PipelineLayout,
        model_dir: &Path,
        node: gltf::Node<'_>,
        buffer_data: &[gltf::buffer::Data],
        document: &gltf::Document,
        pipelines: &mut Vec<ModelPipeline>,
    ) {
        if let Some(mesh) = node.mesh() {
            Self::load_mesh_multithreaded(
                instance,
                model_dir,
                vertex_shader,
                fragment_shader,
                pipeline_layout,
                mesh,
                buffer_data,
                document,
                pipelines,
            );
        }
        node.children().for_each(|child| {
            Self::load_node(
                instance,
                vertex_shader,
                fragment_shader,
                pipeline_layout,
                model_dir,
                child,
                buffer_data,
                document,
                pipelines,
            )
        });
    }

    #[allow(clippy::too_many_arguments)]
    fn load_mesh_multithreaded(
        instance: &VulkanInstance,
        model_dir: &Path,
        vertex_shader: &Path,
        fragment_shader: &Path,
        pipeline_layout: vk::PipelineLayout,
        mesh: gltf::Mesh,
        buffer_data: &[gltf::buffer::Data],
        document: &gltf::Document,
        pipelines: &mut Vec<ModelPipeline>,
    ) {
        let materials = document.materials();

        //  This is very ugly i know, I originally had the idea to put this into vent_rendering::pipeline but then i had no good idea how to cache the vertex and fragment shader modules outside the for loop
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

        // So First we load all Materials our glTF file has
        let mut loaded_materials = Vec::with_capacity(materials.len());
        materials.for_each(|material| {
            let material_data = Self::parse_material_data(model_dir, material, buffer_data);
            let material = Self::load_material(instance, material_data);
            loaded_materials.push(material);
        });

        let mut cached_pipeline = HashMap::new();

        for (i, material) in loaded_materials.into_iter().enumerate() {
            let mut all_meshes = vec![];
            for primitive in mesh
                .primitives()
                .filter(|p| i == p.material().index().unwrap())
            // Maybe there are an better way, we just need the index that the premitive already has
            {
                let final_primitive = Self::load_primitive(buffer_data, primitive);
                let loaded_mesh = Mesh3D::new(
                    instance,
                    &instance.memory_allocator,
                    &final_primitive.0,
                    &final_primitive.1,
                    mesh.name(),
                );
                all_meshes.push(loaded_mesh);
            }
            let pipeline_info = MaterialPipelineInfo {
                mode: vk::PrimitiveTopology::TRIANGLE_LIST, // TODO
                alpha_cut: Some(ordered_float::OrderedFloat(material.alpha_cut)),
                double_sided: material.double_sided,
            };

            let model_material = crate::ModelMaterial {
                material,
                descriptor_set: None,
                meshes: all_meshes,
            };

            if cached_pipeline.contains_key(&pipeline_info) {
                pipelines.push(ModelPipeline {
                    pipeline: *cached_pipeline.get(&pipeline_info).unwrap(),
                    materials: vec![model_material], // TODO
                });
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

                    pipelines.push(ModelPipeline {
                        pipeline: graphics_pipelines[0],
                        materials: vec![model_material], // TODO
                    });

                    cached_pipeline.insert(pipeline_info, graphics_pipelines[0]);
                }
            }
        }

        unsafe {
            instance.device.destroy_shader_module(vertex_module, None);
            instance.device.destroy_shader_module(fragment_module, None);
        }
    }

    /**
     *  We will parse all Materials and save that what we need
     *  the Data will be saved on RAM
     */
    fn parse_material_data<'a>(
        model_dir: &'a Path,
        material: gltf::Material<'a>,
        buffer_data: &'a [gltf::buffer::Data],
        // image_data: &[gltf::image::Data],
    ) -> MaterialData<'a> {
        let pbr = material.pbr_metallic_roughness();

        let diffuse_texture = if let Some(texture) = pbr.base_color_texture() {
            match texture.texture().source().source() {
                gltf::image::Source::View { view, mime_type } => {
                    let sampler = texture.texture().sampler();
                    let image = image::load_from_memory_with_format(
                        &buffer_data[view.buffer().index()],
                        image::ImageFormat::from_mime_type(mime_type)
                            .expect("TODO: Error Handling"),
                    )
                    .unwrap();
                    (image, Some(sampler))
                }
                gltf::image::Source::Uri { uri, mime_type } => {
                    let sampler = texture.texture().sampler();
                    let image = if let Some(mime_type) = mime_type {
                        image::load(
                            BufReader::new(File::open(model_dir.join(uri)).unwrap()),
                            image::ImageFormat::from_mime_type(mime_type)
                                .expect("TODO: Error Handling"),
                        )
                        .unwrap()
                    } else {
                        image::open(model_dir.join(uri)).unwrap()
                    };

                    (image, Some(sampler))
                }
            }
        } else {
            (
                image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
                    128,
                    128,
                    image::Rgba([255, 255, 255, 255]),
                )),
                None,
            )
        };
        MaterialData {
            image: diffuse_texture.0,
            double_sided: material.double_sided(),
            alpha_mode: material.alpha_mode(),
            alpha_cut: material.alpha_cutoff(),
            sampler: diffuse_texture.1,
            base_color: pbr.base_color_factor(),
        }
    }

    /**
     *  Creates an VulkanImage from Material Data, We want to do this Single threaded
     *  RAM -> VRAM
     */
    fn load_material(instance: &VulkanInstance, data: MaterialData) -> Material {
        let diffuse_texture = VulkanImage::from_image(
            instance,
            data.image,
            instance.command_pool,
            &instance.memory_allocator,
            instance.graphics_queue,
            data.sampler.map(|s| Self::convert_sampler(&s)),
        );

        Material {
            diffuse_texture,
            alpha_mode: data.alpha_mode,
            alpha_cut: data.alpha_cut.unwrap_or(0.5),
            double_sided: data.double_sided,
            base_color: data.base_color,
        }
    }

    /// Converts an gltf Texture Sampler into Vulkan Sampler Info
    fn convert_sampler<'a>(
        sampler: &'a gltf::texture::Sampler<'a>,
    ) -> vk::SamplerCreateInfo<'static> {
        let mag_filter = sampler.mag_filter().map_or(
            VulkanImage::DEFAULT_TEXTURE_FILTER,
            |filter| match filter {
                gltf::texture::MagFilter::Nearest => vk::Filter::NEAREST,
                gltf::texture::MagFilter::Linear => vk::Filter::LINEAR,
            },
        );

        let (min_filter, mipmap_filter) = sampler.min_filter().map_or(
            (
                VulkanImage::DEFAULT_TEXTURE_FILTER,
                vk::SamplerMipmapMode::LINEAR,
            ),
            |filter| match filter {
                gltf::texture::MinFilter::Nearest => {
                    (vk::Filter::NEAREST, vk::SamplerMipmapMode::NEAREST)
                }
                gltf::texture::MinFilter::Linear => {
                    (vk::Filter::LINEAR, vk::SamplerMipmapMode::NEAREST)
                }
                gltf::texture::MinFilter::NearestMipmapNearest => {
                    (vk::Filter::NEAREST, vk::SamplerMipmapMode::NEAREST)
                }
                gltf::texture::MinFilter::LinearMipmapNearest => {
                    (vk::Filter::LINEAR, vk::SamplerMipmapMode::NEAREST)
                }
                gltf::texture::MinFilter::NearestMipmapLinear => {
                    (vk::Filter::LINEAR, vk::SamplerMipmapMode::LINEAR)
                }
                gltf::texture::MinFilter::LinearMipmapLinear => {
                    (vk::Filter::LINEAR, vk::SamplerMipmapMode::LINEAR)
                }
            },
        );

        let address_mode_u = Self::conv_wrapping_mode(sampler.wrap_s());
        let address_mode_v = Self::conv_wrapping_mode(sampler.wrap_t());

        vk::SamplerCreateInfo::default()
            .mag_filter(mag_filter)
            .min_filter(min_filter)
            .mipmap_mode(mipmap_filter)
            .address_mode_u(address_mode_u)
            .address_mode_v(address_mode_v)
    }

    #[must_use]
    const fn conv_wrapping_mode(mode: gltf::texture::WrappingMode) -> vk::SamplerAddressMode {
        match mode {
            gltf::texture::WrappingMode::ClampToEdge => vk::SamplerAddressMode::CLAMP_TO_EDGE,
            gltf::texture::WrappingMode::MirroredRepeat => vk::SamplerAddressMode::MIRRORED_REPEAT,
            gltf::texture::WrappingMode::Repeat => vk::SamplerAddressMode::REPEAT,
        }
    }

    #[must_use]
    #[allow(dead_code)]
    const fn conv_primitive_mode(mode: Mode) -> vk::PrimitiveTopology {
        match mode {
            Mode::Points => vk::PrimitiveTopology::POINT_LIST,
            Mode::Lines => vk::PrimitiveTopology::LINE_LIST,
            Mode::LineLoop => vk::PrimitiveTopology::LINE_LIST,
            Mode::LineStrip => vk::PrimitiveTopology::LINE_STRIP,
            Mode::Triangles => vk::PrimitiveTopology::TRIANGLE_LIST,
            Mode::TriangleStrip => vk::PrimitiveTopology::TRIANGLE_STRIP,
            Mode::TriangleFan => vk::PrimitiveTopology::TRIANGLE_FAN,
        }
    }

    fn load_primitive(
        buffer_data: &[gltf::buffer::Data],
        primitive: gltf::Primitive,
    ) -> (Vec<Vertex3D>, Vec<u32>) {
        let reader = primitive.reader(|buffer| Some(&buffer_data[buffer.index()]));

        let mut vertices: Vec<Vertex3D> = reader
            .read_positions()
            .unwrap()
            .map(|position| Vertex3D {
                position,
                tex_coord: Default::default(),
                normal: Default::default(),
            })
            .collect();

        if let Some(normal_attribute) = reader.read_normals() {
            for (normal_index, normal) in normal_attribute.enumerate() {
                vertices[normal_index].normal = normal;
            }
        }

        if let Some(tex_coord_attribute) = reader.read_tex_coords(0).map(|v| v.into_f32()) {
            for (tex_coord_index, tex_coord) in tex_coord_attribute.enumerate() {
                vertices[tex_coord_index].tex_coord = tex_coord;
            }
        }

        let vertices = optimizer::optimize_vertices(vertices);

        let indices: Vec<_> = reader.read_indices().unwrap().into_u32().collect();
        (vertices, indices)
    }
}
