
use std::collections::HashMap;
use vent_common::render::model::Mesh3D;

use vent_ecs::component::Entity;

#[derive(Default)]
pub struct MeshRenderer3D {
    map: HashMap<Entity, Mesh3D>,
}

impl MeshRenderer3D {
    pub fn insert(&mut self, entity: Entity, mesh: Mesh3D) {
        self.map.insert(entity, mesh);
    }

    pub fn remove(&mut self, entity: Entity) {
        self.map.remove(&entity);
    }

    pub fn get(&self, entity: Entity) -> Option<&Mesh3D> {
        self.map.get(&entity)
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut Mesh3D> {
        self.map.get_mut(&entity)
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<Entity, Mesh3D> {
        self.map.iter()
    }

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

    //  pub fn update_trans_matrix(&self, object: Mesh3D, ubo: &mut UBO3D) {
    //      // TODO: Scale
    //
    //      ubo.transformation = glm::rotate(ubo_vs.trans, object.rotation.x.to_radians(), glm::vec3(1.0f, 0.0f, 0.0f));
    //    ubo.transformation = glm::rotate(ubo_vs.trans, object.rotation.y.to_radians(), glm::vec3(0.0f, 1.0f, 0.0f));
    //    ubo.transformation = glm::rotate(ubo_vs.trans, object.rotation.z.to_radians(), glm::vec3(0.0f, 0.0f, 1.0f));
    //}
}
