use std::str::FromStr;
use std::fmt;
use std::ops::Add;
use std::collections::{HashSet, VecDeque, HashMap};

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

/// Represents the minimum and maximum boundary for a given type
/// Both boundaries are inclusive
pub struct Bounds<T> {
    min: T,
    max: T,
}

impl<T: PartialOrd + SampleUniform + Copy> Bounds<T> {
    fn gen<R: Rng>(&self, rng: &mut R) -> T
        where Standard: Distribution<T>,
              T: Add<Output=T> + From<u8> {
        // Need to add 1 for this to be an inclusive range. These fancy type bounds allow for that.
        // From<u8> was chosen because a lot of types support From<u8>.
        rng.gen_range(self.min, self.max + 1.into())
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
    /// The width of the passageways between rooms
    /// Used to calculate the minimum distance between adjacent rooms
    pub passage_size: u32,
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
        let mut map = FloorMap::new(self.rows as usize, self.cols as usize);
        let rooms = self.generate_rooms(rng);
        self.place_rooms(&mut map, &rooms);
        self.fill_passages(rng, &mut map);

        println!("{:?}", map);
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
                // Rooms cannot be directly adjacent to each other, this makes enough room for a
                // passage to come through
                let margin = self.passage_size;
                let other_room = Rect::new(
                    other_room.x() - margin as i32,
                    other_room.y() - margin as i32,
                    other_room.width() + margin * 2,
                    other_room.height() + margin * 2,
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

    fn place_rooms(&self, map: &mut FloorMap, rooms: &[Rect]) {
        for (room_id, room) in rooms.into_iter().enumerate() {
            let room_id = RoomId(room_id);
            for row_i in room.y()..(room.y() + room.height() as i32) {
                for col_i in room.x()..(room.x() + room.width() as i32) {
                    map.place_tile((row_i as usize, col_i as usize), TileType::Room(room_id));
                    //TODO: Need to open walls to all adjacent tiles with the same room ID
                }
            }
        }
    }

    fn fill_passages(&self, rng: &mut StdRng, map: &mut FloorMap) {
        for row_i in 0..self.rows as usize {
            for col_i in 0..self.cols as usize {
                if map.is_empty((row_i, col_i)) {
                    self.generate_maze(rng, map, (row_i, col_i));
                }
            }
        }
    }

    fn generate_maze(&self, rng: &mut StdRng, map: &mut FloorMap, (row_i, col_i): (usize, usize)) {
        assert_eq!(self.passage_size, 1, "only a passage_size of 1 is supported for now");

        let mut parent_map = HashMap::new();
        let mut seen = HashSet::new();
        let mut open = VecDeque::new();
        open.push_front((row_i, col_i));

        while let Some(node) = open.pop_front() {
            if seen.contains(&node) {
                continue;
            }
            seen.insert(node);

            let mut adjacents: Vec<_> = map.adjacent_cells(node)
                .filter(|&pt| !seen.contains(&pt) && map.is_empty(pt))
                .collect();
            rng.shuffle(&mut adjacents);
            let mut adjacents = adjacents.into_iter();
            // This is a depth first search, so we insert the first element and append the rest
            if let Some(adj) = adjacents.next() {
                open.push_front(adj);
                parent_map.insert(adj, node);
            }
            for adj in adjacents {
                open.push_back(adj);
                parent_map.insert(adj, node);
            }
        }

        // Insert new passageway tiles
        for pt in seen {
            map.place_tile(pt, TileType::Passageway);
        }

        // Place all of the found paths onto the tiles
        for (pt1, pt2) in parent_map {
            // Open the walls between these two cells
            map.open_between(pt1, pt2);
        }
    }
}
