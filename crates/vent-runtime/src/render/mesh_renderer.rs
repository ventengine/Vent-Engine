use glam::{Mat4, Quat, Vec3};
use std::collections::HashMap;

use vent_common::render::model::Mesh3D;
use vent_common::render::UBO3D;

use vent_ecs::component::Entity;

#[derive(Default)]
pub struct MeshRenderer3D {
    map: HashMap<Entity, Mesh3D>,
}

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
            Self::update_trans_matrix(mesh, ubo);
            mesh.bind(rpass);
            rpass.pop_debug_group();
            rpass.insert_debug_marker("Draw!");
            mesh.draw(rpass);
        }
    }

    fn update_trans_matrix(object: &Mesh3D, ubo: &mut UBO3D) {
        // TODO:
        let rotation_quat = Quat::from_rotation_x(object.rotation.x)
            * Quat::from_rotation_y(object.rotation.y)
            * Quat::from_rotation_z(object.rotation.z);
        let transformation_matrix = Mat4::from_scale_rotation_translation(
            Vec3::new(20.0, 20.0, 20.0),
            rotation_quat,
            object.position,
        );
        // ubo.transformation = transformation_matrix.to_cols_array_2d();

        // ubo.transformation = glam::Vec2::rotate(
        //     ubo.transformation,
        //     object.rotation.x.to_radians(),
        //     glm::vec3(1.0f, 0.0f, 0.0f),
        // );
        // ubo.transformation = glm::rotate(
        //     ubo.transformation,
        //     object.rotation.y.to_radians(),
        //     glm::vec3(0.0f, 1.0f, 0.0f),
        // );
        // ubo.transformation = ubo.transformation.add().p
        // ubo.transformation = glm::rotate(
        //     ubo.transformation,
        //     object.rotation.z.to_radians(),
        //     glm::vec3(0.0f, 0.0f, 1.0f),
        // );
    }
}
