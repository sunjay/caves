use std::collections::{HashSet, VecDeque};
use std::ops::{Index, IndexMut};
use std::iter::once;

use super::{Tile, TileType, SpriteImage, GridSize, TilePos, RoomId};

/// A single row of the map's tiles
pub type Row = [Option<Tile>];

/// Represents a 2D grid of tiles
#[derive(Clone)]
pub struct TileGrid(Vec<Vec<Option<Tile>>>);

impl Index<usize> for TileGrid {
    type Output = Row;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl IndexMut<usize> for TileGrid {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl TileGrid {
    /// Create a new TileGrid with the given number of rows and columns
    pub fn new(GridSize {rows, cols}: GridSize) -> Self {
        assert!(rows > 0 && cols > 0, "Cannot create a grid with zero rows or columns");
        TileGrid(vec![vec![None; cols]; rows])
    }

    /// Returns the number of rows in this grid
    pub fn rows_len(&self) -> usize {
        self.0.len()
    }

    /// Returns the number of columns in this grid
    pub fn cols_len(&self) -> usize {
        self[0].len()
    }

    /// Returns an iterator over each row
    pub fn rows(&self) -> impl Iterator<Item=&Row> {
        self.0.iter().map(|r| r.as_slice())
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

    /// Returns true if the given position is on the edge of this grid
    pub fn is_on_edge(&self, TilePos {row, col}: TilePos) -> bool {
        row == 0 || col == 0 || row == self.rows_len() - 1 || col == self.rows_len() - 1
    }

    /// Returns true if the given position is part of the room with the given ID
    pub fn is_room(&self, TilePos {row, col}: TilePos, room_id: RoomId) -> bool {
        match self[row][col] {
            Some(Tile {ttype: TileType::Room(id), ..}) => id == room_id,
            _ => false,
        }
    }

    /// Returns true if the given position is a wall of the room with the given ID
    pub fn is_room_wall(&self, TilePos {row, col}: TilePos, room_id: RoomId) -> bool {
        match self[row][col] {
            Some(Tile {ttype: TileType::Wall(id), ..}) => id == room_id,
            _ => false,
        }
    }

    /// Returns true if the given position is a room entrance
    /// A room entrance is defined as a room tile that has at least one other room tile on one side
    /// and exactly one adjacent passageway.
    pub fn is_room_entrance(&self, pos: TilePos) -> bool {
        match self.get(pos) {
            Some(Tile {ttype: TileType::Room(id), ..}) => {
                let mut room_tiles = 0;
                let mut passageways = 0;
                for pos in self.adjacent_positions(pos) {
                    match self.get(pos) {
                        Some(tile) => match tile.ttype {
                            TileType::Room(id2) if id2 == *id => room_tiles += 1,
                            TileType::Passageway => passageways += 1,
                            _ => {}
                        },
                        None => {},
                    }
                }

                room_tiles >= 1 && passageways == 1
            },
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

    /// Returns true if the given position is a passageway
    pub fn is_passageway(&self, TilePos {row, col}: TilePos) -> bool {
        match self[row][col] {
            Some(Tile {ttype: TileType::Passageway, ..}) => true,
            _ => false,
        }
    }

    /// Returns true if the given position is a passageway wall
    pub fn is_passageway_wall(&self, TilePos {row, col}: TilePos) -> bool {
        match self[row][col] {
            Some(Tile {ttype: TileType::PassagewayWall, ..}) => true,
            _ => false,
        }
    }

    /// Returns true if the given position is any kind of wall
    pub fn is_wall(&self, TilePos {row, col}: TilePos) -> bool {
        match self[row][col] {
            Some(Tile {ttype: TileType::Wall(_), ..}) |
            Some(Tile {ttype: TileType::PassagewayWall, ..}) => true,
            _ => false,
        }
    }

    /// Places a tile with the given type at the given location
    ///
    /// Panics if that location was not previously empty
    pub fn place_tile(&mut self, TilePos {row, col}: TilePos, ttype: TileType, sprite: SpriteImage) {
        let tile = &mut self[row][col];
        *tile = Some(Tile::with_type(ttype, sprite));
    }

    /// Removes a passageway tile from the map, leaving an empty tile (no tile) in its place
    /// All adjacent passages will become walls.
    pub(in map) fn remove_passageway(&mut self, pos: TilePos, wall_sprite: SpriteImage) {
        assert!(self.is_passageway(pos), "bug: remove passageway can only be called on a passageway tile");

        for pos in self.adjacent_positions(pos) {
            match self.get_mut(pos) {
                None => {},
                Some(tile) => match tile.ttype {
                    TileType::Passageway => tile.become_wall(wall_sprite),
                    TileType::PassagewayWall | TileType::Wall(_) => {},
                    TileType::Room(_) | TileType::Door {..} => {
                        // While we may want to support this in the future by adding a
                        // `room_wall_sprite` argument to this function, it is not useful right
                        // now so we explicitly disable it.
                        unreachable!("bug: removing a passageway that led into a room");
                    },
                }
            }
        }

        self[pos.row][pos.col] = None;
    }

    /// Removes a passageway wall from the map, leaving an empty tile (no tile) in its place
    /// This is only valid if this tile is only surrounded by other walls/empty tiles. Otherwise,
    /// this can accidentally lead to bugs where there is no wall between a tile and emptiness.
    pub(in map) fn remove_passageway_wall(&mut self, pos: TilePos) {
        assert!(self.is_passageway(pos), "bug: remove passageway can only be called on a passageway tile");

        for adj in self.adjacent_positions(pos) {
            match self.get(adj) {
                // Empty adjacent tile is fine
                None => {},
                // Non-empty adjacent must be a wall so that removing this wall doesn't lead to
                // an invalid map
                Some(tile) if tile.is_wall() => {},
                _ => unreachable!("bug: should not remove a wall unless it is surrounded by walls/empty tiles"),
            }
        }

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
    pub fn open_between(&mut self, pos1: TilePos, pos2: TilePos, room_sprite: SpriteImage) {
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

        if self.get(tile)
            .expect("bug: attempt to open a wall when the tile was in fact empty")
            .is_wall() {

            self.get_mut(tile)
                .expect("bug: attempt to turn an empty tile into a room tile")
                .wall_to_room(room_sprite);
        }
    }

    /// Returns an iterator over the positions of all tiles contained within this map
    pub fn tile_positions(&self) -> impl Iterator<Item=TilePos> {
        let cols = self.cols_len();
        (0..self.rows_len()).flat_map(move |row| (0..cols).map(move |col| TilePos {row, col}))
    }

    /// Returns the tile positions within the region defined by top_left and size
    pub fn tile_positions_within(&self, top_left: TilePos, size: GridSize) -> impl Iterator<Item=TilePos> {
        let start_row = top_left.row;
        let start_col = top_left.col;
        let end_row = top_left.row + size.rows - 1;
        let end_col = top_left.col + size.cols - 1;

        (start_row..=end_row).flat_map(move |row| (start_col..=end_col).map(move |col| {
            TilePos {row, col}
        }))
    }

    /// Returns the tile positions on the edges of the region defined by top_left and size
    pub fn tile_positions_on_edges(&self, top_left: TilePos, size: GridSize) -> impl Iterator<Item=TilePos> {
        let bottom_right = TilePos {
            row: top_left.row + size.rows - 1,
            col: top_left.col + size.cols - 1,
        };

        (top_left.col..top_left.col+size.cols).flat_map(
            // Top and bottom edges
            move |col| once(TilePos {row: top_left.row, col}).chain(once(TilePos {row: bottom_right.row, col}))
        // This code needs to avoid accidentally returning each corner twice
        ).chain((top_left.row+1..top_left.row+size.rows-1).flat_map(
            // Left and right edges
            move |row| once(TilePos {row, col: top_left.col}).chain(once(TilePos {row, col: bottom_right.col}))
        ))
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
        let rows = self.rows_len() as isize;
        let cols = self.cols_len() as isize;
        [(-1, 0), (0, -1), (1, 0), (0, 1)].into_iter().filter_map(move |(row_offset, col_offset)| {
            let row = row as isize + row_offset;
            let col = col as isize + col_offset;

            if row < 0 || row >= rows || col < 0 || col >= cols {
                None
            } else {
                Some(TilePos {row: row as usize, col: col as usize})
            }
        })
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
}
