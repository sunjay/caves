use super::{
    RoomId,
    FloorSprite,
    WallSprite,
    WallSpriteAlternate,
    TorchAnimation,
    MapSprites,
    SpriteImage,
    TileObject,
};

#[derive(Debug, Clone)]
pub enum WallDecoration {
    Torch(TorchAnimation),
    //TODO: Enemy spawn, arrow shooter, portal, spikes, etc.
}

impl PartialEq for WallDecoration {
    fn eq(&self, other: &Self) -> bool {
        use self::WallDecoration::*;
        match (self, other) {
            // Animation state does not matter for equality. This is important for the test that
            // ensures that map generation is deterministic. With this, we can randomly vary the
            // torch animation while still allowing two maps to count as the same.
            (Torch(_), Torch(_)) => true,
        }
    }
}

impl WallDecoration {
    pub fn torch() -> Self {
        WallDecoration::Torch(TorchAnimation::default())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tile {
    /// Tiles that can be traversed
    Floor {
        room_id: RoomId,
        ///
        object: Option<TileObject>,
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
    pub fn foreground_sprite<'a>(&self, sprites: &'a MapSprites) -> Option<&'a SpriteImage> {
        match self {
            Tile::Floor {object: Some(object), ..} => match object {
                &TileObject::ToNextLevel {direction, ..} => Some(sprites.staircase_down_sprite(direction)),
                &TileObject::ToPrevLevel {direction, ..} => Some(sprites.staircase_up_sprite(direction)),
                &TileObject::Door {state, orientation} => sprites.door_sprite(state, orientation),
                _ => unimplemented!(),
            },
            Tile::Wall {decoration: Some(decoration), ..} => match decoration {
                WallDecoration::Torch(animation) => Some(sprites.torch_sprite(animation.current_step())),
            },
            _ => None,
        }
    }

    /// Sets the sprite to the given wall sprite only if the tile is a wall tile
    pub(in map) fn set_wall_sprite(&mut self, wall_sprite: WallSprite) {
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

    /// Sets the sprite to the given floor sprite only if the tile is a floor tile
    pub(in map) fn set_floor_sprite(&mut self, floor_sprite: FloorSprite) {
        match self {
            Tile::Floor {sprite, ..} => *sprite = floor_sprite,
            _ => unreachable!("bug: cannot set a floor sprite for a non-floor tile"),
        }
    }

    /// Sets the wall decoration to a torch. Panics if this is not a wall tile.
    pub fn place_wall_torch(&mut self) {
        match self {
            Tile::Wall {decoration, sprite} => {
                *decoration = Some(WallDecoration::torch());
                sprite.alt = WallSpriteAlternate::TorchLit;
            },
            _ => unreachable!("bug: cannot set wall decoration on non-wall tile"),
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

    /// Returns the object on this tile (if there is any)
    pub fn object(&self) -> Option<&TileObject> {
        match self {
            Tile::Floor {object, ..} => object.as_ref(),
            _ => None,
        }
    }

    /// Returns the object on this tile (if there is any)
    pub fn object_mut(&mut self) -> Option<&mut TileObject> {
        match self {
            Tile::Floor {object, ..} => object.as_mut(),
            _ => None,
        }
    }

    /// Returns true if this tile has an object
    pub fn has_object(&self) -> bool {
        self.object().is_some()
    }

    /// Returns true if this tile has a staircase
    pub fn has_staircase(&self) -> bool {
        match self {
            Tile::Floor {object: Some(TileObject::ToNextLevel {..}), ..} |
            Tile::Floor {object: Some(TileObject::ToPrevLevel {..}), ..} => true,
            _ => false,
        }
    }

    /// Returns true if this tile has a doorway or a gate
    pub fn has_entrance(&self) -> bool {
        match self {
            Tile::Floor {object: Some(TileObject::Door {..}), ..} |
            Tile::Floor {object: Some(TileObject::Gate {..}), ..} => true,
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

    /// Advances the animation of this tile by the given number of frames.
    /// If this tile is not animated, does nothing.
    pub fn advance_animation(&mut self, frames: usize) {
        use self::WallDecoration::*;
        match self {
            Tile::Wall {decoration: Some(Torch(animation)), ..} => {
                animation.advance(frames);
            },
            _ => {},
        }
    }
}
