use std::ops::{Add, Sub, Mul};

use sdl2::rect::{Point, Rect};

use super::GridSize;

/// Represents the location of a single tile in a 2D grid of tiles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TilePos {
    pub row: usize,
    pub col: usize,
}

impl TilePos {
    /// Returns the rectangle in world coordinates that is covered by this tile
    /// Maps columns to x-coordinates and rows to y-coordinates
    pub fn tile_rect(self, tile_size: u32) -> Rect {
        let top_left = self.top_left(tile_size as i32);
        Rect::new(
            top_left.x(),
            top_left.y(),
            tile_size,
            tile_size,
        )
    }

    /// Returns the position in world coordinates of the center of this tile
    /// Maps columns to x-coordinates and rows to y-coordinates
    pub fn center(self, tile_size: i32) -> Point {
        self.top_left(tile_size).offset(tile_size / 2, tile_size / 2)
    }

    /// Returns the position in world coordinates of the bottom right of this tile
    /// Maps columns to x-coordinates and rows to y-coordinates
    pub fn bottom_right(self, tile_size: i32) -> Point {
        self.top_left(tile_size).offset(tile_size, tile_size)
    }

    /// Returns the position in world coordinates of the top left of this tile
    /// Maps columns to x-coordinates and rows to y-coordinates
    pub fn top_left(self, tile_size: i32) -> Point {
        // It's easy to mix up the ordering in this Point constructor, so this method
        // helps avoid that in some cases.
        Point::new(
            self.col as i32 * tile_size,
            self.row as i32 * tile_size,
        )
    }

    /// Returns the position one tile north of this position, if any
    pub fn adjacent_north(self) -> Option<TilePos> {
        self.row.checked_sub(1).map(|row| TilePos {row, col: self.col})
    }

    /// Returns the position one tile east of this position, if any
    pub fn adjacent_east(self, ncols: usize) -> Option<TilePos> {
        if self.col < ncols - 1 {
            Some(TilePos {
                row: self.row,
                col: self.col + 1,
            })
        } else {
            None
        }
    }

    /// Returns the position one tile south of this position, if any
    pub fn adjacent_south(self, nrows: usize) -> Option<TilePos> {
        if self.row < nrows - 1 {
            Some(TilePos {
                row: self.row + 1,
                col: self.col,
            })
        } else {
            None
        }
    }

    /// Returns the position one tile west of this position, if any
    pub fn adjacent_west(self) -> Option<TilePos> {
        self.col.checked_sub(1).map(|col| TilePos {row: self.row, col})
    }

    /// Returns the difference between this position and another position
    /// This is like self - other, but negative values are allowed
    /// Returns (delta row, delta col)
    pub fn difference(self, other: Self) -> (isize, isize) {
        (self.row as isize - other.row as isize, self.col as isize - other.col as isize)
    }
}

impl Add<GridSize> for TilePos {
    type Output = Self;

    fn add(self, other: GridSize) -> Self {
        Self {
            row: self.row + other.rows,
            col: self.col + other.cols,
        }
    }
}

impl Sub<GridSize> for TilePos {
    type Output = Self;

    fn sub(self, other: GridSize) -> Self {
        Self {
            row: self.row - other.rows,
            col: self.col - other.cols,
        }
    }
}

// Subtraction makes sense to implement since you can have points relative to other points
impl Sub for TilePos {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            row: self.row - other.row,
            col: self.col - other.col,
        }
    }
}

impl Mul<usize> for TilePos {
    type Output = Self;

    fn mul(self, factor: usize) -> Self {
        Self {
            row: self.row * factor,
            col: self.col * factor,
        }
    }
}
