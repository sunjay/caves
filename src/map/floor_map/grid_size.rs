use std::ops::{Add, Sub};

use sdl2::rect::Rect;

/// Represents the dimensions of a 2D span of tiles on a grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridSize {
    pub rows: usize,
    pub cols: usize,
}

impl GridSize {
    /// Creates a new GridSize that represents a square of the given size
    pub fn square(size: usize) -> Self {
        Self {rows: size, cols: size}
    }

    /// Converts the given grid size to a rectangle in world coordinates.
    /// The rectangle starts with its top left corner at (0, 0)
    pub fn to_rect(self, tile_size: u32) -> Rect {
        Rect::new(
            0,
            0,
            self.cols as u32 * tile_size,
            self.rows as u32 * tile_size,
        )
    }
}

impl Add for GridSize {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            rows: self.rows + other.rows,
            cols: self.cols + other.cols,
        }
    }
}

impl Sub for GridSize {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            rows: self.rows - other.rows,
            cols: self.cols - other.cols,
        }
    }
}
