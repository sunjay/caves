// The level generator is split across modules because there are a lot of methods (out of
// necessity) but many of those methods can be easily grouped because they do not actually interact
// with one another. These groups of methods usually correspond to the phases of level generation
// that take place. The code was designed this way to make sharing the configuration as easy as
// possible (via &self).
mod rooms;
mod passages;
mod special_tiles;

mod bounds;
pub use self::bounds::*;

use rand::{random, StdRng};

use texture_manager::TextureManager;
use map::*;

pub struct MapGenerator {
    /// The number of levels to generate
    pub levels: usize,
    /// The number of rows of tiles in the entire world (bound on the size of the map)
    pub rows: usize,
    /// The number of columns of tiles in the entire world (bound on the size of the map)
    pub cols: usize,
    /// The width and height of each tile in pixels
    pub tile_size: u32,
    /// The number of rooms to generate on each floor
    pub rooms: usize,
    /// The minimum and maximum width (in tiles) of a room
    pub room_cols: Bounds<usize>,
    /// The minimum and maximum height (in tiles) of a room
    pub room_rows: Bounds<usize>,
    /// The width of the passageways between rooms
    /// Used to calculate the minimum distance between adjacent rooms
    pub passage_size: usize,
    /// The width of the treasure chamber on the last level
    pub treasure_chamber_width: usize,
    /// The height of the treasure chamber on the last level
    pub treasure_chamber_height: usize,
    /// The number of doors to generate on each room
    pub doors: usize,
    /// The number of tiles that take you to the next level/prev level
    /// This will create `next_prev_tiles` number of ToNextLevel tiles and
    /// `next_prev_tiles` number of ToPrevLevel tiles
    pub next_prev_tiles: usize,
}

impl MapGenerator {
    pub fn generate(self, textures: &mut TextureManager) -> GameMap {
        self.generate_with_key(random(), textures)
    }

    pub fn generate_with_key(self, key: MapKey, textures: &mut TextureManager) -> GameMap {
        let mut rng = key.to_rng();
        let levels: Vec<_> = (1..=self.levels)
            .map(|level| self.generate_level(&mut rng, textures, level))
            .collect();

        let game_start = {
            let first_level = levels.first().expect("bug: should be at least one level");

            let level_start_room = first_level.rooms().find(|room| room.is_player_start())
            .expect("bug: should have had a player start level on the first level");
            // Start in the middle of the level start room
            let center = level_start_room.center_tile();
            Point::new(
                center.col as i32 * self.tile_size as i32,
                center.row as i32 * self.tile_size as i32,
            )
        };

        let level_boundary = Rect::new(
            0,
            0,
            self.cols as u32 * self.tile_size,
            self.rows as u32 * self.tile_size,
        );

        GameMap {
            key,
            current_level: 0,
            level_boundary,
            game_start,
            levels,
        }
    }

    fn generate_level(&self, rng: &mut StdRng, textures: &mut TextureManager, level: usize) -> FloorMap {
        let mut map = FloorMap::new(self.rows, self.cols);
        let rooms = self.generate_rooms(rng, &mut map, level);
        self.place_rooms(&mut map, &rooms);
        self.fill_passages(rng, &mut map);
        self.connect_rooms_passages(rng, &mut map, &rooms);
        self.reduce_dead_ends(&mut map);
        if level < self.levels {
            self.place_to_next_level_tiles(rng, &mut map, &rooms);
        }
        if level > 1 {
            self.place_to_prev_level_tiles(rng, &mut map, &rooms);
        }
        map
    }

    // NOTE: This impl block is only for the public interface of MapGenerator + some top-level
    // logic that has to do with that. As much as possible, we want to move level generation into
    // other submodules that each file stays small and understandable. Each submodule usually
    // corresponds to a single phase of level generation. The submodule methods do not typically
    // interact with methods from other submodules. This is a loose guideline, not a hard rule.
}
