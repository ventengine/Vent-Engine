use std::any::Any;

mod input_component;

/// The `Component` trait represents a component in an ECS.
pub trait Component: Any + 'static {}

// Soon ...
