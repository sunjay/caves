use super::{TileRect};

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

/// Represents a rectangular group of tiles. The outermost border of the rectangle is made up of
/// wall tiles and passages (floor tiles) to adjoining rooms. Rooms can overlap at their outer
/// walls. In other words, this means that the rectangles of two rooms may share a wall at their
/// border.
#[derive(Debug, Clone)]
pub struct Room {
    rtype: RoomType,
    rect: TileRect,
}

impl Room {
    /// Create a new normal room
    pub(in super) fn new(rect: TileRect) -> Self {
        Self {rtype: RoomType::Normal, rect}
    }

    pub fn room_type(&self) -> RoomType {
        self.rtype
    }

    pub fn rect(&self) -> &TileRect {
        &self.rect
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

    /// Returns true if this room is the room that the player starts in
    pub fn is_player_start(&self) -> bool {
        match self.rtype {
            RoomType::PlayerStart => true,
            _ => false,
        }
    }

    /// Turns this room into the player start room
    pub(in map) fn become_player_start(&mut self) {
        self.rtype = RoomType::PlayerStart;
    }

    /// Turns this room into the treasure chamber
    pub(in map) fn become_treasure_chamber(&mut self) {
        self.rtype = RoomType::TreasureChamber;
    }
}
