//! Components related to character specific properties

use specs::{Component, VecStorage, NullStorage};

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

/// Entities with this component will be attacked by entities with the Enemy component
#[derive(Debug, Default, Component)]
#[storage(NullStorage)]
pub struct Player;

/// Entities with this component will attempt to attack entities with the Player component
#[derive(Debug, Default, Component)]
#[storage(NullStorage)]
pub struct Enemy;
