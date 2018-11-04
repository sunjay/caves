use std::cmp;
use std::iter::once;

use sdl2::rect::Rect;
use rand::Rng;

use super::{TilePos, GridSize};

/// A 2D span of tiles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileRect {
    top_left: TilePos,
    dim: GridSize,
}

impl TileRect {
    /// Create a new rectangle with the given top left and dimensions
    pub fn new(top_left: TilePos, dim: GridSize) -> Self {
        Self {top_left, dim}
    }

    /// Returns the dimensions of the rectangle
    pub fn dimensions(self) -> GridSize {
        self.dim
    }

    /// Returns the area of this rectangle in tiles^2
    pub fn area(self) -> usize {
        self.dim.rows * self.dim.cols
    }

    /// Returns the position of the top left tile within this rectangle
    pub fn top_left(self) -> TilePos {
        self.top_left
    }

    /// Returns the position of the top right tile within this rectangle
    pub fn top_right(self) -> TilePos {
        TilePos {
            row: self.top_left.row,
            col: self.top_left.col + self.dim.cols - 1,
        }
    }

    /// Returns the position of the bottom left tile within this rectangle
    pub fn bottom_left(self) -> TilePos {
        TilePos {
            row: self.top_left.row + self.dim.rows - 1,
            col: self.top_left.col,
        }
    }

    /// Returns the position of the bottom right tile within this rectangle
    pub fn bottom_right(self) -> TilePos {
        TilePos {
            row: self.top_left.row + self.dim.rows - 1,
            col: self.top_left.col + self.dim.cols - 1,
        }
    }

    /// Returns true if the given position is a corner of this rectangle
    pub fn is_corner(&self, pos: TilePos) -> bool {
        pos == self.top_left() ||
        pos == self.top_right() ||
        pos == self.bottom_left() ||
        pos == self.bottom_right()
    }

    /// Returns the tile position that is considered the "center" of this rectangle.
    ///
    /// If the exact center is not a valid tile position (i.e. it is between 4 tiles), then this
    /// will bias towards the bottom right of the center tile.
    pub fn center_tile(self) -> TilePos {
        TilePos {
            row: self.top_left.row + self.dim.rows / 2,
            col: self.top_left.col + self.dim.cols / 2,
        }
    }

    pub fn intersection(self, other: Self) -> Option<TileRect> {
        //TODO: Implement this without relying on sdl2. Perhaps based on:
        // https://github.com/servo/euclid/blob/7a4f6f77990fafc63d5fe5028df2660488e6749c/src/rect.rs#L124
        self.to_rect().intersection(other.to_rect()).map(|r| {
            let tl = r.top_left();
            Self::new(
                TilePos {
                    row: tl.y() as usize,
                    col: tl.x() as usize,
                },
                GridSize {
                    rows: r.height() as usize,
                    cols: r.width() as usize,
                }
            )
        })
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

    /// Expands the rectangle (as much as possible) to have an additional margin on all sides
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

    /// Returns an iterator over the positions of all tiles contained within this rectangle
    pub fn tile_positions(self) -> impl Iterator<Item=TilePos> {
        (self.top_left.row..self.top_left.row+self.dim.rows)
            .flat_map(move |row| (self.top_left.col..self.top_left.col+self.dim.cols)
                .map(move |col| TilePos {row, col}))
    }

    /// Returns an iterator over the positions of a single row of this rectangle
    pub fn row_positions(self, row: usize) -> impl Iterator<Item=TilePos> {
        (self.top_left.col..self.top_left.col+self.dim.cols).map(move |col| TilePos {row, col})
    }

    /// Returns an iterator over the positions of a single column of this rectangle
    pub fn col_positions(self, col: usize) -> impl Iterator<Item=TilePos> {
        (self.top_left.row..self.top_left.row+self.dim.rows).map(move |row| TilePos {row, col})
    }

    /// Returns an iterator over all positions on an edge of the rectangle
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
        if rng.gen() {
            self.random_left_vertical_edge_tile(rng)
        } else {
            self.random_right_vertical_edge_tile(rng)
        }
    }

    /// Returns a random tile position on the left vertical edge
    pub fn random_left_vertical_edge_tile<R: Rng>(self, rng: &mut R) -> TilePos {
        TilePos {
            row: self.top_left.row + rng.gen_range(0, self.dim.rows),
            col: self.top_left.col,
        }
    }

    /// Returns a random tile position on the right vertical edge
    pub fn random_right_vertical_edge_tile<R: Rng>(self, rng: &mut R) -> TilePos {
        TilePos {
            row: self.top_left.row + rng.gen_range(0, self.dim.rows),
            col: self.top_left.col + self.dim.cols - 1,
        }
    }

