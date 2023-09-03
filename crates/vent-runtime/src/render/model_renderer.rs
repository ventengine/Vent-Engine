use glam::{Mat4, Quat};
use std::collections::HashMap;
use vent_ecs::entity::Entity;

use super::{d3::UBO3D, model::Entity3D};

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

    pub fn render<'rp>(&'rp self, rpass: &mut wgpu::RenderPass<'rp>, ubo: &mut UBO3D) {
        for model in self.map.values() {
            ubo.transformation = Self::calc_trans_matrix(model).to_cols_array_2d();
            model.rendering_model.draw(rpass);
        }
    }

    fn calc_trans_matrix(mesh: &Entity3D) -> glam::Mat4 {
        let rotation_quat = Quat::from_scaled_axis(mesh.rotation.xyz());
        Mat4::from_scale_rotation_translation(mesh.scale, rotation_quat, mesh.position)
    }
}
