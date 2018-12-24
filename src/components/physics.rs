//! Components for physics-related uses

use specs::{Component, VecStorage, HashMapStorage, NullStorage};
use sdl2::rect::{Point, Rect};

/// An entity with this component does not count in collisions and can thus be rendered over
#[derive(Debug, Default, Component)]
#[storage(NullStorage)]
pub struct Ghost;

/// Represents the XY world coordinates of the center of an entity.
///
/// This is distinct from the screen coordinates which are bounded by the size of the display.
///
/// Not to be modified outside of the physics system.
#[derive(Debug, Clone, Component)]
#[storage(VecStorage)]
pub struct Position(pub Point);

/// Represents the direction of movement that a given entity would like to move in
///
/// Used in the physics system to update position every frame
#[derive(Debug, Clone, Component)]
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
#[derive(Debug, Clone, Copy, Component)]
#[storage(VecStorage)]
pub enum BoundingBox {
    /// A full bounding box centered around the entity's position
    Full {
        width: u32,
        height: u32,
    },
    /// A "half" bounding box where the position is the top-middle of the box formed by the given
    /// width and height
    BottomHalf {
        width: u32,
        height: u32,
    },
}

impl BoundingBox {
    /// Shrink the horizontal and vertical size of this bounding box by the given amount centering
    /// the transformation around the reference position. That means that for full bounding boxes
    /// this will shift all four sides inward. For bottom half bounding boxes this will only shift
    /// the left, right, and bottom sides since the top side is at the position already.
    pub fn shrink(self, value: u32) -> Self {
        use self::BoundingBox::*;
        match self {
            Full {width, height} => Full {
                width: width - value * 2,
                height: height - value * 2,
            },
            BottomHalf {width, height} => BottomHalf {
                width: width - value * 2,
                height: height - value,
            },
        }
    }

    /// Given the position of the center of an entity, returns the rectangle that represents the
    /// boundary of the bounding box. The position is interpreted differently depending on the type
    /// of the bounding box.
    pub fn to_rect(self, pos: Point) -> Rect {
        use self::BoundingBox::*;
        match self {
            Full {width, height} => Rect::from_center(pos, width, height),
            BottomHalf {width, height} => Rect::from_center(
                // Make pos be at the top middle of the bounding box
                pos.offset(0, height as i32/2),
                width,
                height
            ),
        }
    }

    /// Treat this bounding box as a full bounding box and return its boundary rectangle as if that
    /// was the case.
    pub fn to_full_rect(self, pos: Point) -> Rect {
        use self::BoundingBox::*;
        match self {
            Full {width, height} => Rect::from_center(pos, width, height),
            BottomHalf {width, height} => Rect::from_center(pos, width, height * 2),
        }
    }
}
