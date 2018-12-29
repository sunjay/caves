//! Components related to character specific properties

use component_group::ComponentGroup;

use specs::{Component, VecStorage, HashMapStorage, NullStorage};

/// All the components of a player. Grouped together so they can be easily copied to and from
/// worlds. The reason this struct exists is because specs doesn't provide a way to copy all the
/// components of one entity from one world to another. This is a less error-prone way of managing
/// that because Rust will tell you if you forget to provide a value for a field.
#[derive(Debug, ComponentGroup)]
pub struct PlayerComponents {
    pub keyboard_controlled: KeyboardControlled,
    pub camera_focus: CameraFocus,
    pub player: Player,
    pub health_points: HealthPoints,
    pub position: super::Position,
    pub bounding_box: super::BoundingBox,
    pub movement: super::Movement,
    pub sprite: super::Sprite,
    pub animation: super::Animation,
    pub animation_manager: super::AnimationManager,
}

/// Represents the amount of health left for a given entity
#[derive(Debug, Clone, Component)]
#[storage(VecStorage)]
pub struct HealthPoints(pub usize); // unit: HP

/// Represents the strength of this entity's attack
#[derive(Debug, Clone, Component)]
#[storage(VecStorage)]
pub struct Attack(pub usize); // unit: HP

/// Represents the amount of time (if at all) that the entity waits when hit before being able to
/// move/attack again
#[derive(Debug, Clone, Component)]
#[storage(VecStorage)]
pub struct HitWait(pub usize); // unit: frames

/// The keyboard controlled player. Only one entity should hold this at a given time.
#[derive(Debug, Clone, Copy, Default, Component)]
#[storage(NullStorage)]
pub struct KeyboardControlled;

/// The entity with this component and a Position component will be centered in the camera
/// when the scene is rendered.
/// Only one entity should hold this at a given time.
#[derive(Debug, Clone, Copy, Default, Component)]
#[storage(NullStorage)]
pub struct CameraFocus;

/// Entities with this component will be attacked by entities with the Enemy component
#[derive(Debug, Clone, Copy, Default, Component)]
#[storage(NullStorage)]
pub struct Player;

/// Entities with this component will attempt to attack entities with the Player component
#[derive(Debug, Default, Component)]
#[storage(HashMapStorage)]
pub struct Enemy {
    pub speed: i32, // movements per second
}
