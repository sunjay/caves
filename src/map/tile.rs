use super::{RoomId};
use assets::SpriteId;
use map_sprites::{MapSprites, FloorSprite, WallSprite, WallSpriteAlternate};

#[derive(Debug, Clone, PartialEq)]
pub enum Tile {
    /// Tiles that can be traversed
    Floor {
        room_id: RoomId,
        /// The floor sprite to use
        sprite: FloorSprite,
    },
    /// Tiles that cannot be traversed, not associated to a particular room
    Wall {
        // The wall sprite to use
        sprite: WallSprite,
    },
    /// A tile that cannot be traversed and has nothing on it
    Empty,
}

impl Tile {
    /// Creates a new floor tile with the given sprite
    pub fn new_floor(room_id: RoomId, sprite: FloorSprite) -> Self {
        Tile::Floor {room_id, sprite}
    }

    /// Creates a new wall tile with the given sprite
    pub fn new_wall(sprite: WallSprite) -> Self {
        Tile::Wall {sprite}
    }

    /// Creates a new empty tile
    pub fn empty() -> Self {
        Tile::Empty
    }

    /// Returns the sprite that should be used as the background of this tile
    pub fn background_sprite<'a>(&self, map_sprites: &'a MapSprites) -> &'a SpriteId {
        use self::Tile::*;
        match *self {
            Floor {sprite, ..} => &map_sprites.floor_sprite(sprite),
            Wall {sprite, ..} => &map_sprites.wall_sprite(sprite),
            Empty => map_sprites.empty_tile_sprite(),
        }
    }

    /// Sets the sprite to the given wall sprite only if the tile is a wall tile
    pub fn set_wall_sprite(&mut self, wall_sprite: WallSprite) {
        match self {
            Tile::Wall {sprite, ..} => *sprite = wall_sprite,
            _ => unreachable!("bug: cannot set a wall sprite for a non-wall tile"),
        }
    }

    /// Returns the wall sprite of this tile if and only if the tile is a wall tile
    pub fn wall_sprite(&self) -> &WallSprite {
        match self {
            Tile::Wall {sprite, ..} => sprite,
            _ => unreachable!("bug: cannot set a wall sprite for a non-wall tile"),
        }
    }

    /// Returns the wall sprite of this tile if and only if the tile is a wall tile
    pub fn wall_sprite_mut(&mut self) -> &mut WallSprite {
        match self {
            Tile::Wall {sprite, ..} => sprite,
            _ => unreachable!("bug: cannot set a wall sprite for a non-wall tile"),
        }
    }

    /// Sets the sprite to the given floor sprite only if the tile is a floor tile
    pub fn set_floor_sprite(&mut self, floor_sprite: FloorSprite) {
        match self {
            Tile::Floor {sprite, ..} => *sprite = floor_sprite,
            _ => unreachable!("bug: cannot set a floor sprite for a non-floor tile"),
        }
    }

    /// Returns the room ID of the tile if it is a floor tile or None if it is not
    pub fn floor_room_id(&self) -> Option<RoomId> {
        match self {
            &Tile::Floor {room_id, ..} => Some(room_id),
            _ => None,
        }
    }

    /// Returns the ID of the gate if the tile contains a ToNextLevel object
    pub fn to_next_level_id(&self) -> Option<usize> {
        match self.object() {
            Some(&TileObject::ToNextLevel {id, ..}) => Some(id),
            _ => None,
        }
    }

    /// Returns the ID of the gate if the tile contains a ToPrevLevel object
    pub fn to_prev_level_id(&self) -> Option<usize> {
        match self.object() {
            Some(&TileObject::ToPrevLevel {id, ..}) => Some(id),
            _ => None,
        }
    }

    /// Returns true if the tile contains a ToNextLevel gate with the given ID
    pub fn is_to_next_level_id(&self, id: usize) -> bool {
        match self.object() {
            Some(&TileObject::ToNextLevel {id: gid, ..}) => gid == id,
            _ => false,
        }
    }

    /// Returns true if the tile contains a ToPrevLevel gate with the given ID
    pub fn is_to_prev_level_id(&self, id: usize) -> bool {
        match self.object() {
            Some(&TileObject::ToPrevLevel {id: gid, ..}) => gid == id,
            _ => false,
        }
    }

    /// Returns true if the player is allowed to move over top of this tile
    pub fn is_traversable(&self) -> bool {
        match self {
            // Floor tiles are traversable by default unless their object is not traversable
            Tile::Floor {object, ..} => object.as_ref().map(|obj| obj.is_traversable()).unwrap_or(true),
            Tile::Wall {..} |
            Tile::Empty => false,
        }
    }

    /// Returns true if this tile is any floor tile
    pub fn is_floor(&self) -> bool {
        match self {
            Tile::Floor {..} => true,
            _ => false,
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
}
