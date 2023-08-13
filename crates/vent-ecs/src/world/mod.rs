use std::collections::HashMap;

/// The `World` struct represents the game world.
use crate::{archetype::Archetype, component::Component, entity::Entity};

pub struct World {
    entities: Vec<Entity>,
    next_entity: Entity,
    archetypes: HashMap<Vec<usize>, Archetype>,
    component_ids: HashMap<String, usize>,
}

impl World {
    pub fn new() -> Self {
        World {
            entities: Vec::new(),
            next_entity: 0,
            archetypes: HashMap::new(),
            component_ids: HashMap::new(),
        }
    }

    /// Creates a new entity in the world and returns its entity ID.
    pub fn create_entity(&mut self) -> Entity {
        let entity = self.next_entity;
        self.next_entity += 1;
        self.entities.push(entity);
        entity
    }

    /// Deletes an entity from the world.
    pub fn delete_entity(&mut self, entity: Entity) -> Result<(), String> {
        if let Some(index) = self.entities.iter().position(|&e| e == entity) {
            self.entities.swap_remove(index);
            self.archetypes.values_mut().for_each(|archetype| {
                archetype.remove_entity(entity);
            });
            Ok(())
        } else {
            Err(format!("Entity with ID {} does not exist", entity))
        }
    }

    /// Registers a component type in the world and returns its component ID.
    pub fn register_component<T: Component + 'static>(&mut self) -> usize {
        let component_name = std::any::type_name::<T>().to_owned();
        let component_id = self.component_ids.len();
        self.component_ids.insert(component_name, component_id);
        component_id
    }

    /// Adds a component to an entity in the world.
    pub fn add_component<T: Component + 'static>(
        &mut self,
        entity: Entity,
        component: T,
    ) -> Result<(), String> {
        let component_id = self
            .component_ids
            .get(&std::any::type_name::<T>().to_owned());
        if let Some(&component_id) = component_id {
            let archetype_key = vec![component_id];
            if let Some(archetype) = self.archetypes.get_mut(&archetype_key) {
                archetype.add_entity(entity);
                archetype.add_component(component_id, component);
                return Ok(());
            } else {
                let mut archetype = Archetype::new();
                archetype.add_entity(entity);
                archetype.add_component(component_id, component);
                self.archetypes.insert(archetype_key, archetype);
                return Ok(());
            }
        }
        Err(format!(
            "Component type not registered: {}",
            std::any::type_name::<T>()
        ))
    }

    /// Removes a component from an entity in the world.
    pub fn remove_component<T: Component + 'static>(
        &mut self,
        entity: Entity,
    ) -> Result<(), String> {
        let component_id = self
            .component_ids
            .get(&std::any::type_name::<T>().to_owned());
        if let Some(&component_id) = component_id {
            let archetype_key = vec![component_id];
            if let Some(archetype) = self.archetypes.get_mut(&archetype_key) {
                archetype.remove_entity(entity);
                archetype.remove_component(component_id, entity);
                return Ok(());
            }
        }
        Err(format!(
            "Component type not registered: {}",
            std::any::type_name::<T>()
        ))
    }

    /// Retrieves a component by its component ID and entity ID.
    pub fn get_component<T: Component + 'static>(&self, entity: Entity) -> Result<&T, String> {
        let component_id = self
            .component_ids
            .get(&std::any::type_name::<T>().to_owned());
        if let Some(&component_id) = component_id {
            let archetype_key = vec![component_id];
            if let Some(archetype) = self.archetypes.get(&archetype_key) {
                if let Some(component) = archetype.get_component::<T>(component_id, entity) {
                    return Ok(component);
                }
            }
        }
        Err(format!(
            "Component not found for entity ID {}: {}",
            entity,
            std::any::type_name::<T>()
        ))
    }

    /// Retrieves a mutable component by its component ID and entity ID.
    pub fn get_component_mut<T: Component + 'static>(
        &mut self,
        entity: Entity,
    ) -> Result<&mut T, String> {
        let component_id = self
            .component_ids
            .get(&std::any::type_name::<T>().to_owned());
        if let Some(&component_id) = component_id {
            let archetype_key = vec![component_id];
            if let Some(archetype) = self.archetypes.get_mut(&archetype_key) {
                if let Some(component) = archetype.get_component_mut::<T>(component_id, entity) {
                    return Ok(component);
                }
            }
        }
        Err(format!(
            "Component not found for entity ID {}: {}",
            entity,
            std::any::type_name::<T>()
        ))
    }

    /// Returns an iterator over the entities in the world.
    pub fn iter_entities(&self) -> impl Iterator<Item = &Entity> {
        self.entities.iter()
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
