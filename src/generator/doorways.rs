use std::collections::HashMap;

use rand::{rngs::StdRng, Rng};

use super::{GameGenerator, RanOutOfAttempts};
use sprites::{FloorSprite, WallSpriteAlternate};
use map::*;

impl GameGenerator {
    pub(in super) fn connect_rooms(&self, rng: &mut StdRng, map: &mut FloorMap) {
        // A mapping from the rooms that were connected to the edge tile that connected them
        let mut connected_rooms = HashMap::new();

        // Strategy: Get all possible edge wall tiles that can become doorways. Choose a
        // random edge tile and make it a doorway. Filter out any other edge that would have opened
        // a space between the same two rooms as the newly added doorway. Keep going until there
        // are no more doorways left to add.
        //
        // This algorithm guarantees that all rooms will be connected such that there is a path
        // from one room to every other room.

        // Get all potential doorways
        let mut doorways: Vec<_> = map.rooms().flat_map(|(room_id, room)| {
            let grid = map.grid();

            // Find all edges that can be turned into a doorway on this room
            room.boundary().edge_positions()
                .filter_map(move |edge| self.doorway_wall_adjacent_rooms(edge, room_id, grid)
                    .map(|pair| (edge, pair)))
        }).collect();

        while let Some(&(edge, pair)) = rng.choose(&doorways[..]) {
            connected_rooms.insert(pair, edge);

            // Only retain the doorways that connect rooms we haven't added a doorway for yet
            doorways.retain(|&(_, (r1, r2))| !connected_rooms.contains_key(&(r1, r2)) && !connected_rooms.contains_key(&(r2, r1)));
        }

        // Perform all the insertions at once (want to avoid immutable + mutable borrow)
        for ((room_id, _), edge) in connected_rooms {
            // Determine if the door should be horizontally or vertically oriented
            let mut row_walls = 0;
            let mut col_walls = 0;
            for adj in map.grid().adjacent_positions(edge) {
                if !map.grid().get(adj).is_wall() {
                    continue;
                }
                if adj.row == edge.row {
                    row_walls += 1;
                }
                if adj.col == edge.col {
                    col_walls += 1;
                }
            }
            // This code assumes that entrances are of width 1. We expect them to have walls either
            // in the same row or in the same column, never both.
            let orientation = match (row_walls, col_walls) {
                (2, 0) => HoriVert::Horizontal,
                (0, 2) => HoriVert::Vertical,
                _ => unreachable!("bug: entrance did not have expected walls"),
            };

            {
                let tile = map.grid_mut().get_mut(edge);
                tile.become_floor(room_id, FloorSprite::default());
                tile.place_object(TileObject::Door {state: Door::Closed, orientation});
            }

            if orientation == HoriVert::Horizontal {
                // Place entrance walls
                for adj in map.grid().adjacent_positions(edge) {
                    let tile = map.grid_mut().get_mut(adj);
                    if !tile.is_wall() {
                        continue;
                    }
                    tile.wall_sprite_mut().alt = if adj.col < edge.col {
                        WallSpriteAlternate::EntranceLeft
                    } else {
                        WallSpriteAlternate::EntranceRight
                    };
                }
            }
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

        Some(pair)
    }

    pub(in super) fn place_locks(&self, rng: &mut StdRng, map: &mut FloorMap) {

    }
}
