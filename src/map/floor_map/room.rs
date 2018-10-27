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

/// Represents a "room" on the map separated from other rooms by walls/entrances. Rooms are allowed
/// to overlap, so the boundary of the room only represents the extent of where tiles may be within
/// the room. Not all tiles within the boundary are guaranteed to be part of this particular room.
#[derive(Debug, Clone, PartialEq)]
pub struct Room {
    rtype: RoomType,
    boundary: TileRect,
}

impl Room {
    /// Create a new normal room
    pub(in super) fn new(boundary: TileRect) -> Self {
        Self {rtype: RoomType::Normal, boundary}
    }

    pub fn room_type(&self) -> RoomType {
        self.rtype
    }

    /// The rectangular boundary of the room. Since rooms are allowed to overlap, all tiles within
    /// this boundary may not be part of this room.
    pub fn boundary(&self) -> &TileRect {
        &self.boundary
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
