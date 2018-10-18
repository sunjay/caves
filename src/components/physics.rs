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
    /// The speed of the entity in px/frame
    pub speed: i32,
}

impl Default for Movement {
    fn default() -> Self {
        Self {
            direction: MovementDirection::East,
            speed: 0,
        }
    }
}

impl Movement {
    pub fn is_moving(&self) -> bool {
        self.speed != 0
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

impl MovementDirection {
    /// Returns a Point that represents the unit vector for a given direction
    pub fn to_vector(self) -> Point {
        use self::MovementDirection::*;
        match self {
            North => Point::new(0, -1),
            South => Point::new(0, 1),
            East => Point::new(1, 0),
            West => Point::new(-1, 0),
        }
    }
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
