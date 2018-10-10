use std::str::FromStr;
use std::fmt;

use sdl2::rect::Rect;
use rand::{
    random,
     StdRng,
     SeedableRng,
     Rng,
     distributions::{
         Distribution,
         Standard,
         uniform::SampleUniform,
     },
 };
use base64::{self, DecodeError};

use map::*;

lazy_static! {
    /// The configuration of the encoder/decoder for the seed
    static ref SEED_ENCODER_CONFIG: base64::Config = base64::Config::new(
        base64::CharacterSet::UrlSafe,
        false,
        false,
        base64::LineWrap::NoWrap,
    );
}

#[derive(Debug)]
pub enum InvalidMapKey {
    InvalidLength,
    DecodeError(DecodeError),
}

/// The seed of the random number generator
type Seed = <StdRng as SeedableRng>::Seed;

/// Uniquely identifies a map
///
/// Can be passed to the generator to recreate a specific map.
///
/// To create a random MapKey, use the `rand::random` function:
///
/// ```rust
/// # use rand::random;
/// # use map_generator::MapKey;
/// let map_key: MapKey = random();
/// ```
///
/// MapKeys can be parsed from strings using `.parse()`:
///
/// ```rust,no_run
/// # use map_generator::MapKey;
/// let map_key: MapKey = "yourvalidmapkey".parse();
/// ```
///
/// You can get the string representation of a MapKey either with `.to_string()` or
/// by directly using Display `{}` formatting:
///
/// ```rust,no_run
/// # use rand::random;
/// # use map_generator::MapKey;
/// let map_key: MapKey = random();
/// assert_eq!(format!("{}", map_key), map_key.to_string());
/// ```
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MapKey(Seed);

impl MapKey {
    fn to_rng(self) -> StdRng {
        StdRng::from_seed(self.0)
    }
}

impl Distribution<MapKey> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> MapKey {
        MapKey(rng.gen())
    }
}

impl fmt::Debug for MapKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MapKey(\"{}\")", self)
    }
}

impl fmt::Display for MapKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", base64::encode_config(&self.0, *SEED_ENCODER_CONFIG))
    }
}

impl FromStr for MapKey {
    type Err = InvalidMapKey;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut key: Seed = Default::default();
        let decoded = base64::decode_config(s, *SEED_ENCODER_CONFIG)
            .map_err(|err| InvalidMapKey::DecodeError(err))?;
        if decoded.len() != key.len() {
            return Err(InvalidMapKey::InvalidLength);
        }
        key.copy_from_slice(&decoded);
        Ok(MapKey(key))
    }
}

pub struct Bounds<T> {
    min: T,
    max: T,
}

impl<T: PartialOrd + SampleUniform + Copy> Bounds<T> {
    fn gen<R: Rng>(&self, rng: &mut R) -> T
        where Standard: Distribution<T> {
        rng.gen_range(self.min, self.max)
    }
}

impl<T> From<(T, T)> for Bounds<T> {
    fn from((min, max): (T, T)) -> Self {
        Bounds {min, max}
    }
}

pub struct MapGenerator {
    /// The number of levels to generate
    pub levels: usize,
    /// The number of rows of tiles in the entire world (bound on the size)
    pub rows: u32,
    /// The number of columns of tiles in the entire world (bound on the size)
    pub cols: u32,
    /// The number of rooms to generate on each floor
    pub rooms: usize,
    /// The minimum and maximum width of a room
    pub room_width: Bounds<u32>,
    /// The minimum and maximum height of a room
    pub room_height: Bounds<u32>,
    /// The minimum distance between adjacent rooms
    pub room_margin: u32,
}

impl MapGenerator {
    pub fn generate(self) -> GameMap {
        self.generate_with_key(random())
    }

    pub fn generate_with_key(self, key: MapKey) -> GameMap {
        let mut rng = key.to_rng();
        GameMap {
            key,
            levels: (0..self.levels).map(|level| self.generate_level(&mut rng, level)).collect(),
        }
    }

    pub fn generate_level(&self, rng: &mut StdRng, level: usize) -> FloorMap {
        let mut tiles = vec![vec![Tile::default(); self.cols as usize]; self.rows as usize];
        let rooms = self.generate_rooms(rng);
        self.place_rooms(&mut tiles, rooms);

        println!("{:?}", FloorMap {level: 1, tiles});
        unimplemented!();
    }

    fn generate_rooms(&self, rng: &mut StdRng) -> Vec<Rect> {
        let mut rooms = self.generate_special_rooms(rng);

        'make_rooms: while rooms.len() < self.rooms {
            let x = rng.gen_range(0, self.cols);
            let y = rng.gen_range(0, self.rows);
            let width = self.room_width.gen(rng);
            let height = self.room_height.gen(rng);

            // Ensure that the room is within the bounds
            if x + width >= self.cols || y + height >= self.rows {
                continue;
            }

            let potential_room = Rect::new(x as i32, y as i32, width, height);

            // Ensure no overlap with any other room
            for other_room in &rooms {
                // Cannot be adjacent to a room that currently exists
                let other_room = Rect::new(
                    other_room.x() - self.room_margin as i32,
                    other_room.y() - self.room_margin as i32,
                    other_room.width() + self.room_margin * 2,
                    other_room.height() + self.room_margin * 2,
                );
                if potential_room.has_intersection(other_room) {
                    continue 'make_rooms;
                }
            }
            rooms.push(potential_room);
        }

        rooms
    }

    fn generate_special_rooms(&self, rng: &mut StdRng) -> Vec<Rect> {
        //TODO: Generate treasure chamber on last level, boss/challenge room, etc.
        Vec::new()
    }

    fn place_rooms(&self, tiles: &mut TileGrid, rooms: Vec<Rect>) {
        for (room_id, room) in rooms.into_iter().enumerate() {
            for row_i in room.y()..(room.y() + room.height() as i32) {
                for col_i in room.x()..(room.x() + room.width() as i32) {
                    let tile = &mut tiles[row_i as usize][col_i as usize];
                    debug_assert!(tile.is_empty(), "bug: should not have overlapping rooms");

                    tile.set_room(RoomId(room_id));
                }
            }
        }
    }
}
