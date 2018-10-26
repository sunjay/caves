use std::collections::{HashSet, VecDeque};
use std::ops::{Index, IndexMut};
use std::iter::once;

use super::{Tile, GridSize, TilePos};

/// Represents a 2D grid of tiles
#[derive(Clone)]
pub struct TileGrid(Vec<Vec<Tile>>);

impl Index<usize> for TileGrid {
    type Output = [Tile];

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
        assert!(rows > 0 && cols > 0, "Cannot create a grid with zero rows or zero columns");
        TileGrid(vec![vec![Tile::empty(); cols]; rows])
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
    pub fn rows(&self) -> impl Iterator<Item=&[Tile]> {
        self.0.iter().map(|r| r.as_slice())
    }

    /// Gets the tile at the given position (or None if empty)
    pub fn get(&self, TilePos {row, col}: TilePos) -> &Tile {
        &self[row][col]
    }

    /// Gets the tile at the given position (or None if empty)
    pub fn get_mut(&mut self, TilePos {row, col}: TilePos) -> &mut Tile {
        &mut self[row][col]
    }

    /// Returns true if the given position is a room entrance
    /// A room entrance is defined as a floor tile that has at least one other floor tile with a
    /// different ID as one of its adjacents.
    pub fn is_room_entrance(&self, pos: TilePos) -> bool {
        match self.get(pos) {
            Tile::Floor {room_id, ..} => {
                for pos in self.adjacent_positions(pos) {
                    match self.get(pos) {
                        Tile::Floor {room_id: room_id2, ..} if room_id != room_id2 => return true,
                        _ => {},
                    }
                }
            },
            _ => {},
        }

        false
    }

    /// Places a tile with the given type at the given location
    ///
    /// Panics if that location was not previously empty
    pub fn place_tile(&mut self, TilePos {row, col}: TilePos, tile: Tile) {
        assert!(!tile.is_empty(), "bug: unsafe to place an empty tile without checking surroundings");
        self[row][col] = tile;
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

    /// Returns an iterator of tile positions adjacent to the given tile in the four cardinal
    /// directions. Only returns valid cell positions.
    pub fn adjacent_positions(&self, pos: TilePos) -> impl Iterator<Item=TilePos> {
        let rows = self.rows_len();
        let cols = self.cols_len();
        pos.adjacent_north().into_iter()
            .chain(pos.adjacent_east(cols))
            .chain(pos.adjacent_south(rows))
            .chain(pos.adjacent_west())
    }

    /// Returns an iterator of tiles adjacent to the given tile in the four cardinal directions.
    /// Only returns as many adjacents as are valid.
    pub fn adjacents(&self, pos: TilePos) -> impl Iterator<Item=&Tile> {
        self.adjacent_positions(pos).map(move |pt| self.get(pt))
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
