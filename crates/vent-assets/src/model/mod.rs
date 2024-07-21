use std::path::Path;

use ash::vk;
use loader::ModelLoader;
use vent_rendering::instance::VulkanInstance;
use vent_sdk::utils::stopwatch::Stopwatch;

use crate::Model3D;

mod loader;
mod optimizer;

impl Model3D {
    #[inline]
    pub async fn load<P: AsRef<Path>>(
        instance: &mut VulkanInstance,
        vertex_shader: P,
        fragment_shader: P,
        pipeline_layout: vk::PipelineLayout,
        path: P,
    ) -> Self {
        let sw = Stopwatch::new_and_start();
        let model = modelz::Model3D::load(path.as_ref()).expect("Failed to Load 3D Model");
        let model = ModelLoader::load(
            instance,
            vertex_shader.as_ref(),
            fragment_shader.as_ref(),
            pipeline_layout,
            model,
        )
        .await;
        log::info!(
            "Model {} took {}ms to Load, {} Pipelines, {} Materials",
            path.as_ref().display(),
            sw.elapsed_ms(),
            model.pipelines.len(),
            model.materials.len(),
        );
        model
    }

    /// So your ideal render loop would be

    /// For each pipeline
    ///  Set pipeline
    ///   For each material that uses pipeline
    ///      Set material bind group
    ///       For each primitive that uses material with pipeline
    ///        Draw primitive
    pub fn draw(
        &self,
        device: &ash::Device,
        pipeline_layout: vk::PipelineLayout,
        command_buffer: vk::CommandBuffer,
        buffer_index: usize,
        with_descriptor_set: bool,
    ) {
        self.pipelines.iter().for_each(|pipeline| {
            unsafe {
                device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    pipeline.pipeline,
                )
            }
            pipeline.materials.iter().for_each(|material| {
                if with_descriptor_set {
                    let material = &self.materials[material.material_index];
                    if let Some(ds) = &material.descriptor_set {
                        unsafe {
                            device.cmd_bind_descriptor_sets(
                                command_buffer,
                                vk::PipelineBindPoint::GRAPHICS,
                                pipeline_layout,
                                0,
                                &ds[buffer_index..=buffer_index],
                                &[],
                            )
                        }
                    }
                }
                material.meshes.iter().for_each(|mesh| {
                    // rpass.push_debug_group("Bind Mesh");
                    mesh.bind(device, command_buffer);
                    // rpass.pop_debug_group();
                    // rpass.insert_debug_marker("Draw!");
                    mesh.draw(device, command_buffer);
                });
            });
        })
    }

    pub fn destroy(&mut self, device: &ash::Device) {
        self.materials.drain(..).for_each(|mut material| {
            material.diffuse_texture.destroy(device);
        });
        self.pipelines.drain(..).for_each(|mut pipeline| {
            unsafe { device.destroy_pipeline(pipeline.pipeline, None) };
            pipeline.materials.drain(..).for_each(|mut model_material| {
                model_material.meshes.drain(..).for_each(|mut mesh| {
                    mesh.destroy(device);
                });
            });
        });
        // We are getting an Validation error when we try to free an descriptor set, They will all automatily freed when the Descriptor pool is destroyed
        unsafe { device.destroy_descriptor_pool(self.descriptor_pool, None) };
    }
}
