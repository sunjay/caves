use std::collections::HashMap;

use rand::{StdRng, Rng};

use super::{MapGenerator, RanOutOfAttempts};
use map::*;

impl MapGenerator {
    pub(in super) fn connect_rooms(&self, rng: &mut StdRng, map: &mut FloorMap) {
        let mut connected_rooms = HashMap::new();

        for (room_id, room) in map.rooms() {
            // Find all edges that can be turned into a doorway
            let edges: Vec<_> = room.boundary().edge_positions()
                .filter_map(|edge| self.doorway_wall_adjacent_rooms(edge, room_id, map.grid()).map(|pair| (edge, pair)))
                .filter(|&(_, (r1, r2))| !connected_rooms.contains_key(&(r1, r2)) && !connected_rooms.contains_key(&(r2, r1)))
                .collect();

            // If a room is only connected to one other room, there may already be a doorway from
            // that room to its adjacent. That means that we won't have any other edges to make
            // into doorways
            if let Some(&(edge, pair)) = rng.choose(&edges[..]) {
                connected_rooms.insert(pair, edge);
            }
        }

        // Perform all the insertions at once (want to avoid immutable + mutable borrow)
        for ((room_id, _), edge) in connected_rooms {
            map.grid_mut().get_mut(edge).become_floor(room_id, FloorSprite::default());
        }
    }

    /// Returns the two distinct adjacent room IDs to a potential doorway if and only if the wall
    /// that is currently at the returned position is in fact able to become a doorway
    fn doorway_wall_adjacent_rooms(&self, edge: TilePos, room_id: RoomId, grid: &TileGrid) -> Option<(RoomId, RoomId)> {
        // Due to overlap, the chosen edge may not actually be a wall anymore
        if !grid.get(edge).is_wall() {
            return None;
        }

        // To connect two rooms, we must have exactly two adjacent tiles that are in two
        // separate rooms. Note that this already rules out corners since corners only have
        // up to two floor tiles that are adjacent to them and those tiles can only ever be
        // in the same room. This also rules out wall tiles that are adjacent to an
        // existing entrance since we look for *exactly* two tiles.
        let adj_rooms: Vec<_> = grid.adjacents(edge)
            .filter_map(|adj| adj.floor_room_id())
            .collect();
        let (r1, r2) = match &adj_rooms[..] {
            [r1, r2] if r1 != r2 => (*r1, *r2),
            _ => return None,
        };

        // Make sure we are actually connecting the room we intended (room_id) to some other room
        let pair = if r1 == room_id {
            (room_id, r2)
        } else if r2 == room_id {
            (room_id, r1)
        } else {
            return None;
        };

        Some((r1, r2))
    }

    pub(in super) fn place_locks(&self, rng: &mut StdRng, map: &mut FloorMap) {

    }
}
