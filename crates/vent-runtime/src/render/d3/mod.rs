use ash::vk;
use glam::Mat4;
use vent_assets::Mesh3D;

use vent_ecs::world::World;
use vent_rendering::{instance::VulkanInstance, Vertex, Vertex3D};
use winit::dpi::PhysicalSize;

use super::{
    camera::{Camera, Camera3D},
    model::Entity3D,
    model_renderer::ModelRenderer3D,
    Renderer,
};

pub mod light_renderer;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct UBO3D {
    pub view_position: [f32; 3],
    pub _padding: u32,
    pub projection: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    pub transformation: [[f32; 4]; 4],
}

impl Default for UBO3D {
    fn default() -> Self {
        Self {
            view_position: Default::default(),
            _padding: Default::default(),
            projection: Default::default(),
            view: Default::default(),
            transformation: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }
}

pub struct Renderer3D {
    mesh_renderer: ModelRenderer3D,
    // light_renderer: LightRenderer,
    tmp_light_mesh: Mesh3D,
    // bind_group: wgpu::BindGroup,
    // depth_view: wgpu::TextureView,
    // uniform_buf: wgpu::Buffer,
    pipeline: vk::Pipeline,
    // pipeline_wire: Option<wgpu::RenderPipeline>,
}

impl Renderer for Renderer3D {
    fn init(instance: &VulkanInstance, camera: &mut dyn Camera) -> Self
    where
        Self: Sized,
    {
        let camera: &Camera3D = camera.downcast_ref().unwrap();
        let vertex_shader = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/res/shaders/app/3D/shader.vert"
        );
        let fragment_shader = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/res/shaders/app/3D/shader.frag"
        );
        let pipeline = vent_rendering::pipeline::create_pipeline(
            instance,
            vertex_shader.to_owned(),
            fragment_shader.to_owned(),
            Vertex3D::BINDING_DESCRIPTION,
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
            let mesh = Entity3D::new(vent_assets::Model3D::load(instance, model).await);
            mesh_renderer.insert(world.create_entity(), mesh);
        });

        // Record Command Buffers
        let render_area = vk::Rect2D::builder()
            .offset(vk::Offset2D::default())
            .extent(instance.surface_resolution)
            .build();

        let color_clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        };

        let depth_clear_value = vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {
                depth: 1.0,
                stencil: 0,
            },
        };

        let clear_values = &[color_clear_value, depth_clear_value];

        for (i, command_buffer) in instance.command_buffers.iter().enumerate() {
            unsafe {
                let info = vk::CommandBufferBeginInfo::default();

                instance.device.begin_command_buffer(*command_buffer, &info);

                let info = vk::RenderPassBeginInfo::builder()
                    .render_pass(instance.render_pass)
                    .framebuffer(instance.frame_buffers[i])
                    .render_area(render_area)
                    .clear_values(clear_values);

                instance.device.cmd_begin_render_pass(
                    *command_buffer,
                    &info,
                    vk::SubpassContents::INLINE,
                );
                instance.device.cmd_bind_pipeline(
                    *command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    pipeline,
                );

                mesh_renderer.record_buffer(
                    instance,
                    *command_buffer,
                    i,
                    instance.pipeline_layout,
                    &mut camera.ubo(),
                );

                // END
                instance.device.cmd_end_render_pass(*command_buffer);
                instance.device.end_command_buffer(*command_buffer);
            }
        }

        // // Create pipeline layout
        // let vertex_group_layout =
        //     device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //         label: Some("3D Bind Group Layout"),
        //         entries: &[wgpu::BindGroupLayoutEntry {
        //             binding: 0,
        //             visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
        //             ty: wgpu::BindingType::Buffer {
        //                 ty: wgpu::BufferBindingType::Uniform,
        //                 has_dynamic_offset: false,
        //                 min_binding_size: wgpu::BufferSize::new(
        //                     mem::size_of::<UBO3D>() as wgpu::BufferAddress
        //                 ),
        //             },
        //             count: None,
        //         }],
        //     });

        // let light_renderer = LightRenderer::new(device, &vertex_group_layout, config.format);

        // // Create other resources
        // let ubo = camera.ubo();
        // let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Uniform Buffer"),
        //     contents: bytemuck::bytes_of(&ubo),
        //     usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        // });

        let tmp_light_mesh = create_tmp_cube(instance);

        // let depth_view = vent_assets::Texture::create_depth_view(
        //     device,
        //     config.width,
        //     config.height,
        //     Some("Depth Buffer"),
        // );

        Self {
            mesh_renderer,
            //    light_renderer,
            tmp_light_mesh,
            // depth_view,
            // bind_group,
            // uniform_buf,
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
        // self.depth_view = vent_assets::Texture::create_depth_view(
        //     device,
        //     config.width,
        //     config.height,
        //     Some("Depth Buffer"),
        // );

        // let camera: &mut Camera3D = camera.downcast_mut().unwrap();

        // camera.recreate_projection(config.width as f32 / config.height as f32);
        // queue.write_buffer(&self.uniform_buf, 0, bytemuck::cast_slice(&[camera.ubo()]));
    }

    fn render(&mut self, instance: &VulkanInstance, image_index: u32, camera: &mut dyn Camera) {
        let camera: &mut Camera3D = camera.downcast_mut().unwrap();

        let _ubo = camera.ubo();

        //         self.light_renderer
        //             .render(&mut rpass, &self.bind_group, &self.tmp_light_mesh);

        //         rpass.set_bind_group(0, &self.bind_group, &[]);
        //         rpass.set_bind_group(2, &self.light_renderer.light_bind_group, &[]);

        //     }
        camera.write(instance, image_index)
    }

    fn destroy(&mut self, instance: &VulkanInstance) {
        self.mesh_renderer.destroy_all(instance);
        unsafe {
            instance.device.destroy_pipeline(self.pipeline, None);
        }
    }
}

