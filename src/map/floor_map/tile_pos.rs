use std::ops::{Add, Sub};

use super::GridSize;

/// Represents the location of a single tile in a 2D grid of tiles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TilePos {
    pub row: usize,
    pub col: usize,
}

impl TilePos {
    /// Returns the difference between this position and another position
    /// This is like self - other, but negative values are allowed
    /// Returns (delta row, delta col)
    pub fn difference(self, other: TilePos) -> (isize, isize) {
        (self.row as isize - other.row as isize, self.col as isize - other.col as isize)
    }
}

impl Add<GridSize> for TilePos {
    type Output = TilePos;

    fn add(self, other: GridSize) -> Self {
        Self {
            row: self.row + other.rows,
            col: self.col + other.cols,
        }
    }
}

impl Sub<GridSize> for TilePos {
    type Output = TilePos;

    fn sub(self, other: GridSize) -> Self {
        Self {
            row: self.row - other.rows,
            col: self.col - other.cols,
        }
    }
}
