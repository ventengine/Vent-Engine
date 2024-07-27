use std::mem::size_of;

use ash::vk;
use pollster::FutureExt;

use skybox_renderer::SkyBoxRenderer;
use vent_ecs::world::World;
use vent_math::{
    scalar::mat4::Mat4,
    vec::{vec3::Vec3, vec4::Vec4},
};
use vent_rendering::{
    any_as_u8_slice, buffer::VulkanBuffer, image::SkyBoxImages, instance::VulkanInstance,
    mesh::Mesh3D, vertex::VertexPos3D,
};

use super::{
    camera::{Camera, Camera3D},
    model::Entity3D,
    model_renderer::ModelRenderer3D,
    Renderer,
};

pub mod light_renderer;
pub mod skybox_renderer;

#[repr(C)]
pub struct MaterialUBO {
    pub base_color: Vec4,
    pub alpha_mode: u32,
    pub alpha_cutoff: f32,
}

#[repr(C)] // This fixed everthing... #[repr(C)]
/// We calculate all values on the CPU, This will save us alot of memory, Push constants only guarante us 128 bytes
pub struct Camera3DData {
    pub view_position: Vec3,
    pub proj_view_trans: Mat4,
}

impl Default for Camera3DData {
    fn default() -> Self {
        Self {
            view_position: Vec3::ZERO,
            proj_view_trans: Mat4::IDENTITY,
        }
    }
}

pub struct Renderer3D {
    mesh_renderer: ModelRenderer3D,
    skybox_renderer: SkyBoxRenderer,
    //light_renderer: LightRenderer,
    tmp_light_mesh: Mesh3D,
    pipeline_layout: vk::PipelineLayout,
    descriptor_set_layout: vk::DescriptorSetLayout,

    material_ubos: Vec<VulkanBuffer>,
    light_ubos: Vec<VulkanBuffer>,
}

