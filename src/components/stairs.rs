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
        /// ID of these stairs and the ID of the ToPrevLevel tile that this should connect to
        id: usize,
    },
    /// Stepping on this tile transports you to the previous level
    ToPrevLevel {
        /// ID of these stairs and the ID of the ToNextLevel tile that this should connect to
        id: usize,
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
