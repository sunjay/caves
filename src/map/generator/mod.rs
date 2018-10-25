// The level generator is split across modules because there are a lot of methods (out of
// necessity) but many of those methods can be easily grouped because they do not actually interact
// with one another. These groups of methods usually correspond to the phases of level generation
// that take place. The code was designed this way to make sharing the configuration as easy as
// possible (via &self).
mod rooms;
mod place_items;
mod doorways;
mod validation;

mod bounds;
pub use self::bounds::*;

use rand::{random, StdRng, Rng, SeedableRng};

use map::*;

/// Represents when we have run out of attempts to generate the map from a given key
/// This can happen if a loop trying to generate something runs too many times
#[derive(Debug, Clone, Copy)]
struct RanOutOfAttempts;

pub struct MapGenerator {
    /// The number of attempts before giving up on placing something randomly
    ///
    /// After this many attempts, the level generator will give up and generate a new seed based
    /// on the same MapKey. This is still deterministic because we are only using the random
    /// number generator from the original key.
    ///
    /// A consequence of this is that we may end up with some keys (rarely) generating the same
    /// map. This is highly unlikely and not a huge deal at all.
    pub attempts: usize,
    /// The number of levels to generate
    pub levels: usize,
    /// The number of rows of tiles in the entire world (bound on the size of the map)
    pub rows: usize,
    /// The number of columns of tiles in the entire world (bound on the size of the map)
    pub cols: usize,
    /// The width and height of each tile in pixels
    pub tile_size: u32,
    /// The minimum and maximum number of rooms to generate on each floor
    pub rooms: Bounds<usize>,
    /// The minimum and maximum height (in tiles) of a room
    pub room_rows: Bounds<usize>,
    /// The minimum and maximum width (in tiles) of a room
    pub room_cols: Bounds<usize>,
    /// The maximum % that a room can overlap another room
    /// Value should be between 0.0 and 1.0
    pub max_overlap: f64,
    /// The min/max number of doors to give every room. Min must be at least 1 or some rooms will
    /// not be reachable.
    pub doors: Bounds<usize>,
    /// The number of tiles that take you to the next level/prev level
    /// This will create `next_prev_tiles` number of ToNextLevel tiles and
    /// `next_prev_tiles` number of ToPrevLevel tiles
    pub next_prev_tiles: usize,
}

impl MapGenerator {
    pub fn generate(self) -> GameMap {
        self.generate_with_key(random())
    }

    pub fn generate_with_key(self, key: MapKey) -> GameMap {
        let mut rng = key.to_rng();

        loop {
            let levels: Result<Vec<_>, _> = (1..=self.levels)
                .map(|level| self.generate_level(&mut rng, level))
                .collect();

            match levels {
                Ok(levels) => return GameMap {
                    key,
                    levels,
                    current_level: 0,
                    map_size: GridSize {rows: self.rows, cols: self.cols},
                    tile_size: self.tile_size,
                },
                // Reseed the rng using itself
                Err(RanOutOfAttempts) => {
                    rng = StdRng::from_seed(rng.gen());
                },
            }
        }
    }

    fn generate_level(&self, rng: &mut StdRng, level: usize) -> Result<FloorMap, RanOutOfAttempts> {
        let mut map = FloorMap::new(
            GridSize {rows: self.rows, cols: self.cols},
            self.tile_size,
        );

        // Levels are generated in "phases". The following calls runs each of those in succession.
        self.generate_rooms(rng, &mut map, level)?;
        println!("{:?}", map);
        self.connect_rooms(rng, &mut map);
        self.place_locks(rng, &mut map);
        //if level < self.levels {
        //    self.place_to_next_level_tiles(rng, &mut map)?;
        //}
        //if level > 1 {
        //    self.place_to_prev_level_tiles(rng, &mut map)?;
        //}

        self.validate_map(&map);
        Ok(map)
    }

    // NOTE: This impl block is only for the public interface of MapGenerator + some top-level
    // logic that has to do with that. As much as possible, we want to move level generation into
    // other submodules that each file stays small and understandable. Each submodule usually
    // corresponds to a single phase of level generation. The submodule methods do not typically
    // interact with methods from other submodules. This is a loose guideline, not a hard rule.
}
