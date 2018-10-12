use std::str::FromStr;
use std::fmt;
use std::ops::Add;
use std::collections::HashMap;

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
    /// The number of rows of tiles in the entire world (bound on the size of the map)
    pub rows: usize,
    /// The number of columns of tiles in the entire world (bound on the size of the map)
    pub cols: usize,
    /// The number of rooms to generate on each floor
    pub rooms: usize,
    /// The minimum and maximum width (in tiles) of a room
    pub room_width: Bounds<usize>,
    /// The minimum and maximum height (in tiles) of a room
    pub room_height: Bounds<usize>,
    /// The width of the passageways between rooms
    /// Used to calculate the minimum distance between adjacent rooms
    pub passage_size: usize,
    /// The number of doors to generate on each room
    pub doors: usize,
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
        GameMap {
            key,
            levels: (1..=self.levels).map(|level| self.generate_level(&mut rng, level)).collect(),
        }
    }

    pub fn generate_level(&self, rng: &mut StdRng, level: usize) -> FloorMap {
        let mut map = FloorMap::new(self.rows, self.cols);
        let rooms = self.generate_rooms(rng, &mut map);
        self.place_rooms(&mut map, &rooms);
        println!("{:?}", map);
        self.fill_passages(rng, &mut map);
        println!("{:?}", map);
        self.connect_rooms_passages(rng, &mut map, &rooms);
        println!("{:?}", map);
        self.reduce_dead_ends(&mut map);
        println!("{:?}", map);
        if level < self.levels {
            self.place_to_next_level_tiles(rng, &mut map, &rooms);
        }
        if level > 1 {
            self.place_to_prev_level_tiles(rng, &mut map, &rooms);
        }
        println!("{:?}", map);

        unimplemented!();
    }

    fn generate_rooms(&self, rng: &mut StdRng, map: &mut FloorMap) -> Vec<(RoomId, Room)> {
        let mut rooms = self.generate_special_rooms(rng, map);

        'make_rooms: while rooms.len() < self.rooms {
            let x = rng.gen_range(0, self.cols);
            let y = rng.gen_range(0, self.rows);
            let width = self.room_width.gen(rng);
            let height = self.room_height.gen(rng);

            // Ensure that the room is within the bounds
            if x + width >= self.cols || y + height >= self.rows {
                continue;
            }

            let potential_room = Room::new(x, y, width, height);

            // Ensure no overlap with any other room
            for (_, other_room) in &rooms {
                // Rooms cannot be directly adjacent to each other, this makes enough room for a
                // passage to come through
                let other_room = other_room.expand(self.passage_size);
                if potential_room.has_intersection(other_room) {
                    continue 'make_rooms;
                }
            }
            let room_id = map.add_room(potential_room);
            rooms.push((room_id, potential_room));
        }

        rooms
    }

    fn generate_special_rooms(&self, rng: &mut StdRng, map: &mut FloorMap) -> Vec<(RoomId, Room)> {
        //TODO: Generate treasure chamber on last level, boss/challenge room, etc.
        Vec::new()
    }

    fn place_rooms(&self, map: &mut FloorMap, rooms: &[(RoomId, Room)]) {
        for &(room_id, ref room) in rooms {
            for row_i in room.y()..(room.y() + room.height()) {
                for col_i in room.x()..(room.x() + room.width()) {
                    map.place_tile((row_i, col_i), TileType::Room(room_id));
                    //TODO: Need to open walls to all adjacent tiles with the same room ID
                }
            }
        }
    }

    fn fill_passages(&self, rng: &mut StdRng, map: &mut FloorMap) {
        for row_i in 0..self.rows {
            for col_i in 0..self.cols {
                if map.is_empty((row_i, col_i)) {
                    self.generate_maze(rng, map, (row_i, col_i));
                }
            }
        }
    }

    fn generate_maze(&self, rng: &mut StdRng, map: &mut FloorMap, (row_i, col_i): (usize, usize)) {
        assert_eq!(self.passage_size, 1, "only a passage_size of 1 is supported for now");

        let mut parent_map = HashMap::new();
        let seen = map.depth_first_search_mut((row_i, col_i), |map, node, adjacents| {
            let mut adjacents: Vec<_> = adjacents.into_iter()
                .filter(|&pt| map.is_empty(pt))
                .collect();
            rng.shuffle(&mut adjacents);

            for &adj in &adjacents {
                parent_map.insert(adj, node);
            }

            adjacents
        });

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

    /// Connects each room to a passage
    fn connect_rooms_passages(&self, rng: &mut StdRng, map: &mut FloorMap, rooms: &[(RoomId, Room)]) {
        for &(room_id, ref room) in rooms {
            let mut doors = self.doors;
            while doors > 0 {
                // Pick a random point on one of the edges of the room
                let (row, col) = if rng.gen() {
                    // Random horizontal edge
                    (
                        room.y() + *rng.choose(&[0, room.height()-1]).unwrap(),
                        room.x() + rng.gen_range(0, room.width()),
                    )
                } else {
                    (
                        room.y() + rng.gen_range(0, room.height()),
                        room.x() + *rng.choose(&[0, room.width()-1]).unwrap(),
                    )
                };

                debug_assert!(map.is_room_id((row, col), room_id),
                    "bug: tried to connect a passage to a room with the wrong ID");

                let adjacents: Vec<_> = map.adjacent_positions((row, col))
                    .filter(|&pt| map.is_passageway(pt))
                    .collect();
                let passage = match rng.choose(&adjacents) {
                    Some(&pt) => pt,
                    // No passage adjacent to this room tile
                    None => continue,
                };

                // Already opened this tile
                if map.is_open_between((row, col), passage) {
                    continue;
                }

                // This check only works if we are putting fewer than 4 doors on every room
                if self.doors <= 4 {
                    // Don't put a door on the same side of a room as another door

                    // We check for this by scanning horizontally and vertically for any other
                    // doors. This approach isn't perfect though because in addition to disallowing
                    // what we don't want, it also makes it impossible for two doors to be directly
                    // opposite from each other in a room. This is not great, so in the future it
                    // would be good to improve this to be a more sophisticated check--taking
                    // taking corners into account properly. (TODO)

                    // Search the horizontal edge
                    if (0..room.width()).any(|col| map.adjacent_open_passages((row, room.x() + col)).next().is_some()) {
                        continue;
                    }
                    // Search the vertical edge
                    if (0..room.height()).any(|row| map.adjacent_open_passages((room.y() + row, col)).next().is_some()) {
                        continue;
                    }
                }

                map.open_between((row, col), passage);
                doors -= 1;
            }
        }
    }

    fn reduce_dead_ends(&self, map: &mut FloorMap) {
        for row_i in 0..self.rows {
            for col_i in 0..self.cols {
                if map.is_dead_end((row_i, col_i)) {
                    self.reduce_dead_ends_search(map, (row_i, col_i));
                }
            }
        }
    }

    fn reduce_dead_ends_search(&self, map: &mut FloorMap, (row_i, col_i): (usize, usize)) {
        map.depth_first_search_mut((row_i, col_i), |map, node, adjacents| {
            map.remove_passageway(node);

            adjacents.into_iter().filter(|&pt| map.is_dead_end(pt)).collect()
        });
    }

    fn place_to_next_level_tiles(&self, rng: &mut StdRng, map: &mut FloorMap, rooms: &[(RoomId, Room)]) {
        self.place_object_in_rooms(rng, map, rooms, self.next_prev_tiles, TileObject::ToNextLevel);
    }

    fn place_to_prev_level_tiles(&self, rng: &mut StdRng, map: &mut FloorMap, rooms: &[(RoomId, Room)]) {
        self.place_object_in_rooms(rng, map, rooms, self.next_prev_tiles, TileObject::ToPrevLevel);
    }

    /// Places `nrooms` copies of a TileObject into `nrooms` randomly choosen rooms from rooms
    fn place_object_in_rooms<OB>(&self, rng: &mut StdRng, map: &mut FloorMap, rooms: &[(RoomId, Room)], nrooms: usize, mut object: OB)
        where OB: FnMut(usize) -> TileObject {
        assert!(rooms.len() >= nrooms,
            "Not enough rooms to place next/prev level tiles");

        // To do this using choose we would need to allocate anyway, so we might as well just use
        // shuffle to do all the random choosing at once
        let mut rooms: Vec<_> = rooms.to_vec();
        rng.shuffle(&mut rooms);

        for (i, (room_id, room)) in rooms.into_iter().take(nrooms).enumerate() {
            loop {
                // Pick a random point on one of the edges of the room
                let (row, col) = if rng.gen() {
                    // Random horizontal edge
                    (
                        room.y() + *rng.choose(&[0, room.height()-1]).unwrap(),
                        room.x() + rng.gen_range(0, room.width()),
                    )
                } else {
                    (
                        room.y() + rng.gen_range(0, room.height()),
                        room.x() + *rng.choose(&[0, room.width()-1]).unwrap(),
                    )
                };

                assert!(map.is_room_id((row, col), room_id),
                    "bug: picked a tile that was not in the room it was supposed to be");

                // Don't put anything beside a doorway
                if map.adjacent_open_passages((row, col)).next().is_some() {
                    continue;
                }

                let tile = map[row][col].as_mut().expect("bug: did not choose a valid room tile");
                if tile.has_object() {
                    continue;
                }

                tile.place_object(object(i));
                break;
            }
        }
    }
}
