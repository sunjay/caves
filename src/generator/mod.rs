// The level generator is split across modules because there are a lot of methods (out of
// necessity) but many of those methods can be easily grouped because they do not actually interact
// with one another. These groups of methods usually correspond to the phases of level generation
// that take place. The code was designed this way to make sharing the configuration as easy as
// possible (via &self).
mod rooms;
mod sprite_patterns;
mod place_items;
mod doorways;

mod map_key;
mod bounds;
mod world_helpers;

pub use self::map_key::*;
pub use self::bounds::*;

use rand::{random, rngs::StdRng, Rng, SeedableRng};
use specs::{World, Dispatcher};
use sdl2::rect::Point;
use rayon::prelude::*;

use map::*;
use map_sprites::MapSprites;

pub struct GenLevel<'a, 'b> {
    pub world: World,
    pub dispatcher: Dispatcher<'a, 'b>,
}

impl<'a, 'b> GenLevel<'a, 'b> {
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }
}

pub struct GenGame<'a, 'b> {
    pub key: MapKey,
    pub levels: Vec<GenLevel<'a, 'b>>,
    /// The point that the player spawns at when the game begins. This point is only valid on the
    /// first level and the player should only be spawned at this point on the first level.
    pub player_start: Point,
}

fn find_player_start<'a, 'b>(levels: &[GenLevel<'a, 'b>]) -> Point {
    let first_level = levels.first().expect("bug: should be at least one level");
    let map = first_level.world.read_resource::<FloorMap>();

    let (room_id, level_start_room) = map.rooms()
        .find(|(_, room)| room.is_player_start())
        .expect("bug: should have had a player start room on the first level");
    // Start in the middle of the level start room
    let center = level_start_room.boundary().center_tile();
    assert!(map.grid().get(center).is_room_floor(room_id),
        "bug: the center of the player start room was not a tile in that room");

    let tile_size = map.tile_size() as i32;
    // Start in the middle of the tile
    center.top_left(tile_size).offset(tile_size/2, tile_size/2)
}

impl<'a, 'b> GenGame<'a, 'b> {
    fn new(key: MapKey, levels: Vec<GenLevel<'a, 'b>>) -> Self {
        // Calculate the player start position
        let player_start = find_player_start(&levels);
        GenGame {key, levels, player_start}
    }
}

/// Represents when we have run out of attempts to generate the map from a given key
/// This can happen if a loop trying to generate something runs too many times
#[derive(Debug, Clone, Copy)]
struct RanOutOfAttempts;

#[derive(Clone)]
pub struct GameGenerator<'a> {
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
    /// Sprites from the spritesheet
    pub sprites: &'a MapSprites,
}

impl<'a> GameGenerator<'a> {
    pub fn generate<'b, 'c>(self, setup_world: impl Fn() -> (Dispatcher<'b, 'c>, World)) -> GenGame<'b, 'c> {
        self.generate_with_key(random(), setup_world)
    }

    pub fn generate_with_key<'b, 'c>(self, key: MapKey, setup_world: impl Fn() -> (Dispatcher<'b, 'c>, World)) -> GenGame<'b, 'c> {
        let mut rng = key.to_rng();

        // If this takes more than 10 attempts, we can conclude that it was essentially impossible
        // to generate the map.
        for _ in 0..10 {
            let (rngs_worlds, dispatchers): (Vec<_>, Vec<_>) = (1..=self.levels).map(|level| {
                let (dispatcher, world) = setup_world();
                ((self.clone(), level, StdRng::from_seed(rng.gen()), world), dispatcher)
            }).unzip();
            let levels: Result<Vec<_>, _> = rngs_worlds.into_par_iter()
                .map(|(generator, level, mut rng, world)| generator.populate_level(&mut rng, level, world))
                .collect();
            let levels = levels.map(|levels| levels.into_iter()
                .zip(dispatchers.into_iter())
                .map(|(world, dispatcher)| GenLevel {world, dispatcher})
                .collect());

            match levels {
                Ok(levels) => return GenGame::new(key, levels),
                // Reseed the rng using itself
                Err(RanOutOfAttempts) => {
                    rng = StdRng::from_seed(rng.gen());
                },
            }
        }

        panic!("Never succeeded in generating a map with key `{}`!", key);
    }

    fn populate_level(&self, rng: &mut StdRng, level: usize, mut world: World) -> Result<World, RanOutOfAttempts> {
        // Levels are generated in "phases". The following calls runs each of those in succession.
        let mut map = FloorMap::new(
            GridSize {rows: self.rows, cols: self.cols},
            self.tile_size,
        );

        self.generate_rooms(rng, &mut map, level)?;

        self.connect_rooms(rng, &mut map, &mut world);

        if level < self.levels {
            self.place_to_next_level_tiles(rng, &mut map, &mut world)?;
        }
        if level > 1 {
            self.place_to_prev_level_tiles(rng, &mut map, &mut world)?;
        }

        self.layout_floor_wall_sprites(rng, &mut map);
        self.layout_wall_torch_sprites(&mut map, &mut world);

        world.add_resource(map);
        Ok(world)
    }

    // NOTE: This impl block is only for the public interface of GameGenerator + some top-level
    // logic that has to do with that. As much as possible, we want to move level generation into
    // other submodules that each file stays small and understandable. Each submodule usually
    // corresponds to a single phase of level generation. The submodule methods do not typically
    // interact with methods from other submodules. This is a loose guideline, not a hard rule.
}
