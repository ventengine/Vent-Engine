use ash::vk::{self};
use std::collections::HashMap;
use vent_ecs::entity::Entity;
use vent_rendering::instance::VulkanInstance;

use super::{camera::Camera3D, model::Entity3D};

#[derive(Default)]
pub struct ModelRenderer3D {
    map: HashMap<Entity, Entity3D>,
}

#[allow(dead_code)]
impl ModelRenderer3D {
    #[inline]
    pub fn insert(&mut self, entity: Entity, mesh: Entity3D) {
        self.map.insert(entity, mesh);
    }

    #[inline]
    pub fn remove(&mut self, entity: Entity) {
        self.map.remove(&entity);
    }

    #[inline]
    #[must_use]
    pub fn get(&self, entity: Entity) -> Option<&Entity3D> {
        self.map.get(&entity)
    }

    #[inline]
    #[must_use]
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut Entity3D> {
        self.map.get_mut(&entity)
    }

    #[inline]
    #[must_use]
    pub fn iter(&self) -> std::collections::hash_map::Iter<Entity, Entity3D> {
        self.map.iter()
    }

    #[inline]
    #[must_use]
    pub fn iter_mut(&mut self) -> std::collections::hash_map::IterMut<Entity, Entity3D> {
        self.map.iter_mut()
    }

    pub fn record_buffer(
        &self,
        instance: &VulkanInstance,
        command_buffer: vk::CommandBuffer,
        buffer_index: usize,
        pipeline_layout: vk::PipelineLayout,
        camera: &mut Camera3D,
    ) {
        for model in self.map.values() {
            camera.transformation = Entity3D::calc_trans_matrix(&model.model);
            camera.calc_matrix();
            camera.write(instance, pipeline_layout, command_buffer);

            model.model.draw(
                &instance.device,
                pipeline_layout,
                command_buffer,
                buffer_index,
                true,
            );
        }
    }

    pub fn destroy_all(&mut self, device: &ash::Device) {
        for model in self.map.values_mut() {
            model.model.destroy(device)
        }
    }
}
