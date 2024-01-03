use std::mem::size_of;

use ash::vk;
use glam::{Mat4, Vec3, Vec4};
use vent_assets::Mesh3D;

use vent_ecs::world::World;
use vent_rendering::{buffer::VulkanBuffer, instance::VulkanInstance, Vertex, Vertex3D};
use winit::dpi::PhysicalSize;

use self::light_renderer::LightUBO;

use super::{
    camera::{Camera, Camera3D},
    model::Entity3D,
    model_renderer::ModelRenderer3D,
    Renderer,
};

pub mod light_renderer;

pub struct MaterialUBO {
    pub base_color: Vec4,
}

pub struct Camera3DData {
    pub view_position: Vec3,
    pub projection: Mat4,
    pub view: Mat4,
    pub transformation: Mat4,
}

impl Default for Camera3DData {
    fn default() -> Self {
        Self {
            view_position: Default::default(),
            projection: Default::default(),
            view: Default::default(),
            transformation: Mat4::IDENTITY,
        }
    }
}

pub struct Renderer3D {
    mesh_renderer: ModelRenderer3D,
    // light_renderer: LightRenderer,
    tmp_light_mesh: Mesh3D,
    // depth_view: wgpu::TextureView,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    // pipeline_wire: Option<wgpu::RenderPipeline>,
}

impl Renderer for Renderer3D {
    fn init(instance: &VulkanInstance, camera: &mut dyn Camera) -> Self
    where
        Self: Sized,
    {
        let _camera: &Camera3D = camera.downcast_ref().unwrap();
        let vertex_shader = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/res/shaders/app/3D/shader.vert"
        );
        let fragment_shader = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/res/shaders/app/3D/shader.frag"
        );

        let push_contant_size = size_of::<Camera3DData>() as u32;
        let pipeline_layout = instance.create_pipeline_layout(push_contant_size);

        let pipeline = vent_rendering::pipeline::create_pipeline(
            instance,
            vertex_shader.to_owned(),
            fragment_shader.to_owned(),
            Vertex3D::BINDING_DESCRIPTION,
            pipeline_layout,
            &Vertex3D::input_descriptions(),
            instance.surface_resolution,
        );

        let mut mesh_renderer = ModelRenderer3D::default();

        // // -------------- DEMO -------------------
        let mut world = World::new();

        let model = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/res/models/test/Sponza-GLTF/Sponza.gltf"
        );

        pollster::block_on(async {
            let mut mesh = Entity3D::new(vent_assets::Model3D::load(instance, model).await);
            for mesh in mesh.rendering_model.meshes.iter_mut() {
                if let Some(material) = &mesh.material {
                    let descriptor_sets = VulkanInstance::allocate_descriptor_sets(
                        &instance.device,
                        instance.descriptor_pool,
                        instance.descriptor_set_layout,
                        instance.swapchain_images.len(),
                    );

                    let mut material_buffers = vec![];
                    let mut light_buffers = vec![];

                    for _ in 0..instance.swapchain_images.len() {
                        let buffer = VulkanBuffer::new_init_type(
                            instance,
                            &instance.memory_allocator,
                            size_of::<MaterialUBO>() as vk::DeviceSize,
                            vk::BufferUsageFlags::UNIFORM_BUFFER,
                            &MaterialUBO {
                                base_color: Vec4::from_array(material.base_color),
                            },
                            None,
                        );
                        material_buffers.push(buffer);
                        let buffer = VulkanBuffer::new_init_type(
                            instance,
                            &instance.memory_allocator,
                            size_of::<LightUBO>() as vk::DeviceSize,
                            vk::BufferUsageFlags::UNIFORM_BUFFER,
                            &LightUBO {
                                position: [2.0, 100.0, 2.0].into(),
                                color: [1.0, 1.0, 1.0].into(),
                            },
                            None,
                        );
                        light_buffers.push(buffer)
                    }

                    for (i, &descriptor_set) in descriptor_sets.iter().enumerate() {
                        let diffuse_texture = &material.diffuse_texture;

                        let image_info = vk::DescriptorImageInfo::builder()
                            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                            .image_view(diffuse_texture.image_view)
                            .sampler(diffuse_texture.sampler)
                            .build();

                        let material_buffer_info = vk::DescriptorBufferInfo::builder()
                            .buffer(material_buffers[i].buffer)
                            .offset(0)
                            .range(size_of::<MaterialUBO>() as vk::DeviceSize)
                            .build();

                        let light_buffer_info = vk::DescriptorBufferInfo::builder()
                            .buffer(light_buffers[i].buffer)
                            .offset(0)
                            .range(size_of::<LightUBO>() as vk::DeviceSize)
                            .build();

                        let desc_sets = [
                            vk::WriteDescriptorSet {
                                dst_set: descriptor_set,
                                dst_binding: 0, // From DescriptorSetLayoutBinding
                                descriptor_count: 1,
                                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                                p_image_info: &image_info,
                                ..Default::default()
                            },
                            vk::WriteDescriptorSet {
                                dst_set: descriptor_set,
                                dst_binding: 1,
                                descriptor_count: 1,
                                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                                p_buffer_info: &material_buffer_info,
                                ..Default::default()
                            },
                            vk::WriteDescriptorSet {
                                dst_set: descriptor_set,
                                dst_binding: 2,
                                descriptor_count: 1,
                                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                                p_buffer_info: &light_buffer_info,
                                ..Default::default()
                            },
                        ];

                        unsafe {
                            instance.device.update_descriptor_sets(&desc_sets, &[]);
                        }
                    }
                    mesh.set_descriptor_set(descriptor_sets);
                }
            }

            mesh_renderer.insert(world.create_entity(), mesh);
        });

        let tmp_light_mesh = create_tmp_cube(instance);

        Self {
            mesh_renderer,
            //    light_renderer,
            tmp_light_mesh,
            // depth_view,
            // bind_group,
            // uniform_buf,
            pipeline_layout,
            pipeline,
            // pipeline_wire,
        }
    }

    fn resize(
        &mut self,
        _instance: &VulkanInstance,
        _new_size: &PhysicalSize<u32>,
        _camera: &mut dyn Camera,
    ) {
        // TODO recreate depth image
    }

    fn render(&mut self, instance: &VulkanInstance, image_index: u32, camera: &mut dyn Camera) {
        let camera: &mut Camera3D = camera.downcast_mut().unwrap();

        camera.recreate_view();

        let image_index = image_index as usize;

        let command_buffer = instance.command_buffers[image_index];

        let render_area = vk::Rect2D::builder()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(instance.surface_resolution)
            .build();

        let viewport = vk::Viewport::builder()
            .width(instance.surface_resolution.width as f32)
            .height(instance.surface_resolution.height as f32)
            .max_depth(1.0);

        let color_clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.5, 0.5, 0.5, 1.0],
            },
        };

        let depth_clear_value = vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {
                depth: 1.0,
                stencil: 0,
            },
        };

        let clear_values = &[color_clear_value, depth_clear_value];

        unsafe {
            instance
                .device
                .reset_command_buffer(
                    command_buffer,
                    vk::CommandBufferResetFlags::RELEASE_RESOURCES,
                )
                .unwrap();

            let info = vk::CommandBufferBeginInfo::default();

            instance
                .device
                .begin_command_buffer(command_buffer, &info)
                .unwrap();

            let info = vk::RenderPassBeginInfo::builder()
                .render_pass(instance.render_pass)
                .framebuffer(instance.frame_buffers[image_index])
                .render_area(render_area)
                .clear_values(clear_values);

            let subpass_info =
                vk::SubpassBeginInfo::builder().contents(vk::SubpassContents::INLINE);

            instance
                .device
                .cmd_begin_render_pass2(command_buffer, &info, &subpass_info);
            instance.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );

            instance
                .device
                .cmd_set_scissor(command_buffer, 0, &[render_area]);
            instance
                .device
                .cmd_set_viewport(command_buffer, 0, &[*viewport]);

            self.mesh_renderer.record_buffer(
                instance,
                command_buffer,
                image_index,
                self.pipeline_layout,
                &mut camera.ubo,
            );

            camera.write(instance, self.pipeline_layout, command_buffer);

            // END
            let subpass_end_info = vk::SubpassEndInfo::default();
            instance
                .device
                .cmd_end_render_pass2(command_buffer, &subpass_end_info);
            instance.device.end_command_buffer(command_buffer).unwrap();
        }
    }

    fn destroy(&mut self, instance: &VulkanInstance) {
        self.mesh_renderer.destroy_all(instance);
        self.tmp_light_mesh
            .destroy(instance.descriptor_pool, &instance.device);
        unsafe {
            instance
                .device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            instance.device.destroy_pipeline(self.pipeline, None);
        }
    }
}

