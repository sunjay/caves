use std::ops::{Add, Sub};

/// Represents the dimensions of a 2D span of tiles on a grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridSize {
    pub rows: usize,
    pub cols: usize,
}

impl GridSize {
    pub fn square(size: usize) -> Self {
        Self {rows: size, cols: size}
    }
}

impl Add for GridSize {
    type Output = GridSize;

    fn add(self, other: GridSize) -> Self {
        Self {
            rows: self.rows + other.rows,
            cols: self.cols + other.cols,
        }
    }
}

impl Sub for GridSize {
    type Output = GridSize;

    fn sub(self, other: GridSize) -> Self {
        Self {
            rows: self.rows - other.rows,
            cols: self.cols - other.cols,
        }
    }
}
