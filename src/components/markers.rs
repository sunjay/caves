//! Components that act as markers for specific properties of entities

use specs::{Component, NullStorage};

/// The keyboard controlled player. Only one entity should hold this at a given time.
#[derive(Debug, Default, Component)]
#[storage(NullStorage)]
pub struct KeyboardControlled;

/// The entity with this component and a Position component will be centered in the camera
/// when the scene is rendered.
/// Only one entity should hold this at a given time.
#[derive(Debug, Default, Component)]
#[storage(NullStorage)]
pub struct CameraFocus;