    /// Split the rectangle along the horizontal axis producing two rectangles that share their
    /// bottom and top edge.
    pub fn split_horizontal(self) -> (Self, Self) {
        let GridSize {rows, cols} = self.dim;
        let top = Self {
            top_left: self.top_left,
            dim: GridSize {
                rows: rows / 2,
                cols,
            },
        };
        let bottom = Self {
            top_left: self.top_left + GridSize {rows: rows/2 - 1, cols: 0},
            dim: GridSize {
                rows: rows - rows / 2 + 1,
                cols,
            },
        };

        (top, bottom)
    }

    /// Split the rectangle along the vertical axis producing two rectangles that share their
    /// right and left edge.
    pub fn split_vertical(self) -> (Self, Self) {
        let GridSize {rows, cols} = self.dim;
        let left = Self {
            top_left: self.top_left,
            dim: GridSize {
                rows,
                cols: cols / 2,
            },
        };
        let right = Self {
            top_left: self.top_left + GridSize {rows: 0, cols: cols/2 - 1},
            dim: GridSize {
                rows,
                cols: cols - cols / 2 + 1,
            },
        };

        (left, right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center() {
        let rect = TileRect::new(TilePos {row: 1, col: 2}, GridSize {rows: 11, cols: 9});
        // Want to be directly in the center
        assert_eq!(rect.center_tile(), TilePos {row: 6, col: 6});
        let rect = TileRect::new(TilePos {row: 1, col: 2}, GridSize {rows: 12, cols: 14});
        // Want to bias towards the bottom-right of the center
        assert_eq!(rect.center_tile(), TilePos {row: 7, col: 9});
    }

    #[test]
    fn rectangle_expand() {
        // Expanding a rectangle should not go beyond (0,0) - i.e. it should avoid subtraction with
        // underflow
        let rect = TileRect::new(TilePos {row: 0, col: 0}, GridSize {rows: 10, cols: 10});
        assert_eq!(rect.expand(2), TileRect::new(TilePos {row: 0, col: 0}, GridSize {rows: 12, cols: 12}));
        let rect = TileRect::new(TilePos {row: 1, col: 1}, GridSize {rows: 10, cols: 10});
        assert_eq!(rect.expand(2), TileRect::new(TilePos {row: 0, col: 0}, GridSize {rows: 13, cols: 13}));
        let rect = TileRect::new(TilePos {row: 2, col: 2}, GridSize {rows: 10, cols: 10});
        assert_eq!(rect.expand(2), TileRect::new(TilePos {row: 0, col: 0}, GridSize {rows: 14, cols: 14}));
        let rect = TileRect::new(TilePos {row: 2, col: 3}, GridSize {rows: 10, cols: 12});
        assert_eq!(rect.expand(2), TileRect::new(TilePos {row: 0, col: 1}, GridSize {rows: 14, cols: 16}));
    }

    #[test]
    fn split() {
        let rect = TileRect::new(TilePos {row: 0, col: 0}, GridSize {rows: 11, cols: 15});
        let center = rect.center_tile();

        {
            let (top, bottom) = rect.split_horizontal();
            // Should share their bottom and top edge
            assert_eq!(top.bottom_left(), bottom.top_left());
            assert_eq!(top.bottom_right(), bottom.top_right());
            // Should still reach the original rectangle
            assert_eq!(top.top_left(), rect.top_left());
            assert_eq!(top.top_right(), rect.top_right());
            assert_eq!(bottom.bottom_left(), rect.bottom_left());
            assert_eq!(bottom.bottom_right(), rect.bottom_right());
            // Should overlap horizontally
            assert_eq!(top.dimensions().rows + bottom.dimensions().rows, rect.dimensions().rows + 1);
            // Check the exact rectangles
            assert_eq!(top, TileRect::new(rect.top_left(), GridSize {rows: 5, cols: 15}));
            assert_eq!(bottom, TileRect::new(TilePos {row: center.row - 1, col: 0}, GridSize {rows: 7, cols: 15}));
        }

        {
            let (left, right) = rect.split_vertical();
            // Should share their right and left edge
            assert_eq!(left.top_right(), right.top_left());
            assert_eq!(left.bottom_right(), right.bottom_left());
            // Should still reach the original rectangle
            assert_eq!(left.top_left(), rect.top_left());
            assert_eq!(right.top_right(), rect.top_right());
            assert_eq!(left.bottom_left(), rect.bottom_left());
            assert_eq!(right.bottom_right(), rect.bottom_right());
            // Should overlap vertically
            assert_eq!(left.dimensions().cols + right.dimensions().cols, rect.dimensions().cols + 1);
            // Check the exact rectangles
            assert_eq!(left, TileRect::new(rect.top_left(), GridSize {rows: 11, cols: 7}));
            assert_eq!(right, TileRect::new(TilePos {row: 0, col: center.col - 1}, GridSize {rows: 11, cols: 9}));
        }
    }
}
