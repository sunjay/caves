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
    levels: Vec<FloorMap>,
    current_level: usize,
    map_size: GridSize,
}

impl GameMap {
    /// Returns the floor map of the current level
    pub fn current_level_map(&self) -> &FloorMap {
        &self.levels[self.current_level]
    }

    /// Returns the level boundary in pixels of the current map
    pub fn level_boundary(&self) -> Rect {
        self.map_size.to_rect(self.current_level_map().tile_size())
    }

    /// Return the point that represents the start of the game. This point is always on the
    /// first level and the player should only be spawned at this point on the first level.
    pub fn game_start(&self) -> Point {
        let first_level = self.levels.first().expect("bug: should be at least one level");

        let (room_id, level_start_room) = first_level.rooms()
            .find(|(_, room)| room.is_player_start())
            .expect("bug: should have had a player start level on the first level");
        // Start in the middle of the level start room
        let center = level_start_room.boundary().center_tile();
        assert!(first_level.grid().get(center).is_room_floor(room_id),
            "bug: the center of the player start room was not a tile in that room");

        let tile_size = first_level.tile_size();
        // Start in the middle of the tile
        center.to_point(tile_size as i32).offset(tile_size as i32/2, tile_size as i32/2)
    }
}
