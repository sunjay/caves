use std::fmt;

use super::{TilePos};

#[derive(Debug, Clone)]
pub enum Item {
    TreasureKey,
    RoomKey,
    Potion {stength: u32},
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Door {
    /// Door is open and can be passed through
    Open,
    /// Door cannot be passed through and requires a RoomKey to be opened
    Locked,
}

/// Represents the direction that the stairway should face
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

#[derive(Debug, Clone)]
pub enum Chest {
    Item(Item),
    Opened,
}

/// The object or item placed at a particular tile
#[derive(Debug, Clone)]
pub enum TileObject {
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
    /// A door that is either locked or open (can be opened with a RoomKey)
    Door(Door),
    /// A gate that can not be opened without some external event (e.g. switch, challenge room, etc.)
    Gate(Door),
    /// A chest containing an item that can be collected
    Chest(Chest),
}

impl fmt::Display for TileObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::TileObject::*;
        write!(f, "{}", match *self {
            ToNextLevel {..} => "\u{2193}",
            ToPrevLevel {..} => "\u{2191}",
            Door(self::Door::Locked) => "\u{1F510}",
            Door(self::Door::Open) => "\u{1F513}",
            Gate(self::Door::Locked) => "\u{1F512}",
            Gate(self::Door::Open) => "\u{1F513}",
            Chest(_) => "$",
        })
    }
}

impl TileObject {
    /// Returns true if the player is allowed to move over top of this object
    pub fn is_traversable(&self) -> bool {
        use self::TileObject::*;
        match self {
            ToNextLevel {..} |
            ToPrevLevel {..} |
            Door(self::Door::Open) |
            Gate(self::Door::Open) => true,
            Door(self::Door::Locked) |
            Gate(self::Door::Locked) |
            Chest(_) => false,
        }
    }
}
