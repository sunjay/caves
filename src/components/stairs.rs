//! Props on the map

use std::fmt;

use specs::{Component, HashMapStorage};

use map::TilePos;

/// A staircase to the next level or to the previous level
#[derive(Debug, Component)]
#[storage(HashMapStorage)]
pub enum Stairs {
    /// Stepping on this tile transports you to the next level
    ToNextLevel {
        /// ID of this gate and the ID of the ToPrevLevel tile that this should connect to
        id: usize,
        direction: StairsDirection,
    },
    /// Stepping on this tile transports you to the previous level
    ToPrevLevel {
        /// ID of this gate and the ID of the ToNextLevel tile that this should connect to
        id: usize,
        direction: StairsDirection,
    },
}

impl fmt::Display for Stairs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Stairs::*;
        write!(f, "{}", match *self {
            ToNextLevel {..} => "\u{2193}",
            ToPrevLevel {..} => "\u{2191}",
        })
    }
}

/// The direction that the stairway should face
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StairsDirection {
    Left,
    Right,
}

impl StairsDirection {
    /// Returns the orientation that would cause the tile at the given position to face `target`.
    /// The positions do not need to be adjacent, but they do need to be in the same row or column..
    pub fn towards_target(pos: TilePos, target: TilePos) -> Self {
        match pos.difference(target) {
            (0, 0) => unreachable!("bug: a position cannot face itself"),
            (0, a) if a > 0 => StairsDirection::Left,
            (0, a) if a < 0 => StairsDirection::Right,
            _ => unreachable!("bug: stairs only support facing left or right"),
        }
    }
}
