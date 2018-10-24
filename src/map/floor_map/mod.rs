mod tile;
mod grid;
mod room;
mod tile_pos;
mod grid_size;
mod tile_rect;
mod sprite;

pub use self::tile::*;
pub use self::grid::*;
pub use self::room::*;
pub use self::tile_pos::*;
pub use self::grid_size::*;
pub use self::tile_rect::*;
pub use self::sprite::*;

use std::fmt;
use std::cmp;

use sdl2::rect::{Rect, Point};

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
    grid: TileGrid,
    /// The RoomId is the index into this field
    rooms: Vec<Room>,
    /// The width and height of every tile
    tile_size: u32,
}

impl fmt::Debug for FloorMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use colored::*;

        for row in self.grid().rows() {
            for tile in row {
                use self::Tile::*;
                write!(f, "{}", match tile {
                    &Floor {room_id, ref object, ..} => {
                        let object = object.as_ref().map(|o| o.to_string())
                            .unwrap_or_else(|| " ".to_string());

                        match self.room(room_id).room_type() {
                            RoomType::Normal => object.on_blue(),
                            RoomType::Challenge => object.on_red(),
                            RoomType::PlayerStart => object.on_bright_blue(),
                            RoomType::TreasureChamber => object.on_yellow(),
                        }
                    },
                    Wall {..} => "\u{25a2}".on_black(),
                    Empty => " ".on_black(),
                })?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

impl FloorMap {
    /// Create a new FloorMap with the given number of rows and columns
    pub fn new(size: GridSize, tile_size: u32) -> Self {
        FloorMap {
            grid: TileGrid::new(size),
            rooms: Vec::new(),
            tile_size,
        }
    }

    /// Returns the size of each tile on this map
    pub fn tile_size(&self) -> u32 {
        self.tile_size
    }

    /// Returns an iterator over the rooms in the map and their IDs
    pub fn rooms(&self) -> impl Iterator<Item=(RoomId, &Room)> {
        self.rooms.iter().enumerate().map(|(i, room)| (RoomId(i), room))
    }

    /// Returns the room with the specified room ID
    pub fn room(&self, room_id: RoomId) -> &Room {
        &self.rooms[room_id.0]
    }

    /// Returns a mutable reference to all of the rooms.
    /// Not for use after map generation is complete.
    pub(in super) fn rooms_mut(&mut self) -> &mut [Room] {
        &mut self.rooms
    }

    /// Add a room with the given rectangle to the map.
    /// Rooms should not be added after map generation is complete.
    pub(in super) fn add_room(&mut self, rect: TileRect) -> RoomId {
        let room = Room::new(rect);
        self.rooms.push(room);
        RoomId(self.rooms.len() - 1)
    }

    pub fn grid(&self) -> &TileGrid {
        &self.grid
    }

    pub(in super) fn grid_mut(&mut self) -> &mut TileGrid {
        &mut self.grid
    }

    /// Returns the tiles within (or around) the region defined by bounds
    pub fn tiles_within(&self, bounds: Rect) -> impl Iterator<Item=(Point, TilePos, &Tile)> {
        // While the caller is allowed to ask for tiles within a boundary Rect that starts at
        // negative coordinates, the top left of the map is defined as (0, 0). That means that we
        // can at most request tiles up to that top left corner. The calls to `max()` here help
        // enforce that by making sure we don't convert a negative number to an unsigned type.
        let x = cmp::max(bounds.x(), 0) as usize;
        let y = cmp::max(bounds.y(), 0) as usize;
        let width = bounds.width() as usize;
        let height = bounds.height() as usize;

        let clamp_row = |row| cmp::min(cmp::max(row, 0), self.grid().rows_len()-1);
        let clamp_col = |col| cmp::min(cmp::max(col, 0), self.grid().cols_len()-1);

        let start_row = clamp_row(y / self.tile_size as usize);
        let start_col = clamp_col(x / self.tile_size as usize);

        let end_row = clamp_row((y + height) / self.tile_size as usize);
        let end_col = clamp_col((x + width) / self.tile_size as usize);

        let rows = end_row - start_row + 1;
        let cols = end_col - start_col + 1;

        self.grid().tile_positions_within(
            TilePos {row: start_row, col: start_col},
            GridSize {rows, cols},
        ).map(move |pos| {
            // The position of the tile in world coordinates
            (pos.to_point(self.tile_size as i32), pos, self.grid().get(pos))
        })
    }
}
