use std::fmt;

use super::{RoomId, FloorSprite, WallSprite, MapSprites, SpriteImage, TilePos};

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

/// Represents the direction that an object should face
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Orientation {
    FaceNorth,
    FaceEast,
    FaceSouth,
    FaceWest,
}

impl Orientation {
    /// Returns the orientation that would cause the tile at the given position to face `target`.
    /// The positions do not need to be adjacent, but they do need to be in the same row or column..
    pub fn face_target(pos: TilePos, target: TilePos) -> Self {
        match pos.difference(target) {
            (0, 0) => unreachable!("bug: cannot find orientation of a point facing itself"),
            (a, 0) => if a > 0 {
                // target is north of pos
                Orientation::FaceNorth
            } else {
                // target is south of pos
                Orientation::FaceSouth
            },
            (0, a) => if a > 0 {
                // target is west of pos
                Orientation::FaceWest
            } else {
                // target is east of pos
                Orientation::FaceEast
            },
            _ => unreachable!("bug: positions were not in the same row/column"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Tile {
    /// Tiles that can be traversed
    Floor {
        room_id: RoomId,
        ///
        object: Option<(TileObject, Orientation)>,
        /// The floor sprite to use
        sprite: FloorSprite,
    },
    /// Tiles that cannot be traversed, not associated to a particular room
    Wall {
        decoration: Option<WallDecoration>,
        // The wall sprite to use
        sprite: WallSprite,
    },
    /// A tile that cannot be traversed and has nothing on it
    Empty,
}

impl Tile {
    /// Creates a new floor tile with no object and the given sprite
    pub fn new_floor(room_id: RoomId, sprite: FloorSprite) -> Self {
        Tile::Floor {room_id, object: None, sprite}
    }

    /// Creates a new wall tile with no decoration and the given sprite
    pub fn new_wall(sprite: WallSprite) -> Self {
        Tile::Wall {decoration: None, sprite}
    }

    /// Creates a new empty tile
    pub fn empty() -> Self {
        Tile::Empty
    }

    /// Returns the sprite that should be used as the background of this tile
    pub fn background_sprite<'a>(&self, sprites: &'a MapSprites) -> &'a SpriteImage {
        use self::Tile::*;
        match *self {
            Floor {sprite, ..} => &sprites.floor_sprite(sprite),
            Wall {sprite, ..} => &sprites.wall_sprite(sprite),
            Empty => sprites.empty_tile_sprite(),
        }
    }

    /// Returns the sprite that should be drawn on top of the background of this sprite
    pub fn object_sprite<'a>(&self, sprites: &'a MapSprites) -> Option<&'a SpriteImage> {
        match self {
            &Tile::Floor {object: Some((ref object, orientation)), ..} => Some(match object {
                TileObject::ToNextLevel(_) => sprites.staircase_down_sprite(orientation),
                TileObject::ToPrevLevel(_) => sprites.staircase_up_sprite(orientation),
                _ => unimplemented!(),
            }),
            _ => None,
        }
    }

    /// Sets the sprite to the given wall sprite only if the tile is a wall tile
    pub(in map) fn set_wall_sprite(&mut self, wall_sprite: WallSprite) {
        use self::Tile::*;
        match self {
            Wall {sprite, ..} => *sprite = wall_sprite,
            _ => unreachable!("bug: cannot set a wall sprite for a non-wall tile"),
        }
    }

    /// Returns the room ID of the tile if it is a floor tile or None if it is not
    pub fn floor_room_id(&self) -> Option<RoomId> {
        match self {
            &Tile::Floor {room_id, ..} => Some(room_id),
            _ => None,
        }
    }

    /// Returns true if this tile is a floor tile from the given room
    pub fn is_room_floor(&self, id: RoomId) -> bool {
        match self {
            Tile::Floor {room_id, ..} if *room_id == id => true,
            _ => false,
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
    pub fn become_wall(&mut self, sprite: WallSprite) {
        *self = Self::new_wall(sprite);
    }

    /// Turns this tile into a Floor tile
    pub fn become_floor(&mut self, room_id: RoomId, sprite: FloorSprite) {
        *self = Self::new_floor(room_id, sprite);
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
    pub fn place_object(&mut self, object: TileObject, orientation: Orientation) {
        match self {
            // Ensure that we don't replace an object that was already placed by matching on None
            Tile::Floor {object: obj@None, ..} => *obj = Some((object, orientation)),
            Tile::Floor {..} => unreachable!("bug: attempt to place an object on a tile that already had an object"),
            _ => unreachable!("bug: attempt to place an object on a tile that does not support objects"),
        }
    }
}
