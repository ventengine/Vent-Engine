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
    #[must_use]
    pub fn insert(&mut self, entity: Entity, mesh: Mesh3D) {
        self.map.insert(entity, mesh);
    }

    #[inline]
    #[must_use]
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

    pub fn render<'rp>(&'rp self, rpass: &mut wgpu::RenderPass<'rp>) {
        for map in self.iter() {
            let mesh = map.1;
            mesh.bind(rpass);
            mesh.draw(rpass);
        }
    }

    pub fn update_trans_matrix(&self, _object: Mesh3D, _ubo: &mut UBO3D) {
        // TODO: Scale

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
