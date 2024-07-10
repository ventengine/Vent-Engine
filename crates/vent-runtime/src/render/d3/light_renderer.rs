use ash::vk;
use vent_math::vec::vec3::Vec3;
use vent_rendering::{instance::VulkanInstance, mesh::Mesh3D, pipeline::VulkanPipeline, Vertex3D};

#[allow(dead_code)]
#[repr(C)]
pub struct LightUBO {
    pub position: Vec3,
    pub color: Vec3,
}

pub struct LightRenderer {
    pipeline_layout: vk::PipelineLayout,
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

        let pipeline_layout = instance.create_pipeline_layout(&[]);

        let pipeline = VulkanPipeline::create_simple_pipeline(
            instance,
            vertex_shader.as_ref(),
            fragment_shader.as_ref(),
            &[Vertex3D::binding_description()],
            pipeline_layout,
            &Vertex3D::input_descriptions(),
            instance.surface_resolution,
        );

        Self {
            pipeline_layout,
            pipeline,
        }
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
        unsafe {
            self.pipeline.destroy(device);
            device.destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }
}
