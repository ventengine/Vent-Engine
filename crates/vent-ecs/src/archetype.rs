use std::{any::Any, collections::HashMap};

use crate::{component::Component, entity::Entity};

pub struct Archetype {
    entities: Vec<Entity>,
    component_data: HashMap<usize, Vec<Box<dyn Any>>>,
}

impl Archetype {
    /// Creates a new empty archetype.
    pub fn new() -> Self {
        Archetype {
            entities: Vec::new(),
            component_data: HashMap::new(),
        }
    }

    /// Adds an entity to the archetype.
    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    /// Removes an entity from the archetype.
    pub fn remove_entity(&mut self, entity: Entity) {
        if let Some(index) = self.entities.iter().position(|&e| e == entity) {
            self.entities.swap_remove(index);
        }
    }

    /// Adds a component to the archetype.
    pub fn add_component<T: Component>(&mut self, component_id: usize, component: T) {
        let component_data = self
            .component_data
            .entry(component_id)
            .or_insert(Vec::new());
        component_data.push(Box::new(component));
    }

    /// Removes a component from the archetype.
    pub fn remove_component<T: Component>(&mut self, component_id: usize, entity: Entity) {
        if let Some(component_data) = self.component_data.get_mut(&component_id) {
            if let Some(index) = self.entities.iter().position(|&e| e == entity) {
                component_data.swap_remove(index);
            }
        }
    }

    /// Retrieves a component from the archetype.
    pub fn get_component<T: Component>(&self, component_id: usize, entity: Entity) -> Option<&T> {
        if let Some(component_data) = self.component_data.get(&component_id) {
            if let Some(index) = self.entities.iter().position(|&e| e == entity) {
                if let Some(component) = component_data[index].downcast_ref::<T>() {
                    return Some(component);
                }
            }
        }
        None
    }

    /// Retrieves a mutable component from the archetype.
    pub fn get_component_mut<T: Component>(
        &mut self,
        component_id: usize,
        entity: Entity,
    ) -> Option<&mut T> {
        if let Some(component_data) = self.component_data.get_mut(&component_id) {
            if let Some(index) = self.entities.iter().position(|&e| e == entity) {
                if let Some(component) = component_data[index].downcast_mut::<T>() {
                    return Some(component);
                }
            }
        }
        None
    }

    /// Returns an iterator over the entities in the archetype.
    pub fn iter_entities(&self) -> impl Iterator<Item = &Entity> {
        self.entities.iter()
    }
}
