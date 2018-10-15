mod tile;
mod room;
mod tile_pos;
mod grid_size;

pub use self::tile::*;
pub use self::room::*;
pub use self::tile_pos::*;
pub use self::grid_size::*;

use std::fmt;
use std::cmp;
use std::ops::{Index, IndexMut};
use std::collections::{HashSet, VecDeque};

use sdl2::rect::{Rect, Point};

/// A single row of the map's tiles
pub type Row = [Option<Tile>];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoomId(usize);

impl fmt::Display for RoomId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A type that represents the static floor plan of a map
#[derive(Clone)]
pub struct FloorMap {
    tiles: Vec<Vec<Option<Tile>>>,
    /// The RoomId is the index into this field
    rooms: Vec<Room>,
    /// The width and height of every tile
    tile_size: u32,
    /// The sprite used to render empty tiles (i.e. when there is no tile)
    empty_tile_sprite: SpriteImage,
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
                    Some(tile) => {
                        let object = tile.object.as_ref().map(|o| o.to_string())
                            .unwrap_or_else(|| " ".to_string());
                        use self::TileType::*;
                        write!(f, "{}", match tile.ttype {
                            Passageway => {
                                object.on_green()
                            },
                            Room(id) | Wall(id) | Door {room_id: id, ..} => {
                                let object = if tile.is_wall() {
                                    "\u{25a2}".to_string()
                                } else { object };

                                match self.room(id).room_type() {
                                    RoomType::Normal => object.on_blue(),
                                    RoomType::Challenge => object.on_red(),
                                    RoomType::PlayerStart => object.on_bright_blue(),
                                    RoomType::TreasureChamber => object.on_yellow(),
                                }
                            },
                        })?;
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
    pub fn new(GridSize {rows, cols}: GridSize, tile_size: u32, empty_tile_sprite: SpriteImage) -> Self {
        assert!(rows > 0 && cols > 0, "Cannot create a grid with zero rows or columns");
        FloorMap {
            tiles: vec![vec![None; cols]; rows],
            rooms: Vec::new(),
            tile_size,
            empty_tile_sprite,
        }
    }

    /// Returns the sprite that should be used to render empty tiles (i.e. when there is no tile)
    pub fn empty_tile_sprite(&self) -> SpriteImage {
        self.empty_tile_sprite
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
    pub fn get(&self, TilePos {row, col}: TilePos) -> Option<&Tile> {
        self[row][col].as_ref()
    }

    /// Gets the tile at the given position (or None if empty)
    pub fn get_mut(&mut self, TilePos {row, col}: TilePos) -> Option<&mut Tile> {
        self[row][col].as_mut()
    }

    /// Returns true if the given position is empty (no tile)
    pub fn is_empty(&self, TilePos {row, col}: TilePos) -> bool {
        self[row][col].is_none()
    }

    /// Returns true if the given position is part of the room with the given ID
    pub fn is_room_id(&self, TilePos {row, col}: TilePos, room_id: RoomId) -> bool {
        match self[row][col] {
            Some(Tile {ttype: TileType::Room(id), ..}) => id == room_id,
            _ => false,
        }
    }

    /// Returns true if the given position is a dead end passageway
    /// A dead end is defined as a passage that is surrounded by 3 wall tiles and 1 passage tile.
    pub fn is_dead_end(&self, pos: TilePos) -> bool {
        match self.get(pos) {
            Some(Tile {ttype: TileType::Passageway, ..}) => {
                let mut walls = 0;
                let mut passageways = 0;
                for pos in self.adjacent_positions(pos) {
                    match self.get(pos) {
                        Some(tile) => match tile.ttype {
                            TileType::Wall(_) => walls += 1,
                            TileType::Passageway => passageways += 1,
                            _ => {}
                        },
                        None => {},
                    }
                }

                walls == 3 && passageways == 1
            },
            _ => false,
        }
    }

    /// Returns true if the given position is passageway
    pub fn is_passageway(&self, TilePos {row, col}: TilePos) -> bool {
        match self[row][col] {
            Some(Tile {ttype: TileType::Passageway, ..}) => true,
            _ => false,
        }
    }

    pub fn rooms(&self) -> impl Iterator<Item=&Room> {
        self.rooms.iter()
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
    pub fn place_tile(&mut self, TilePos {row, col}: TilePos, ttype: TileType, sprite: SpriteImage) {
        let tile = &mut self[row][col];
        // Should not be any other tile here already
        debug_assert!(tile.is_none(),
            "bug: attempt to place tile on a position where a tile was already placed");
        *tile = Some(Tile::with_type(ttype, sprite));
    }

    /// Removes a passageway tile from the map, leaving an empty tile (no tile) in its place
    pub(in super) fn remove_passageway(&mut self, pos: TilePos) {
        assert!(self.is_passageway(pos), "bug: remove passageway can only be called on a passageway tile");

        self[pos.row][pos.col] = None;
    }

    /// Returns true if there is NO wall between two cells. The cells must have exactly one tile
    /// between them in one of the cardinal directions
    pub fn is_open_between(&self, pos1: TilePos, pos2: TilePos) -> bool {
        let potential_wall = match pos2.difference(pos1) {
            // second position is north of first position
            (-2, 0) => self.adjacent_north(pos1),
            // second position is east of first position
            (0, 2) => self.adjacent_east(pos1),
            // second position is south of first position
            (2, 0) => self.adjacent_south(pos1),
            // second position is west of first position
            (0, -2) => self.adjacent_west(pos1),
            _ => unreachable!("bug: attempt to check if two cells have no wall between them when cells did not have exactly one cell between them"),
        };

        self.get(potential_wall).map(|tile| tile.is_wall()).unwrap_or(false)
    }

    /// Removes the wall between two cells and replaces it with a Room tile of the same RoomId.
    /// The tile between the two given positions must be a Room tile.
    pub fn open_between(&mut self, pos1: TilePos, pos2: TilePos) {
        let tile = match pos2.difference(pos1) {
            // second position is north of first position
            (-2, 0) => self.adjacent_north(pos1),
            // second position is east of first position
            (0, 2) => self.adjacent_east(pos1),
            // second position is south of first position
            (2, 0) => self.adjacent_south(pos1),
            // second position is west of first position
            (0, -2) => self.adjacent_west(pos1),
            _ => unreachable!("bug: attempt to open a wall between two cells when the cells did not have exactly one cell between them"),
        };
        self.get_mut(tile).expect("bug: attempt to turn an empty tile into a room tile").wall_to_room();
    }

    /// Returns an iterator over the positions of all tiles contained within this map
    pub fn tile_positions(&self) -> impl Iterator<Item=TilePos> {
        let cols = self.cols_len();
        (0..self.rows_len()).flat_map(move |row| (0..cols).map(move |col| TilePos {row, col}))
    }

    /// Returns the position one tile north of the given position, panics if already at the northmost tile
    pub fn adjacent_north(&self, TilePos {row, col}: TilePos) -> TilePos {
        assert_ne!(row, 0, "bug: already at northmost position");
        TilePos {
            row: row - 1,
            col,
        }
    }

    /// Returns the position one tile east of the given position, panics if already at the eastmost tile
    pub fn adjacent_east(&self, TilePos {row, col}: TilePos) -> TilePos {
        assert!(col < self.cols_len() - 1, "bug: already at eastmost position");
        TilePos {
            row,
            col: col + 1,
        }
    }

    /// Returns the position one tile south of the given position, panics if already at the southmost tile
    pub fn adjacent_south(&self, TilePos {row, col}: TilePos) -> TilePos {
        assert!(row < self.rows_len() - 1, "bug: already at southmost position");
        TilePos {
            row: row + 1,
            col,
        }
    }

    /// Returns the position one tile west of the given position, panics if already at the westmost tile
    pub fn adjacent_west(&self, TilePos {row, col}: TilePos) -> TilePos {
        assert_ne!(col, 0, "bug: already at westmost position");
        TilePos {
            row,
            col: col - 1,
        }
    }

    /// Returns an iterator of tile positions adjacent to the given tile in the four cardinal
    /// directions. Only returns valid cell positions.
    pub fn adjacent_positions(&self, TilePos {row, col}: TilePos) -> impl Iterator<Item=TilePos> {
        let rows = self.rows_len();
        let cols = self.cols_len();
        [(-1, 0), (0, -1), (1, 0), (0, 1)].into_iter().filter_map(move |(row_offset, col_offset)| {
            let row = row as isize + row_offset;
            let col = col as isize + col_offset;

            if row < 0 || row >= rows as isize || col < 0 || col >= cols as isize {
                None
            } else {
                Some(TilePos {row: row as usize, col: col as usize})
            }
        })
    }

    /// Returns an iterator of tile positions one tile (gap) away from the given tile in the four
    /// cardinal directions. Only returns valid cell positions.
    pub fn gap_adjacent_positions(&self, TilePos {row, col}: TilePos) -> impl Iterator<Item=TilePos> {
        let rows = self.rows_len();
        let cols = self.cols_len();
        [(-2, 0), (0, -2), (2, 0), (0, 2)].into_iter().filter_map(move |(row_offset, col_offset)| {
            let row = row as isize + row_offset;
            let col = col as isize + col_offset;

            if row < 0 || row >= rows as isize || col < 0 || col >= cols as isize {
                None
            } else {
                Some(TilePos {row: row as usize, col: col as usize})
            }
        })
    }

    /// Returns an iterator of adjacent passages that do not have a wall between them and the
    /// given position.
    pub fn adjacent_open_passages(&self, pos: TilePos) -> impl Iterator<Item=TilePos> + '_ {
        self.adjacent_positions(pos)
            .filter(move |&pt| self.is_passageway(pt) && self.is_open_between(pos, pt))
    }

    /// Executes a depth-first search starting from a given tile
    ///
    /// Takes a closure that is given the next position "node" to be processed and its
    /// adjacents. The closure should return the adjacents that you want it to keep searching.
    ///
    /// Returns the positions that were visited
    pub fn depth_first_search_mut<F>(&mut self, start: TilePos, mut next_adjacents: F) -> HashSet<TilePos>
        where F: FnMut(&mut Self, TilePos, Vec<TilePos>) -> Vec<TilePos> {

        let mut seen = HashSet::new();
        let mut open = VecDeque::new();
        open.push_front(start);

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

    /// Returns the sprites of tiles within (or around) the region defined by bounds
    pub fn sprites_within(&self, bounds: Rect) -> impl Iterator<Item=(Point, SpriteImage)> + '_ {
        // While the caller is allowed to ask for tiles within a boundary Rect that starts at
        // negative coordinates, the top left of the map is defined as (0, 0). That means that we
        // can at most request tiles up to that top left corner. The calls to `max()` here help
        // enforce that by making sure we don't convert a negative number to an unsigned type.
        let x = cmp::max(bounds.x(), 0) as usize;
        let y = cmp::max(bounds.y(), 0) as usize;
        let width = bounds.width() as usize;
        let height = bounds.height() as usize;

        let clamp_col = |col| cmp::min(cmp::max(col, 0), self.cols_len()-1);
        let clamp_row = |row| cmp::min(cmp::max(row, 0), self.rows_len()-1);

        let start_col = clamp_col(x / self.tile_size as usize);
        let start_row = clamp_row(y / self.tile_size as usize);
        let end_col = clamp_col((x + width) / self.tile_size as usize);
        let end_row = clamp_row((y + height) / self.tile_size as usize);

        (start_row..=end_row).flat_map(move |row| (start_col..=end_col).map(move |col| {
            // The position of the tile in world coordinates
            let pos = Point::new(
                col as i32 * self.tile_size as i32,
                row as i32 * self.tile_size as i32,
            );
            match self.get(TilePos {row, col}) {
                None => (pos, self.empty_tile_sprite),
                Some(tile) => (pos, tile.sprite)
            }
        }))
    }
}
