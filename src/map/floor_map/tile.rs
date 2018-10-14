use std::fmt;

use sdl2::rect::Rect;

use texture_manager::TextureId;
use super::{RoomId, TileWalls};

#[derive(Debug, Clone)]
pub enum Item {
    TreasureKey,
    RoomKey,
    Potion {stength: u32},
}

#[derive(Debug, Clone)]
pub enum TileType {
    /// Tiles that can be used to pass between rooms
    Passageway,
    /// Tiles that are part of a given room
    Room(RoomId),
}

/// The object or item placed at a particular tile
#[derive(Debug, Clone)]
pub enum TileObject {
    /// Stepping on this tile transports you to the next level
    /// Field is the ID of this gate and the ID of the ToPrevLevel tile that this should connect to
    ToNextLevel(usize),
    /// Stepping on this tile transports you to the previous level
    /// Field is the ID of this gate and the ID of the ToNextLevel tile that this should connect to
    ToPrevLevel(usize),
    /// A point where an enemy *may* spawn
    EnemySpawn {
        /// Probability that an enemy will spawn here: 1.0 means that the enemy will definitely
        /// spawn and 0.0 means that an enemy will not spawn
        probability: f64,
    },
    Chest(Item),
}

impl fmt::Display for TileObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::TileObject::*;
        write!(f, "{}", match *self {
            ToNextLevel(_) => "\u{2193}",
            ToPrevLevel(_) => "\u{2191}",
            _ => " ",
        })
    }
}

/// Represents an image/texture that will be renderered
///
/// The convention is that the sprite begins pointing to the right and flipping it horizontally
/// results in it facing left
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpriteImage {
    /// The spritesheet to pull the image from
    pub texture_id: TextureId,
    /// The region of the spritesheet to use, unrelated to the actual bounding box
    pub region: Rect,
    /// Whether to flip the sprite along the horizontal axis
    pub flip_horizontal: bool,
    /// Whether to flip the sprite along the vertical axis
    pub flip_vertical: bool,
}

impl SpriteImage {
    /// Creates a new SpriteImage that is not flipped either horizontally or vertically
    pub fn new_unflipped(texture_id: TextureId, region: Rect) -> Self {
        SpriteImage {
            texture_id,
            region,
            flip_horizontal: false,
            flip_vertical: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tile {
    pub ttype: TileType,
    pub sprite: SpriteImage,
    pub object: Option<TileObject>,
    pub walls: TileWalls,
}

impl Tile {
    pub(in super) fn with_type(ttype: TileType, sprite: SpriteImage) -> Self {
        Self {
            ttype,
            sprite,
            object: Default::default(),
            walls: Default::default(),
        }
    }

    pub fn has_object(&self) -> bool {
        self.object.is_some()
    }

    pub fn place_object(&mut self, object: TileObject) {
        self.object = Some(object);
    }
}
