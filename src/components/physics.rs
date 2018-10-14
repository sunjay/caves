//! Components for physics-related uses

use specs::{VecStorage, HashMapStorage};
use sdl2::rect::Point;

/// Represents the XY world coordinates of the center of an entity.
///
/// This is distinct from the screen coordinates which are bounded by the size of the display.
///
/// Not to be modified outside of the physics system.
#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Position(pub Point);

/// Represents the direction of movement that a given entity would like to move in
///
/// Used in the physics system to update position every frame
#[derive(Debug, Component)]
#[storage(HashMapStorage)]
pub struct Movement {
    /// The most recent direction that the entity was moving in
    pub direction: MovementDirection,
    /// This is true if the entity should move in the given direction
    pub is_moving: bool,
}

impl Default for Movement {
    fn default() -> Self {
        Self {
            direction: MovementDirection::East,
            is_moving: false,
        }
    }
}

impl Movement {
    pub fn move_in_direction(&mut self, direction: MovementDirection) {
        self.is_moving = true;
        self.direction = direction;
    }
}

/// Represents the direction that an entity would like to move in
///
/// This may not always be possible if there is no way to move further in a given direction (e.g.
/// because of something in the way or a wall or something)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementDirection {
    North,
    South,
    East,
    West,
}

/// Represents the bounding box centered around an entity's position. BoundingBox alone doesn't
/// mean much without a Position also attached to the entity.
///
/// Modifying this after it is initially set is currently NOT supported.
#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct BoundingBox {
    pub width: u32,
    pub height: u32,
}
