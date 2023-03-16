use std::collections::HashMap;
use vent_common::component::Entity;
use vent_common::render::model::Mesh3D;

#[derive(Default)]
pub struct MeshRenderer {
    map: HashMap<Entity, Mesh3D>,
}

impl MeshRenderer {
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
}


