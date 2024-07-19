use std::mem::size_of;

use ash::vk;
use vent_math::scalar::mat4::Mat4;
use vent_rendering::{
    any_as_u8_slice, image::VulkanImage, instance::VulkanInstance, mesh::Mesh3D,
    pipeline::VulkanPipeline, Vertex3D,
};

use crate::render::camera::Camera3D;

use super::create_tmp_cube;

#[allow(dead_code)]
pub struct SkyBoxRenderer {
    pipeline: VulkanPipeline,
    image: VulkanImage,
    descriptor_pool: vk::DescriptorPool,
    push_constants: SkyBoxUBO,
    descriptor_sets: Vec<vk::DescriptorSet>,
    cube: Mesh3D,
}

#[repr(C)]
pub struct SkyBoxUBO {
    pub projection: Mat4,
    pub model: Mat4,
}

impl SkyBoxRenderer {
    pub fn new(instance: &VulkanInstance, path: &str) -> Self {
        log::debug!("Creating skybox");
        let vertex_shader = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/shaders/app/3D/skybox.vert.spv"
        );
        let fragment_shader = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/shaders/app/3D/skybox.frag.spv"
        );

        let desc_layout_bindings = [vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            ..Default::default()
        }];

        let push_constant_range = vk::PushConstantRange::default()
            .size(size_of::<SkyBoxUBO>() as u32)
            .stage_flags(vk::ShaderStageFlags::VERTEX);

        let pipeline = VulkanPipeline::create_simple_pipeline(
            instance,
            vertex_shader.as_ref(),
            fragment_shader.as_ref(),
            &[Vertex3D::binding_description()],
            &Vertex3D::input_descriptions(),
            instance.surface_resolution,
            &[push_constant_range],
            &desc_layout_bindings,
        );
        let cube = create_tmp_cube(instance);
        let push_constants = SkyBoxUBO {
            projection: Mat4::IDENTITY,
            model: Mat4::IDENTITY,
        };

        let descriptor_pool =
            Self::create_descriptor_pool(instance.swapchain_images.len() as u32, &instance.device);

        let descriptor_sets = VulkanInstance::allocate_descriptor_sets(
            &instance.device,
            descriptor_pool,
            pipeline.descriptor_set_layout,
            instance.swapchain_images.len(),
        );
        let image = VulkanImage::load_cubemap(instance, image::open(path).unwrap());

        for &descriptor_set in descriptor_sets.iter() {
            let diffuse_texture = &image;

            let image_info = vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(diffuse_texture.image_view)
                .sampler(diffuse_texture.sampler);

            let desc_sets = [vk::WriteDescriptorSet {
                dst_set: descriptor_set,
                dst_binding: 0, // From DescriptorSetLayoutBinding
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                p_image_info: &image_info,
                ..Default::default()
            }];

            unsafe {
                instance.device.update_descriptor_sets(&desc_sets, &[]);
            }
        }

        Self {
            pipeline,
            cube,
            image,
            push_constants,
            descriptor_pool,
            descriptor_sets,
        }
    }

    pub fn create_descriptor_pool(
        swapchain_count: u32,
        device: &ash::Device,
    ) -> vk::DescriptorPool {
        let pool_sizes = [vk::DescriptorPoolSize {
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 2 * swapchain_count,
        }];

        let create_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(2 * swapchain_count);

        unsafe { device.create_descriptor_pool(&create_info, None) }.unwrap()
    }
    #[allow(dead_code)]
    pub fn draw(
        &mut self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        camera: &Camera3D,
        buffer_index: usize,
    ) {
        self.push_constants = SkyBoxUBO {
            projection: camera.projection,
            model: camera.view,
        };
        unsafe {
            device.cmd_push_constants(
                command_buffer,
                self.pipeline.pipeline_layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                any_as_u8_slice(&self.push_constants),
            );
            device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.pipeline,
            );
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.pipeline_layout,
                0,
                &self.descriptor_sets[buffer_index..=buffer_index],
                &[],
            )
        };
        self.cube.bind(device, command_buffer);
        self.cube.draw(device, command_buffer);
    }

    pub fn destroy(&mut self, device: &ash::Device) {
        self.pipeline.destroy(device);
        self.cube.destroy(device);
        self.image.destroy(device);
        unsafe {
            device.destroy_descriptor_pool(self.descriptor_pool, None);
        }
    }
}
