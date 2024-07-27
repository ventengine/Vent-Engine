use ash::vk;
use vent_math::vec::vec3::Vec3;
use vent_rendering::{
    instance::VulkanInstance, mesh::Mesh3D, pipeline::VulkanPipeline, vertex::Vertex3D,
};

#[allow(dead_code)]
#[repr(C)]
pub struct LightUBO {
    pub position: Vec3,
    pub color: Vec3,
}

pub struct LightRenderer {
    pipeline: VulkanPipeline,
}

#[allow(dead_code)]
impl LightRenderer {
    pub fn new(instance: &VulkanInstance) -> Self {
        let vertex_shader = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/shaders/app/3D/light.vert.spv"
        );
        let fragment_shader = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/shaders/app/3D/light.frag.spv"
        );

        let desc_layout_bindings = [
            vk::DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            vk::DescriptorSetLayoutBinding {
                binding: 1,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
        ];

        let pipeline = VulkanPipeline::create_simple_pipeline(
            instance,
            vertex_shader.as_ref(),
            fragment_shader.as_ref(),
            &[Vertex3D::binding_description()],
            &Vertex3D::input_descriptions(),
            instance.surface_resolution,
            &[],
            &desc_layout_bindings,
        );

        Self { pipeline }
    }

    pub fn render(
        &self,
        instance: &VulkanInstance,
        command_buffer: vk::CommandBuffer,
        _buffer_index: usize,
        mesh: &Mesh3D,
    ) {
        unsafe {
            instance.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.pipeline,
            )
        };

        mesh.bind(&instance.device, command_buffer);
        mesh.draw(&instance.device, command_buffer);
    }

    pub fn destroy(&mut self, device: &ash::Device) {
        self.pipeline.destroy(device);
    }
}
