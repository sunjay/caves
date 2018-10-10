mod generator;

use std::fmt;

use sdl2::rect::{Rect};

pub use self::generator::*;

#[derive(Debug, Clone)]
pub enum Item {
    TreasureKey,
    RoomKey,
    Potion {stength: u32},
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoomId(usize);

impl fmt::Display for RoomId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub enum TileType {
    Empty,
    Room(RoomId),
}

impl Default for TileType {
    fn default() -> Self {
        TileType::Empty
    }
}

/// The object or item placed at a particular tile
#[derive(Debug, Clone)]
pub enum TileObject {
    /// Stairs up to the next level
    StairsUp,
    /// Stairs down to the previous level
    StairsDown,
    /// A point where an enemy *may* spawn
    EnemySpawn {
        /// Probability that an enemy will spawn here: 1.0 means that the enemy will definitely
        /// spawn and 0.0 means that an enemy will not spawn
        probability: f64,
    },
    Chest(Item),
}

#[derive(Debug, Clone, Default)]
pub struct Tile {
    ttype: TileType,
    object: Option<TileObject>,
}

impl Tile {
    pub fn is_empty(&self) -> bool {
        match self.ttype {
            TileType::Empty => true,
            _ => false,
        }
    }

    pub fn set_room(&mut self, room_id: RoomId) {
        self.ttype = TileType::Room(room_id);
    }
}

pub type TileGrid = Vec<Vec<Tile>>;

#[derive(Clone)]
pub struct FloorMap {
    level: usize,
    tiles: TileGrid,
}

impl fmt::Debug for FloorMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in &self.tiles {
            for tile in row {
                match tile.ttype {
                    TileType::Empty => write!(f, "_")?,
                    TileType::Room(id) => write!(f, "{}", id)?,
                }
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

impl FloorMap {
    pub fn generate() -> Self {
        Self {
            level: 1, //TODO
            tiles: Vec::new(),
        }
    }

    pub fn level_boundary(&self) -> Rect {
        Rect::new(0, 0, self.tiles[0].len() as u32, self.tiles.len() as u32)
    }
}

#[derive(Debug, Clone)]
pub struct GameMap {
    key: MapKey,
    levels: Vec<FloorMap>,
}
