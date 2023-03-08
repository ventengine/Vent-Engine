use std::any::Any;
use std::collections::HashMap;

type Entity = usize;

struct Component<T> {
    map: HashMap<Entity, T>,
}

impl<T> Component<T> {
    fn new() -> Component<T> {
        Component {
            map: HashMap::new(),
        }
    }

    fn insert(&mut self, entity: Entity, component: T) {
        self.map.insert(entity, component);
    }

    fn remove(&mut self, entity: Entity) {
        self.map.remove(&entity);
    }

    fn get(&self, entity: Entity) -> Option<&T> {
        self.map.get(&entity)
    }

    fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        self.map.get_mut(&entity)
    }

    fn iter(&self) -> std::collections::hash_map::Iter<Entity, T> {
        self.map.iter()
    }

    fn iter_mut(&mut self) -> std::collections::hash_map::IterMut<Entity, T> {
        self.map.iter_mut()
    }
}

struct World {
    entities: Vec<Entity>,
    next_entity: Entity,
    components: Vec<Box<dyn Any>>,
}

impl World {
    fn new() -> World {
        World {
            entities: Vec::new(),
            next_entity: 0,
            components: Vec::new(),
        }
    }

    fn create_entity(&mut self) -> Entity {
        let entity = self.next_entity;
        self.next_entity += 1;
        self.entities.push(entity);
        entity
    }

    fn delete_entity(&mut self, entity: Entity) {
        let index = self.entities.iter().position(|&e| e == entity).unwrap();
        self.entities.swap_remove(index);
        for component in &mut self.components {
            component.downcast_mut::<Component<Entity>>().unwrap().remove(entity);
        }
    }

    fn register_component<T: 'static>(&mut self) -> usize {
        self.components.push(Box::new(Component::<T>::new()));
        self.components.len() - 1
    }

    fn get_component<T: 'static>(&self, component_id: usize) -> &Component<T> {
        self.components[component_id].downcast_ref::<Component<T>>().unwrap()
    }

    fn get_component_mut<T: 'static>(&mut self, component_id: usize) -> &mut Component<T> {
        self.components[component_id]
            .downcast_mut::<Component<T>>()
            .unwrap()
    }

    fn iter_entities(&self) -> std::slice::Iter<Entity> {
        self.entities.iter()
    }
}
