use ash::vk::{self};
use glam::{Mat4, Quat};
use std::collections::HashMap;
use vent_ecs::entity::Entity;
use vent_rendering::instance::VulkanInstance;

use super::{d3::Camera3DData, model::Entity3D};

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
        ubo: &mut Camera3DData,
    ) {
        for model in self.map.values() {
            ubo.transformation = Self::calc_trans_matrix(model);
            model.model.draw(
                &instance.device,
                pipeline_layout,
                command_buffer,
                buffer_index,
            );
        }
    }

    pub fn destroy_all(&mut self, instance: &VulkanInstance) {
        for model in self.map.values_mut() {
            model.model.destroy(instance)
        }
    }

    fn calc_trans_matrix(mesh: &Entity3D) -> glam::Mat4 {
        let rotation_quat = Quat::from_scaled_axis(Quat::from_array(mesh.model.rotation).xyz());
        Mat4::from_scale_rotation_translation(
            mesh.model.scale.into(),
            rotation_quat,
            mesh.model.position.into(),
        )
    }
}