fn write_sets(
    instance: &VulkanInstance,
    diffuse_texture: VulkanImage,
    uniforms_buffers: &Vec<VulkanBuffer>,
) -> Vec<vk::DescriptorSet> {
    let descriptor_sets = VulkanInstance::allocate_descriptor_sets(
        &instance.device,
        instance.descriptor_pool,
        instance.descriptor_set_layout,
        uniforms_buffers.len(),
    );

    for (i, &_descritptor_set) in descriptor_sets.iter().enumerate() {
        let image_info = vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(diffuse_texture.image_view)
            .sampler(diffuse_texture.sampler)
            .build();

        let material_buffer_info = vk::DescriptorBufferInfo::builder()
            .buffer(uniforms_buffers[i].buffer)
            .offset(0)
            .range(size_of::<Material>() as vk::DeviceSize)
            .build();

        let light_buffer_info = vk::DescriptorBufferInfo::builder()
            .buffer(uniforms_buffers[i].buffer)
            .offset(0)
            .range(size_of::<Light>() as vk::DeviceSize)
            .build();

        let desc_sets = [
            // Vertex
            vk::WriteDescriptorSet {
                dst_set: descriptor_sets[0],
                dst_binding: 0,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                p_buffer_info: &buffer_info,
                ..Default::default()
            },
            // Fragment
            vk::WriteDescriptorSet {
                dst_set: descriptor_sets[0],
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                p_image_info: &image_info,
                ..Default::default()
            },
            vk::WriteDescriptorSet {
                dst_set: descriptor_sets[0],
                dst_binding: 1,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                p_buffer_info: &material_buffer_info,
                ..Default::default()
            },
            vk::WriteDescriptorSet {
                dst_set: descriptor_sets[0],
                dst_binding: 2,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                p_buffer_info: &light_buffer_info,
                ..Default::default()
            },
        ];

        unsafe {
            instance
                .device
                .update_descriptor_sets(&fragment_desc_sets, &[]);
        }
    }
    descriptor_sets
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
        &instance.device,
        &instance.memory_allocator,
        &vertices,
        &indices,
        None,
        None,
    )
}
