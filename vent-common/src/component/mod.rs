use std::any::Any;
use std::collections::HashMap;

pub type Entity = usize;

pub struct Component<T> {
    map: HashMap<Entity, T>,
}

impl<T> Component<T> {
    pub fn new() -> Component<T> {
        Component {
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, entity: Entity, component: T) {
        self.map.insert(entity, component);
    }

    pub fn remove(&mut self, entity: Entity) {
        self.map.remove(&entity);
    }

    pub fn get(&self, entity: Entity) -> Option<&T> {
        self.map.get(&entity)
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        self.map.get_mut(&entity)
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<Entity, T> {
        self.map.iter()
    }

    pub fn iter_mut(&mut self) -> std::collections::hash_map::IterMut<Entity, T> {
        self.map.iter_mut()
    }
}
