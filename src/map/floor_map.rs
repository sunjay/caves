use std::fmt;
use std::cmp;
use std::ops::{Index, IndexMut};
use std::collections::{HashSet, VecDeque};

use sdl2::rect::{Rect};

#[derive(Debug, Clone)]
pub enum Item {
    TreasureKey,
    RoomKey,
    Potion {stength: u32},
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoomId(usize);

impl fmt::Display for RoomId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub enum TileType {
    /// Tiles that can be used to pass between rooms
    Passageway,
    /// Tiles that are part of a given room
    Room(RoomId),
}

/// The object or item placed at a particular tile
#[derive(Debug, Clone)]
pub enum TileObject {
    /// Stepping on this tile transports you to the next level
    /// Field is the ID of this gate and the ID of the ToPrevLevel tile that this should connect to
    ToNextLevel(usize),
    /// Stepping on this tile transports you to the previous level
    /// Field is the ID of this gate and the ID of the ToNextLevel tile that this should connect to
    ToPrevLevel(usize),
    /// A point where an enemy *may* spawn
    EnemySpawn {
        /// Probability that an enemy will spawn here: 1.0 means that the enemy will definitely
        /// spawn and 0.0 means that an enemy will not spawn
        probability: f64,
    },
    Chest(Item),
}

impl fmt::Display for TileObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::TileObject::*;
        write!(f, "{}", match *self {
            ToNextLevel(_) => "\u{2193}",
            ToPrevLevel(_) => "\u{2191}",
            _ => " ",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wall {
    Open,
    Closed,
    Locked,
}

#[derive(Debug, Clone)]
pub struct TileWalls {
    pub north: Wall,
    pub east: Wall,
    pub south: Wall,
    pub west: Wall,
}

impl Default for TileWalls {
    fn default() -> Self {
        use self::Wall::*;

        Self {
            north: Closed,
            east: Closed,
            south: Closed,
            west: Closed,
        }
    }
}

impl fmt::Display for TileWalls {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Wall::*;
        write!(f, "{}", match *self {
            TileWalls {north: Closed, east: Closed, south: Closed, west: Closed} => {
                "\u{26ac}" // all closed
            },
            TileWalls {north: Open, east: Closed, south: Closed, west: Closed} => {
                "\u{257d}" // N
            },
            TileWalls {north: Closed, east: Open, south: Closed, west: Closed} => {
                "\u{257e}" // E
            },
            TileWalls {north: Closed, east: Closed, south: Open, west: Closed} => {
                "\u{257f}" // S
            },
            TileWalls {north: Closed, east: Closed, south: Closed, west: Open} => {
                "\u{257c}" // W
            },
            TileWalls {north: Open, east: Open, south: Closed, west: Closed} => {
                "\u{2514}" // NE
            },
            TileWalls {north: Closed, east: Open, south: Open, west: Closed} => {
                "\u{250C}" // SE
            },
            TileWalls {north: Closed, east: Closed, south: Open, west: Open} => {
                "\u{2510}" // SW
            },
            TileWalls {north: Open, east: Closed, south: Closed, west: Open} => {
                "\u{2518}" // NW
            },
            TileWalls {north: Open, east: Closed, south: Open, west: Closed} => {
                "\u{2502}" // NS
            },
            TileWalls {north: Closed, east: Open, south: Closed, west: Open} => {
                "\u{2500}" // EW
            },
            TileWalls {north: Open, east: Open, south: Open, west: Closed} => {
                "\u{251c}" // NES
            },
            TileWalls {north: Closed, east: Open, south: Open, west: Open} => {
                "\u{252c}" // ESW
            },
            TileWalls {north: Open, east: Closed, south: Open, west: Open} => {
                "\u{2524}" // NSW
            },
            TileWalls {north: Open, east: Open, south: Closed, west: Open} => {
                "\u{2534}" // NEW
            },
            TileWalls {north: Open, east: Open, south: Open, west: Open} => {
                "\u{253c}" // NESW
            },
            _ => " ",
        })
    }
}

impl TileWalls {
    /// Returns true if only a single wall is open
    pub fn is_dead_end(&self) -> bool {
        use self::Wall::*;
        match *self {
            TileWalls {north: Open, east: Closed, south: Closed, west: Closed} |
            TileWalls {north: Closed, east: Open, south: Closed, west: Closed} |
            TileWalls {north: Closed, east: Closed, south: Open, west: Closed} |
            TileWalls {north: Closed, east: Closed, south: Closed, west: Open} => {
                true
            },
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tile {
    pub ttype: TileType,
    pub object: Option<TileObject>,
    pub walls: TileWalls,
}

impl Tile {
    fn with_type(ttype: TileType) -> Self {
        Self {
            ttype,
            object: Default::default(),
            walls: Default::default(),
        }
    }

    pub fn has_object(&self) -> bool {
        self.object.is_some()
    }

    pub fn place_object(&mut self, object: TileObject) {
        self.object = Some(object);
    }
}

pub type Row = [Option<Tile>];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomType {
    /// A normal room containing enemeies, chests, special tiles, etc. Most rooms have this type.
    Normal,
    /// Challenge rooms can appear on any level and provide the player with some reward if they
    /// overcome all the enemies in that room without dying
    Challenge,
    /// The room that the player should spawn in at the start of the game. Should only exist
    /// on the first level. No ToNextLevel or ToPrevLevel tiles should be in this room.
    /// Enemies should also not be placed in this room.
    PlayerStart,
    /// The goal of the game. Entering this game means the player has won. Should only exist on
    /// the very last level. No ToNextLevel or ToPrevLevel tiles should be in this room.
    /// Enemies should also not be placed in this room.
    TreasureChamber,
}

/// A room is represented by a 2D span of tiles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Room {
    rtype: RoomType,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

impl Room {
    /// Create a new normal room
    pub fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self::with_type(RoomType::Normal, x, y, width, height)
    }

    pub fn with_type(rtype: RoomType, x: usize, y: usize, width: usize, height: usize) -> Self {
        Self {x, y, width, height, rtype}
    }

    pub fn x(self) -> usize { self.x }
    pub fn y(self) -> usize { self.y }
    pub fn width(self) -> usize { self.width }
    pub fn height(self) -> usize { self.height }

    /// Returns true if a room is allowed to contain ToNextLevel tiles
    pub fn can_contain_to_next_level(&self) -> bool {
        match self.rtype {
            RoomType::Normal | RoomType::Challenge => true,
            _ => false,
        }
    }

    /// Returns true if a room is allowed to contain ToPrevLevel tiles
    pub fn can_contain_to_prev_level(&self) -> bool {
        // currently the same as the rooms that can contain ToNextLevel
        self.can_contain_to_next_level()
    }

    pub fn to_rect(self) -> Rect {
        Rect::new(self.x as i32, self.y as i32, self.width as u32, self.height as u32)
    }

    pub fn has_intersection(self, other: Self) -> bool {
        //TODO: Implement this without relying on sdl2. Perhaps based on:
        // https://github.com/servo/euclid/blob/7a4f6f77990fafc63d5fe5028df2660488e6749c/src/rect.rs#L124
        self.to_rect().has_intersection(other.to_rect())
    }

    /// Expands the room (as much as possible) to have an additional margin on all sides
    ///
    /// Will only expand up to the point (0,0). Can expand arbitrarily in the other direction.
    pub fn expand(self, margin: usize) -> Self {
        // Avoid integer overflow by only subtracting as much as possible
        let top_expansion = cmp::min(self.y, margin);
        let left_expansion = cmp::min(self.x, margin);
        Self::new(
            self.x - left_expansion,
            self.y - top_expansion,
            self.width + left_expansion + margin,
            self.height + top_expansion + margin,
        )
    }
}

/// A type that represents the static floor plan of a map
#[derive(Clone)]
pub struct FloorMap {
    tiles: Vec<Vec<Option<Tile>>>,
    /// The RoomId is the index into this field
    rooms: Vec<Room>,
}

impl Index<usize> for FloorMap {
    type Output = Row;

    fn index(&self, index: usize) -> &Self::Output {
        self.tiles.index(index)
    }
}

impl IndexMut<usize> for FloorMap {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.tiles.index_mut(index)
    }
}

impl fmt::Debug for FloorMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use colored::*;

        for row in self.rows() {
            for tile in row {
                match tile {
                    None => write!(f, "{}", " ".on_black())?,
                    Some(tile) => match tile.ttype {
                        TileType::Passageway => {
                            write!(f, "{}", tile.walls.to_string().on_green())?
                        },
                        TileType::Room(id) => {
                            let object = tile.object.as_ref().map(|o| o.to_string())
                                .unwrap_or(" ".to_string());
                            write!(f, "{}", match self.room(id).rtype {
                                RoomType::Normal => object.on_blue(),
                                RoomType::Challenge => object.on_red(),
                                RoomType::PlayerStart => object.on_bright_blue(),
                                RoomType::TreasureChamber => object.on_yellow(),
                            })?
                        },
                    },
                }
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

impl FloorMap {
    /// Create a new FloorMap with the given number of rows and columns
    pub fn new(rows: usize, cols: usize) -> Self {
        assert!(rows > 0 && cols > 0, "Cannot create a grid with zero rows or columns");
        FloorMap {
            tiles: vec![vec![None; cols]; rows],
            rooms: Vec::new(),
        }
    }

    pub fn level_boundary(&self) -> Rect {
        Rect::new(0, 0, self.cols_len() as u32, self.rows_len() as u32)
    }

    /// Returns the number of rows in this grid
    pub fn rows_len(&self) -> usize {
        self.tiles.len()
    }

    /// Returns the number of columns in this grid
    pub fn cols_len(&self) -> usize {
        self[0].len()
    }

    /// Returns an iterator over each row
    pub fn rows(&self) -> impl Iterator<Item=&Row> {
        self.tiles.iter().map(|r| r.as_slice())
    }

    /// Gets the tile at the given position (or None if empty)
    pub fn get(&self, (row, col): (usize, usize)) -> Option<&Tile> {
        self[row][col].as_ref()
    }

    /// Returns true if the given position is empty (no tile)
    pub fn is_empty(&self, (row, col): (usize, usize)) -> bool {
        self[row][col].is_none()
    }

    /// Returns true if the given position is part of the room with the given ID
    pub fn is_room_id(&self, (row, col): (usize, usize), room_id: RoomId) -> bool {
        match self[row][col] {
            Some(Tile {ttype: TileType::Room(id), ..}) => id == room_id,
            _ => false,
        }
    }

    /// Returns true if the given position is a dead end passageway
    pub fn is_dead_end(&self, (row, col): (usize, usize)) -> bool {
        match self[row][col] {
            Some(Tile {ttype: TileType::Passageway, ref walls, ..}) => walls.is_dead_end(),
            _ => false,
        }
    }

    /// Returns true if the given position is passageway
    pub fn is_passageway(&self, (row, col): (usize, usize)) -> bool {
        match self[row][col] {
            Some(Tile {ttype: TileType::Passageway, ..}) => true,
            _ => false,
        }
    }

    pub fn room(&self, room_id: RoomId) -> &Room {
        &self.rooms[room_id.0]
    }

    /// Add a room to the map. Rooms should NOT be overlapping, though this condition is NOT
    /// checked by this method. Hence why this is private.
    pub(in super) fn add_room(&mut self, room: Room) -> RoomId {
        self.rooms.push(room);
        RoomId(self.rooms.len() - 1)
    }

    /// Places a tile with the given type at the given location
    ///
    /// Panics if that location was not previously empty
    pub fn place_tile(&mut self, (row, col): (usize, usize), ttype: TileType) {
        let tile = &mut self[row][col];
        // Should not be any other tile here already
        debug_assert!(tile.is_none(),
            "bug: attempt to place tile on a position where a tile was already placed");
        *tile = Some(Tile::with_type(ttype));
    }

    /// Removes a passageway from the map and closes any walls around it
    pub(in super) fn remove_passageway(&mut self, (row, col): (usize, usize)) {
        assert!(self.is_passageway((row, col)), "bug: remove passageway can only be called on a passageway tile");
        let adjacents: Vec<_> = self.adjacent_positions((row, col))
            .filter(|&pt| !self.is_empty(pt))
            .collect();

        for adj in adjacents {
            self.close_between((row, col), adj);
        }

        self[row][col] = None;
    }

    /// Returns true if there is NO wall between two adjacent cells
    pub fn is_open_between(&self, (row1, col1): (usize, usize), (row2, col2): (usize, usize)) -> bool {
        macro_rules! wall_is_open {
            ($dir:ident, $opp:ident) => {
                match (self.get((row1, col1)), self.get((row2, col2))) {
                    (Some(tile1), Some(tile2)) => {
                        debug_assert_eq!(tile1.walls.$dir, tile2.walls.$opp);
                        tile1.walls.$dir == Wall::Open
                    },
                    // If either option is an empty tile then by definition we cannot have an open
                    // wall since that would lead nowhere!
                    (Some(tile1), None) => {
                        debug_assert_eq!(tile1.walls.$dir, Wall::Closed);
                        false
                    },
                    (None, Some(tile2)) => {
                        debug_assert_eq!(tile2.walls.$opp, Wall::Closed);
                        false
                    },
                    (None, None) => false,
                }
            };
        }
        match (row2 as isize - row1 as isize, col2 as isize - col1 as isize) {
            // second position is north of first position
            (-1, 0) => wall_is_open!(north, south),
            // second position is east of first position
            (0, 1) => wall_is_open!(east, west),
            // second position is south of first position
            (1, 0) => wall_is_open!(south, north),
            // second position is west of first position
            (0, -1) => wall_is_open!(west, east),
            _ => unreachable!("bug: attempt to check if two non-adjacent cells have an open wall between them"),
        }
    }

    /// Removes the wall between two adjacent cells
    pub fn open_between(&mut self, (row1, col1): (usize, usize), (row2, col2): (usize, usize)) {
        macro_rules! open {
            ($wall1:ident, $wall2:ident) => {
                {
                    self[row1][col1].as_mut().expect("Cannot open a wall to an empty tile").walls.$wall1 = Wall::Open;
                    self[row2][col2].as_mut().expect("Cannot open a wall to an empty tile").walls.$wall2 = Wall::Open;
                }
            };
        }
        match (row2 as isize - row1 as isize, col2 as isize - col1 as isize) {
            // second position is north of first position
            (-1, 0) => open!(north, south),
            // second position is east of first position
            (0, 1) => open!(east, west),
            // second position is south of first position
            (1, 0) => open!(south, north),
            // second position is west of first position
            (0, -1) => open!(west, east),
            _ => unreachable!("bug: attempt to open a wall between two non-adjacent cells"),
        }
    }

    /// Adds a wall between two adjacent cells
    pub fn close_between(&mut self, (row1, col1): (usize, usize), (row2, col2): (usize, usize)) {
        macro_rules! close {
            ($wall1:ident, $wall2:ident) => {
                {
                    // Note that walls aren't allowed to be open when there is no tile on the other side
                    self[row1][col1].as_mut().expect("Cannot close a wall to an empty tile").walls.$wall1 = Wall::Closed;
                    self[row2][col2].as_mut().expect("Cannot close a wall to an empty tile").walls.$wall2 = Wall::Closed;
                }
            };
        }
        match (row2 as isize - row1 as isize, col2 as isize - col1 as isize) {
            // second position is north of first position
            (-1, 0) => close!(north, south),
            // second position is east of first position
            (0, 1) => close!(east, west),
            // second position is south of first position
            (1, 0) => close!(south, north),
            // second position is west of first position
            (0, -1) => close!(west, east),
            _ => unreachable!("bug: attempt to open a wall between two non-adjacent cells"),
        }
    }

    /// Returns an iterator of cell positions adjacent to the given cell in the four cardinal
    /// directions. Only returns valid cell positions.
    pub fn adjacent_positions(&self, (row, col): (usize, usize)) -> impl Iterator<Item=(usize, usize)> + '_ {
        [(-1, 0), (0, -1), (1, 0), (0, 1)].into_iter().filter_map(move |(row_offset, col_offset)| {
            let row = row as isize + row_offset;
            let col = col as isize + col_offset;

            if row < 0 || row >= self.rows_len() as isize || col < 0 || col >= self.cols_len() as isize {
                None
            } else {
                Some((row as usize, col as usize))
            }
        })
    }

    /// Returns an iterator of adjacent passages that do not have a wall between them and the
    /// given position.
    pub fn adjacent_open_passages(&self, (row, col): (usize, usize)) -> impl Iterator<Item=(usize, usize)> + '_ {
        self.adjacent_positions((row, col))
            .filter(move |&pt| self.is_passageway(pt) && self.is_open_between((row, col), pt))
    }

    /// Executes a depth-first search starting from a given tile
    ///
    /// Takes a closure that is given the next (usize, usize) node to be processed and its
    /// adjacents. The closure should return the adjacents that you want it to keep searching.
    ///
    /// Returns the (usize, usize) positions that were visited
    pub fn depth_first_search_mut<F>(&mut self, (row, col): (usize, usize), mut next_adjacents: F) -> HashSet<(usize, usize)>
        where F: FnMut(&mut Self, (usize, usize), Vec<(usize, usize)>) -> Vec<(usize, usize)> {

        let mut seen = HashSet::new();
        let mut open = VecDeque::new();
        open.push_front((row, col));

        while let Some(node) = open.pop_front() {
            if seen.contains(&node) {
                continue;
            }
            seen.insert(node);

            let adjacents = self.adjacent_positions(node).filter(|pt| !seen.contains(pt)).collect();
            let mut adjacents = next_adjacents(self, node, adjacents).into_iter();

            // This is a depth first search, so we insert the first element and append the rest
            if let Some(adj) = adjacents.next() {
                open.push_front(adj);
            }
            for adj in adjacents {
                open.push_back(adj);
            }
        }

        seen
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn room_expand() {
        // Expanding a room should not go beyond (0,0) - i.e. it should avoid subtraction with
        // underflow
        let room = Room::new(0, 0, 10, 10);
        assert_eq!(room.expand(2), Room::new(0, 0, 12, 12));
        let room = Room::new(1, 1, 10, 10);
        assert_eq!(room.expand(2), Room::new(0, 0, 13, 13));
        let room = Room::new(2, 2, 10, 10);
        assert_eq!(room.expand(2), Room::new(0, 0, 14, 14));
        let room = Room::new(3, 2, 10, 12);
        assert_eq!(room.expand(2), Room::new(1, 0, 14, 16));
    }
}
