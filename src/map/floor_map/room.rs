use std::cmp;
use std::iter::once;

use sdl2::rect::Rect;
use rand::Rng;

use super::{TilePos, GridSize};

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
    top_left: TilePos,
    dim: GridSize,
}

impl Room {
    /// Create a new normal room
    pub fn new(top_left: TilePos, dim: GridSize) -> Self {
        Self::with_type(RoomType::Normal, top_left, dim)
    }

    pub fn with_type(rtype: RoomType, top_left: TilePos, dim: GridSize) -> Self {
        Self {rtype, top_left, dim}
    }

    pub fn room_type(self) -> RoomType {
        self.rtype
    }

    pub fn dimensions(self) -> GridSize {
        self.dim
    }

    /// Returns the position of the top left tile within this room
    pub fn top_left(self) -> TilePos {
        self.top_left
    }

    /// Returns the position of the top right tile within this room
    pub fn top_right(self) -> TilePos {
        TilePos {
            row: self.top_left.row,
            col: self.top_left.col + self.dim.cols - 1,
        }
    }

    /// Returns the position of the bottom left tile within this room
    pub fn bottom_left(self) -> TilePos {
        TilePos {
            row: self.top_left.row + self.dim.rows - 1,
            col: self.top_left.col,
        }
    }

    /// Returns the position of the bottom right tile within this room
    pub fn bottom_right(self) -> TilePos {
        TilePos {
            row: self.top_left.row + self.dim.rows - 1,
            col: self.top_left.col + self.dim.cols - 1,
        }
    }

    /// Returns true if a room is allowed to contain ToNextLevel tiles
    pub fn can_contain_to_next_level(&self) -> bool {
        match self.rtype {
            RoomType::Normal => true,
            _ => false,
        }
    }

    /// Returns true if a room is allowed to contain ToPrevLevel tiles
    pub fn can_contain_to_prev_level(&self) -> bool {
        // currently the same as the rooms that can contain ToNextLevel
        self.can_contain_to_next_level()
    }

    pub fn is_player_start(self) -> bool {
        match self.rtype {
            RoomType::PlayerStart => true,
            _ => false,
        }
    }

    /// Returns true if the given position is a corner of this room
    pub fn is_corner(&self, pos: TilePos) -> bool {
        pos == self.top_left() ||
        pos == self.top_right() ||
        pos == self.bottom_left() ||
        pos == self.bottom_right()
    }

    /// Returns the tile position that is considered the "center" of this room.
    ///
    /// For rooms with even dimensions this will favor the top and/or left position of the "real"
    /// center of the room.
    pub fn center_tile(self) -> TilePos {
        TilePos {
            row: self.top_left.row + self.dim.rows / 2,
            col: self.top_left.col + self.dim.cols / 2,
        }
    }

    pub fn has_intersection(self, other: Self) -> bool {
        //TODO: Implement this without relying on sdl2. Perhaps based on:
        // https://github.com/servo/euclid/blob/7a4f6f77990fafc63d5fe5028df2660488e6749c/src/rect.rs#L124
        self.to_rect().has_intersection(other.to_rect())
    }

    //TODO: Remove this method. It is only used to implement certain operations using Rect. It
    // technically violates the convention that Point and Rect are for world coordinates whereas
    // Room and TilePos are for tile positions
    fn to_rect(self) -> Rect {
        Rect::new(
            self.top_left.col as i32,
            self.top_left.row as i32,
            self.dim.cols as u32,
            self.dim.rows as u32,
        )
    }

    /// Expands the room (as much as possible) to have an additional margin on all sides
    ///
    /// Will only expand up to the point (0,0). Can expand arbitrarily in the other direction.
    pub fn expand(self, margin: usize) -> Self {
        // Avoid integer overflow by only subtracting as much as possible
        let top_left_expansion = GridSize {
            rows: cmp::min(self.top_left.row, margin),
            cols: cmp::min(self.top_left.col, margin),
        };

        Self::new(
            self.top_left - top_left_expansion,
            self.dim + top_left_expansion + GridSize::square(margin),
        )
    }

    /// Returns an iterator over the positions of all tiles contained within this room
    pub fn tile_positions(self) -> impl Iterator<Item=TilePos> {
        (self.top_left.row..self.top_left.row+self.dim.rows)
            .flat_map(move |row| (self.top_left.col..self.top_left.col+self.dim.cols)
                .map(move |col| TilePos {row, col}))
    }

    /// Returns an iterator over the positions of a single row of this room
    pub fn row_positions(self, row: usize) -> impl Iterator<Item=TilePos> {
        (self.top_left.col..self.top_left.col+self.dim.cols).map(move |col| TilePos {row, col})
    }

    /// Returns an iterator over the positions of a single column of this room
    pub fn col_positions(self, col: usize) -> impl Iterator<Item=TilePos> {
        (self.top_left.row..self.top_left.row+self.dim.rows).map(move |row| TilePos {row, col})
    }

    /// Returns an iterator over all positions on an edge of the room
    pub fn edge_positions(self) -> impl Iterator<Item=TilePos> {
        let tl = self.top_left();
        let br = self.bottom_right();
        let GridSize {rows, cols} = self.dimensions();

        (tl.col..tl.col+cols).flat_map(
            // Top and bottom edges
            move |col| once(TilePos {row: tl.row, col}).chain(once(TilePos {row: br.row, col}))
        // This code needs to avoid accidentally returning each corner twice
        ).chain((tl.row+1..tl.row+rows-1).flat_map(
            // Left and right edges
            move |row| once(TilePos {row, col: tl.col}).chain(once(TilePos {row, col: br.col}))
        ))
    }

    /// Returns a random tile position on one of the horizontal (top or bottom) edges
    pub fn random_horizontal_edge_tile<R: Rng>(self, rng: &mut R) -> TilePos {
        TilePos {
            row: self.top_left.row + *rng.choose(&[0, self.dim.rows - 1]).unwrap(),
            col: self.top_left.col + rng.gen_range(0, self.dim.cols),
        }
    }

    /// Returns a random tile position on one of the vertical (left or right) edges
    pub fn random_vertical_edge_tile<R: Rng>(self, rng: &mut R) -> TilePos {
        TilePos {
            row: self.top_left.row + rng.gen_range(0, self.dim.rows),
            col: self.top_left.col + *rng.choose(&[0, self.dim.cols - 1]).unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn room_expand() {
        // Expanding a room should not go beyond (0,0) - i.e. it should avoid subtraction with
        // underflow
        let room = Room::new(TilePos {row: 0, col: 0}, GridSize {rows: 10, cols: 10});
        assert_eq!(room.expand(2), Room::new(TilePos {row: 0, col: 0}, GridSize {rows: 12, cols: 12}));
        let room = Room::new(TilePos {row: 1, col: 1}, GridSize {rows: 10, cols: 10});
        assert_eq!(room.expand(2), Room::new(TilePos {row: 0, col: 0}, GridSize {rows: 13, cols: 13}));
        let room = Room::new(TilePos {row: 2, col: 2}, GridSize {rows: 10, cols: 10});
        assert_eq!(room.expand(2), Room::new(TilePos {row: 0, col: 0}, GridSize {rows: 14, cols: 14}));
        let room = Room::new(TilePos {row: 2, col: 3}, GridSize {rows: 10, cols: 12});
        assert_eq!(room.expand(2), Room::new(TilePos {row: 0, col: 1}, GridSize {rows: 14, cols: 16}));
    }
}
