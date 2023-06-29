use glam::{Mat4, Quat};
use std::collections::HashMap;
use vent_assets::mesh::Mesh3D;
use vent_ecs::entity::Entity;

use vent_common::render::UBO3D;

#[derive(Default)]
pub struct MeshRenderer3D {
    map: HashMap<Entity, Mesh3D>,
}

#[allow(dead_code)]
impl MeshRenderer3D {
    #[inline]
    pub fn insert(&mut self, entity: Entity, mesh: Mesh3D) {
        self.map.insert(entity, mesh);
    }

    #[inline]
    pub fn remove(&mut self, entity: Entity) {
        self.map.remove(&entity);
    }

    #[inline]
    #[must_use]
    pub fn get(&self, entity: Entity) -> Option<&Mesh3D> {
        self.map.get(&entity)
    }

    #[inline]
    #[must_use]
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut Mesh3D> {
        self.map.get_mut(&entity)
    }

    #[inline]
    #[must_use]
    pub fn iter(&self) -> std::collections::hash_map::Iter<Entity, Mesh3D> {
        self.map.iter()
    }

    #[inline]
    #[must_use]
    pub fn iter_mut(&mut self) -> std::collections::hash_map::IterMut<Entity, Mesh3D> {
        self.map.iter_mut()
    }

    pub fn render<'rp>(&'rp self, rpass: &mut wgpu::RenderPass<'rp>, ubo: &mut UBO3D) {
        for map in self.iter() {
            let mesh = map.1;

            rpass.push_debug_group("Bind Mesh");
            mesh.bind(rpass);
            Self::update_trans_matrix(self, mesh, ubo);
            rpass.pop_debug_group();
            rpass.insert_debug_marker("Draw!");
            mesh.draw(rpass);
        }
    }

    fn update_trans_matrix(&self, mesh: &Mesh3D, ubo: &mut UBO3D) {
        let rotation_quat = Quat::from_rotation_x(mesh.rotation.x)
            * Quat::from_rotation_y(mesh.rotation.y)
            * Quat::from_rotation_z(mesh.rotation.z);
        let transformation_matrix =
            Mat4::from_scale_rotation_translation(mesh.scale, rotation_quat, mesh.position);
        ubo.transformation = transformation_matrix.to_cols_array_2d();
    }
}
