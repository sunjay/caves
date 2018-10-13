mod generator;
mod floor_map;
mod map_key;

use sdl2::rect::Point;

pub use self::generator::*;
pub use self::floor_map::*;
pub use self::map_key::*;

#[derive(Debug, Clone)]
pub struct GameMap {
    key: MapKey,
    levels: Vec<FloorMap>,
}

impl GameMap {
    /// Return the point that represents the start of the game. This point is always on the
    /// first level and the player should only be spawned at this point on the first level.
    pub fn game_start(&self) -> Point {
        let first_level = self.levels.first().expect("bug: should be at least one level");

        let level_start_room = first_level.rooms().find(|room| room.is_player_start())
            .expect("bug: should have had a player start level on the first level");
        // Start in the middle of the level start room
        level_start_room.center()
    }
}
