use std::cmp;

use sdl2::rect::{Point, Rect};

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
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

impl Room {
    /// Create a new normal room
    pub fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self::with_type(RoomType::Normal, x, y, width, height)
    }

    pub fn with_type(rtype: RoomType, x: usize, y: usize, width: usize, height: usize) -> Self {
        Self {x, y, width, height, rtype}
    }

    pub fn room_type(self) -> RoomType { self.rtype }
    pub fn x(self) -> usize { self.x }
    pub fn y(self) -> usize { self.y }
    pub fn width(self) -> usize { self.width }
    pub fn height(self) -> usize { self.height }

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

    pub fn is_player_start(&self) -> bool {
        match self.rtype {
            RoomType::PlayerStart => true,
            _ => false,
        }
    }

    pub fn center(self) -> (usize, usize) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }

    pub fn to_rect(self) -> Rect {
        Rect::new(self.x as i32, self.y as i32, self.width as u32, self.height as u32)
    }

    pub fn has_intersection(self, other: Self) -> bool {
        //TODO: Implement this without relying on sdl2. Perhaps based on:
        // https://github.com/servo/euclid/blob/7a4f6f77990fafc63d5fe5028df2660488e6749c/src/rect.rs#L124
        self.to_rect().has_intersection(other.to_rect())
    }

    /// Expands the room (as much as possible) to have an additional margin on all sides
    ///
    /// Will only expand up to the point (0,0). Can expand arbitrarily in the other direction.
    pub fn expand(self, margin: usize) -> Self {
        // Avoid integer overflow by only subtracting as much as possible
        let top_expansion = cmp::min(self.y, margin);
        let left_expansion = cmp::min(self.x, margin);
        Self::new(
            self.x - left_expansion,
            self.y - top_expansion,
            self.width + left_expansion + margin,
            self.height + top_expansion + margin,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn room_expand() {
        // Expanding a room should not go beyond (0,0) - i.e. it should avoid subtraction with
        // underflow
        let room = Room::new(0, 0, 10, 10);
        assert_eq!(room.expand(2), Room::new(0, 0, 12, 12));
        let room = Room::new(1, 1, 10, 10);
        assert_eq!(room.expand(2), Room::new(0, 0, 13, 13));
        let room = Room::new(2, 2, 10, 10);
        assert_eq!(room.expand(2), Room::new(0, 0, 14, 14));
        let room = Room::new(3, 2, 10, 12);
        assert_eq!(room.expand(2), Room::new(1, 0, 14, 16));
    }
}
