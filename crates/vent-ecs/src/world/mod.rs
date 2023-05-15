use crate::component::{Component, Entity};
use std::any::Any;

/// The `World` struct represents the game world.
#[derive(Default)]
pub struct World {
    entities: Vec<Entity>,
    next_entity: Entity,
    components: Vec<Box<dyn Any>>,
}

impl World {
    /// Creates a new entity in the world and returns its entity ID.
    pub fn create_entity(&mut self) -> Entity {
        let entity = self.next_entity;
        self.next_entity += 1;
        self.entities.push(entity);
        entity
    }

    /// Deletes an entity from the world.
    pub fn delete_entity(&mut self, entity: Entity) {
        let index = self.entities.iter().position(|&e| e == entity).unwrap();
        self.entities.swap_remove(index);
        for component in &mut self.components {
            component
                .downcast_mut::<Component<Entity>>()
                .unwrap()
                .remove(entity);
        }
    }

    /// Registers a component type in the world and returns its component ID.
    pub fn register_component<T: 'static>(&mut self) -> usize {
        self.components.push(Box::<Component<T>>::default());
        self.components.len() - 1
    }

    /// Retrieves a component by its component ID.
    pub fn get_component<T: 'static>(&self, component_id: usize) -> &Component<T> {
        self.components[component_id]
            .downcast_ref::<Component<T>>()
            .unwrap()
    }

    /// Retrieves a mutable reference to a component by its component ID.
    pub fn get_component_mut<T: 'static>(&mut self, component_id: usize) -> &mut Component<T> {
        self.components[component_id]
            .downcast_mut::<Component<T>>()
            .unwrap()
    }

    /// Returns an iterator over the entities in the world.
    pub fn iter_entities(&self) -> impl Iterator<Item = &Entity> {
        self.entities.iter()
    }

    /// Returns an iterator over the components of type `T` in the world.
    pub fn iter_components<'a, T: 'static>(&'a self) -> impl Iterator<Item = &'a Component<T>> {
        self.components
            .iter()
            .filter_map(|component| component.downcast_ref::<Component<T>>())
    }

    /// Returns an iterator over the mutable components of type `T` in the world.
    pub fn iter_components_mut<'a, T: 'static>(
        &'a mut self,
    ) -> impl Iterator<Item = &'a mut Component<T>> {
        self.components
            .iter_mut()
            .filter_map(|component| component.downcast_mut::<Component<T>>())
    }

    /// Returns an iterator over tuples of components of types `T` and `U` in the world.
    pub fn iter_component_tuples<'a, T: 'static, U: 'static>(
        &'a self,
    ) -> impl Iterator<Item = (&'a Component<T>, &'a Component<U>)> {
        let iter1 = self.iter_components::<T>();
        let iter2 = self.iter_components::<U>();
        iter1.zip(iter2)
    }
}
