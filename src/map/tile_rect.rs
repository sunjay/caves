use std::cmp;
use std::iter::once;

use sdl2::rect::Rect;
use rand::{Rng, seq::SliceRandom};

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

    /// Returns a random non-edge tile position inside the rect
    pub fn random_inner_tile<R: Rng>(self, rng: &mut R) -> TilePos {
        TilePos {
            row: self.top_left.row + rng.gen_range(1, self.dim.rows - 1),
            col: self.top_left.col + rng.gen_range(1, self.dim.cols - 1),
        }
    }

    /// Returns a random tile position on one of the horizontal (top or bottom) edges
    pub fn random_horizontal_edge_tile<R: Rng>(self, rng: &mut R) -> TilePos {
        TilePos {
            row: self.top_left.row + *[0, self.dim.rows - 1].choose(rng).unwrap(),
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
}
