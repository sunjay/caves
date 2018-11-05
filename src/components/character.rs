//! Components related to character specific properties

use specs::{Component, VecStorage, NullStorage};

/// Represents the amount of health left for a given entity
#[derive(Debug, Clone, Component)]
#[storage(VecStorage)]
pub struct HealthPoints(pub usize);

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