fn create_tmp_cube(instance: &VulkanInstance) -> vent_assets::Mesh3D {
    let indices = [
        //Top
        2, 6, 7, 2, 3, 7, //Bottom
        0, 4, 5, 0, 1, 5, //Left
        0, 2, 6, 0, 4, 6, //Right
        1, 3, 7, 1, 5, 7, //Front
        0, 2, 3, 0, 1, 3, //Back
        4, 6, 7, 4, 5, 7,
    ];

    let vertices = [
        Vertex3D {
            position: [-1.0, -1.0, 0.5],
            tex_coord: [0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
        }, //0
        Vertex3D {
            position: [1.0, -1.0, 0.5],
            tex_coord: [0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
        }, //1
        Vertex3D {
            position: [-1.0, 1.0, 0.5],
            tex_coord: [0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
        }, //2
        Vertex3D {
            position: [1.0, 1.0, 0.5],
            tex_coord: [0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
        }, //3
        Vertex3D {
            position: [-1.0, -1.0, -0.5],
            tex_coord: [0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
        }, //4
        Vertex3D {
            position: [0.0, -1.0, -0.5],
            tex_coord: [0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
        }, //5
        Vertex3D {
            position: [-1.0, 1.0, -0.5],
            tex_coord: [0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
        }, //6
        Vertex3D {
            position: [1.0, 1.0, -0.5],
            tex_coord: [0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
        }, //7
    ];

    vent_assets::Mesh3D::new(
        instance,
        &instance.memory_allocator,
        &vertices,
        &indices,
        None,
        None,
    )
}