impl Renderer for Renderer3D {
    fn init(instance: &mut VulkanInstance, _camera: &mut dyn Camera) -> Self
    where
        Self: Sized,
    {
        //  let _camera: &Camera3D = camera.downcast_ref().unwrap();

        let skybox_renderer = SkyBoxRenderer::new(
            instance,
            SkyBoxImages {
                right: concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/assets/textures/skybox/right.jpg"
                )
                .into(),
                left: concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/assets/textures/skybox/left.jpg"
                )
                .into(),
                top: concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/assets/textures/skybox/top.jpg"
                )
                .into(),
                bottom: concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/assets/textures/skybox/bottom.jpg"
                )
                .into(),
                front: concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/assets/textures/skybox/front.jpg"
                )
                .into(),
                back: concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/assets/textures/skybox/back.jpg"
                )
                .into(),
            },
        );

        let push_constant_range = vk::PushConstantRange::default()
            .size(size_of::<Camera3DData>() as u32)
            .stage_flags(vk::ShaderStageFlags::VERTEX);

        let desc_layout_bindings = [
            // Fragment
            vk::DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            },
            vk::DescriptorSetLayoutBinding {
                binding: 1,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            },
            // vk::DescriptorSetLayoutBinding {
            //     binding: 2,
            //     descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            //     descriptor_count: 1,
            //     stage_flags: vk::ShaderStageFlags::FRAGMENT,
            //     ..Default::default()
            // },
        ];
        let info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&desc_layout_bindings);

        let descriptor_set_layout =
            unsafe { instance.device.create_descriptor_set_layout(&info, None) }.unwrap();

        let pipeline_layout =
            instance.create_pipeline_layout(&[push_constant_range], &[descriptor_set_layout]);

        let mut mesh_renderer = ModelRenderer3D::default();

        // // -------------- DEMO -------------------
        let mut world = World::new();

        let model = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/models/test/Sponza-GLTF/Sponza.gltf"
        );

        // Sponza-GLTF/Sponza.gltf
        // bistro_outside.glb

        let mut material_ubos = vec![];
        let light_ubos = vec![];

        let vertex_shader = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/shaders/app/3D/shader.vert.spv"
        );
        let fragment_shader = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/shaders/app/3D/shader.frag.spv"
        );

        let mut mesh = Entity3D::new(
            vent_assets::Model3D::load(
                instance,
                vertex_shader,
                fragment_shader,
                pipeline_layout,
                model,
            )
            .block_on(),
        );
        for material in mesh.model.materials.iter_mut() {
            let descriptor_sets = VulkanInstance::allocate_descriptor_sets(
                &instance.device,
                mesh.model.descriptor_pool,
                descriptor_set_layout,
                instance.swapchain_images.len(),
            );

            for &descriptor_set in descriptor_sets.iter() {
                let diffuse_texture = &material.diffuse_texture;

                let matieral_buffer = VulkanBuffer::new_init(
                    instance,
                    size_of::<MaterialUBO>() as vk::DeviceSize,
                    vk::BufferUsageFlags::UNIFORM_BUFFER,
                    any_as_u8_slice(&MaterialUBO {
                        base_color: Vec4::from_array(material.base_color),
                        alpha_mode: material.alpha_mode as u32,
                        alpha_cutoff: material.alpha_cut,
                    }),
                    vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::DEVICE_LOCAL,
                    None,
                );
                // let light_buffer = VulkanBuffer::new_init(
                //     instance,
                //     size_of::<LightUBO>() as vk::DeviceSize,
                //     vk::BufferUsageFlags::UNIFORM_BUFFER,
                //     any_as_u8_slice(&LightUBO {
                //         position: Vec3::new(2.0, 100.0, 2.0),
                //         color: Vec3::new(1.0, 1.0, 1.0),
                //     }),
                //     vk::MemoryPropertyFlags::HOST_VISIBLE
                //         | vk::MemoryPropertyFlags::DEVICE_LOCAL,
                //     None,
                // );

                let image_info = vk::DescriptorImageInfo::default()
                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .image_view(diffuse_texture.image_view)
                    .sampler(diffuse_texture.sampler);

                let material_buffer_info = vk::DescriptorBufferInfo::default()
                    .buffer(*matieral_buffer)
                    .offset(0)
                    .range(size_of::<MaterialUBO>() as vk::DeviceSize);

                // let light_buffer_info = vk::DescriptorBufferInfo::default()
                //     .buffer(*light_buffer)
                //     .offset(0)
                //     .range(size_of::<LightUBO>() as vk::DeviceSize);

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
                    // vk::WriteDescriptorSet {
                    //     dst_set: descriptor_set,
                    //     dst_binding: 2,
                    //     descriptor_count: 1,
                    //     descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                    //     p_buffer_info: &light_buffer_info,
                    //     ..Default::default()
                    // },
                ];

                unsafe {
                    instance.device.update_descriptor_sets(&desc_sets, &[]);
                }

                material_ubos.push(matieral_buffer);
                //  light_ubos.push(light_buffer);
            }
            material.descriptor_set = Some(descriptor_sets);
        }

        mesh_renderer.insert(world.create_entity(), mesh);

        let tmp_light_mesh = create_simple_cube(instance);
        //  let light_renderer = LightRenderer::new(instance);

        Self {
            mesh_renderer,
            skybox_renderer,
            descriptor_set_layout,
            //   light_renderer,
            tmp_light_mesh,
            pipeline_layout,
            material_ubos,
            light_ubos,
            // pipeline_wire,
        }
    }

    fn resize(
        &mut self,
        _instance: &mut VulkanInstance,
        _new_size: (u32, u32),
        _camera: &mut dyn Camera,
    ) {
    }

    fn render(
        &mut self,
        instance: &VulkanInstance,
        image_index: u32,
        command_buffer: vk::CommandBuffer,
        camera: &mut dyn Camera,
    ) {
        let camera: &mut Camera3D = camera.downcast_mut().unwrap();

        let image_index = image_index as usize;

        let render_area = vk::Rect2D::default()
            .offset(vk::Offset2D::default())
            .extent(instance.surface_resolution);

        let viewport = vk::Viewport::default()
            .width(instance.surface_resolution.width as f32)
            .height(instance.surface_resolution.height as f32)
            .max_depth(1.0);

        unsafe {
            instance
                .device
                .cmd_set_scissor(command_buffer, 0, &[render_area]);
            instance
                .device
                .cmd_set_viewport(command_buffer, 0, &[viewport]);

            self.skybox_renderer
                .draw(&instance.device, command_buffer, camera, image_index);

            self.mesh_renderer.record_buffer(
                instance,
                command_buffer,
                image_index,
                self.pipeline_layout,
                camera,
            );

            //    self.light_renderer.render(instance, command_buffer, image_index, &self.tmp_light_mesh);

            //  camera.write(instance, self.pipeline_layout, command_buffer);

            // END
        }
    }

    fn destroy(&mut self, instance: &VulkanInstance) {
        unsafe { instance.device.device_wait_idle().unwrap() };
        self.mesh_renderer.destroy_all(&instance.device);
        self.skybox_renderer.destroy(&instance.device);
        unsafe {
            instance
                .device
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None)
        };
        //self.light_renderer.destroy(&instance.device);
        self.material_ubos
            .drain(..)
            .for_each(|mut ubo| ubo.destroy(&instance.device));
        self.light_ubos
            .drain(..)
            .for_each(|mut ubo| ubo.destroy(&instance.device));

        self.tmp_light_mesh.destroy(&instance.device);
        unsafe {
            instance
                .device
                .destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }
}

fn create_simple_cube(instance: &VulkanInstance) -> Mesh3D {
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
        VertexPos3D {
            position: [-1.0, -1.0, 0.5],
        }, //0
        VertexPos3D {
            position: [1.0, -1.0, 0.5],
        }, //1
        VertexPos3D {
            position: [-1.0, 1.0, 0.5],
        }, //2
        VertexPos3D {
            position: [1.0, 1.0, 0.5],
        }, //3
        VertexPos3D {
            position: [-1.0, -1.0, -0.5],
        }, //4
        VertexPos3D {
            position: [0.0, -1.0, -0.5],
        }, //5
        VertexPos3D {
            position: [-1.0, 1.0, -0.5],
        }, //6
        VertexPos3D {
            position: [1.0, 1.0, -0.5],
        }, //7
    ];

    Mesh3D::new(
        instance,
        &vertices,
        vent_rendering::Indices::U16(indices.to_vec()),
        None,
    )
}
