use std::fmt;

use super::{RoomId, SpriteTable, SpriteImage};

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

/// The object or item placed at a particular tile
#[derive(Debug, Clone)]
pub enum TileObject {
    /// Stepping on this tile transports you to the next level
    /// Field is the ID of this gate and the ID of the ToPrevLevel tile that this should connect to
    ToNextLevel(usize),
    /// Stepping on this tile transports you to the previous level
    /// Field is the ID of this gate and the ID of the ToNextLevel tile that this should connect to
    ToPrevLevel(usize),
    /// A door that is either locked or open (can be opened with a RoomKey)
    Door(Door),
    /// A gate that can not be opened without some external event (e.g. switch, challenge room, etc.)
    Gate(Door),
    /// A chest containing an item that can be collected
    /// None - means object has been collected
    Chest(Option<Item>),
}

impl fmt::Display for TileObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::TileObject::*;
        write!(f, "{}", match *self {
            ToNextLevel(_) => "\u{2193}",
            ToPrevLevel(_) => "\u{2191}",
            Door(self::Door::Locked) => "\u{1F510}",
            Door(self::Door::Open) => "\u{1F513}",
            Gate(self::Door::Locked) => "\u{1F512}",
            Gate(self::Door::Open) => "\u{1F513}",
            Chest(_) => "$",
        })
    }
}

#[derive(Debug, Clone)]
pub enum WallDecoration {
    Torch,
    //TODO: Enemy spawn, arrow shooter, portal, spikes, etc.
}

#[derive(Debug, Clone)]
pub enum Tile {
    /// Tiles that can be traversed
    Floor {
        room_id: RoomId,
        object: Option<TileObject>,
        // The index of the room sprite to use in the sprite table
        room_sprite_index: usize,
    },
    /// Tiles that cannot be traversed, not associated to a particular room
    Wall {
        decoration: Option<WallDecoration>,
        // The index of the wall sprite to use in the sprite table
        wall_sprite_index: usize,
    },
    /// A tile that cannot be traversed and has nothing on it
    Empty,
}

impl Tile {
    /// Creates a new floor tile with no object and the given sprite
    pub fn new_floor(room_id: RoomId, room_sprite_index: usize) -> Self {
        Tile::Floor {room_id, object: None, room_sprite_index}
    }

    /// Creates a new wall tile with no decoration and the given sprite
    pub fn new_wall(wall_sprite_index: usize) -> Self {
        Tile::Wall {decoration: None, wall_sprite_index}
    }

    /// Creates a new empty tile
    pub fn empty() -> Self {
        Tile::Empty
    }

    /// Returns the sprite that should be used as the background of this tile
    pub fn background_sprite<'a>(&self, sprites: &'a SpriteTable) -> &'a SpriteImage {
        use self::Tile::*;
        match *self {
            Floor {room_sprite_index, ..} => &sprites.floor_tiles[room_sprite_index],
            Wall {wall_sprite_index, ..} => &sprites.wall_tiles[wall_sprite_index],
            Empty => &sprites.empty_tile_sprite,
        }
    }

    /// Returns the sprite that should be drawn on top of the background of this sprite
    pub fn object_sprite<'a>(&self, sprites: &'a SpriteTable) -> Option<&'a SpriteImage> {
        match self {
            Tile::Floor {object: Some(object), ..} => unimplemented!(),
            _ => None,
        }
    }

    /// Returns true if this tile is a floor tile from the given room
    pub fn is_room_floor(&self, id: RoomId) -> bool {
        match self {
            Tile::Floor {room_id, ..} if *room_id == id => true,
            _ => false
        }
    }

    /// Returns true if this tile is a wall
    pub fn is_wall(&self) -> bool {
        match self {
            Tile::Wall {..} => true,
            _ => false
        }
    }

    /// Returns true if this tile is empty
    pub fn is_empty(&self) -> bool {
        match self {
            Tile::Empty => true,
            _ => false
        }
    }

    /// Turns this tile into a Wall tile
    pub fn become_wall(&mut self, wall_sprite_index: usize) {
        *self = Self::new_wall(wall_sprite_index);
    }

    /// Turns this tile into a Floor tile
    pub fn become_floor(&mut self, room_id: RoomId, room_sprite_index: usize) {
        *self = Self::new_floor(room_id, room_sprite_index);
    }

    /// Returns true if this tile has an object
    pub fn has_object(&self) -> bool {
        match self {
            Tile::Floor {object, ..} => object.is_some(),
            _ => false,
        }
    }

    /// Attempts to place an object on this tile. Panics if this is not possible for this type of
    /// tile.
    pub fn place_object(&mut self, object: TileObject) {
        match self {
            // Ensure that we don't replace an object that was already placed by matching on None
            Tile::Floor {object: obj@None, ..} => *obj = Some(object),
            Tile::Floor {..} => unreachable!("bug: attempt to place an object on a tile that already had an object"),
            _ => unreachable!("bug: attempt to place an object on a tile that does not support objects"),
        }
    }
}
