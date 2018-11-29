use sdl2::rect::Point;
use specs::World;

use map::FloorMap;
use generator::MapKey;

#[derive(Debug, Clone)]
pub struct Game {
    key: MapKey,
    levels: Vec<World>,
    current_level: usize,
}

impl Game {
    /// Returns the current level
    pub fn current_level(&self) -> &World {
        &self.levels[self.current_level]
    }

    /// Advances to the next level. Panics if there is no next level
    pub fn to_next_level(&mut self) {
        self.current_level += 1;
        assert!(self.current_level < self.levels.len(), "bug: advanced too many levels");
    }

    /// Goes back to the previous level. Panics if there is no previous level.
    pub fn to_prev_level(&mut self) {
        self.current_level = self.current_level.checked_sub(1)
            .expect("bug: went back too many levels");
    }

    /// Returns an iterator of the game levels
    pub fn levels(&self) -> impl Iterator<Item=&FloorMap> {
        self.levels.iter()
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
