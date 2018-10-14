mod generator;
mod floor_map;
mod map_key;

use sdl2::rect::{Point, Rect};

pub use self::generator::*;
pub use self::floor_map::*;
pub use self::map_key::*;

#[derive(Debug, Clone)]
pub struct GameMap {
    key: MapKey,
    current_level: usize,
    level_boundary: Rect,
    levels: Vec<FloorMap>,
    game_start: Point,
}

impl GameMap {
    /// Returns the floor map of the current level
    pub fn current_level_map(&self) -> &FloorMap {
        &self.levels[self.current_level]
    }

    /// Returns the level boundary in pixels of the current map
    pub fn level_boundary(&self) -> Rect {
        self.level_boundary
    }

    /// Return the point that represents the start of the game. This point is always on the
    /// first level and the player should only be spawned at this point on the first level.
    pub fn game_start(&self) -> Point {
        self.game_start
    }
}
