use glam::{Mat4, Quat};
use std::collections::HashMap;
use vent_ecs::entity::Entity;

use vent_common::render::UBO3D;

use super::model::Model3D;

#[derive(Default)]
pub struct ModelRenderer3D {
    map: HashMap<Entity, Model3D>,
}

#[allow(dead_code)]
impl ModelRenderer3D {
    #[inline]
    pub fn insert(&mut self, entity: Entity, mesh: Model3D) {
        self.map.insert(entity, mesh);
    }

    #[inline]
    pub fn remove(&mut self, entity: Entity) {
        self.map.remove(&entity);
    }

    #[inline]
    #[must_use]
    pub fn get(&self, entity: Entity) -> Option<&Model3D> {
        self.map.get(&entity)
    }

    #[inline]
    #[must_use]
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut Model3D> {
        self.map.get_mut(&entity)
    }

    #[inline]
    #[must_use]
    pub fn iter(&self) -> std::collections::hash_map::Iter<Entity, Model3D> {
        self.map.iter()
    }

    #[inline]
    #[must_use]
    pub fn iter_mut(&mut self) -> std::collections::hash_map::IterMut<Entity, Model3D> {
        self.map.iter_mut()
    }

    pub fn render<'rp>(&'rp self, rpass: &mut wgpu::RenderPass<'rp>, ubo: &mut UBO3D) {
        for (_, model) in self.map.iter() {
            Self::update_trans_matrix(model, ubo);
            model.rendering_model.draw(rpass);
        }
    }

    fn update_trans_matrix(mesh: &Model3D, ubo: &mut UBO3D) {
        let rotation_quat = Quat::from_rotation_x(mesh.rotation.x)
            * Quat::from_rotation_y(mesh.rotation.y)
            * Quat::from_rotation_z(mesh.rotation.z);
        let transformation_matrix =
            Mat4::from_scale_rotation_translation(mesh.scale, rotation_quat, mesh.position);
        ubo.transformation = transformation_matrix.to_cols_array_2d();
    }
}
