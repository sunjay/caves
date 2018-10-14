use std::collections::HashMap;

use rand::{StdRng, Rng};

use super::MapGenerator;
use map::*;

impl MapGenerator {
    pub(in super) fn fill_passages(&self, rng: &mut StdRng, map: &mut FloorMap) {
        for pos in map.tile_positions() {
            if map.is_empty(pos) {
                self.generate_maze(rng, map, pos);
            }
        }
    }

    fn generate_maze(&self, rng: &mut StdRng, map: &mut FloorMap, pos: TilePos) {
        assert_eq!(self.passage_size, 1, "only a passage_size of 1 is supported for now");

        let mut parent_map = HashMap::new();
        let seen = map.depth_first_search_mut(pos, |map, node, adjacents| {
            let mut adjacents: Vec<_> = adjacents.into_iter()
                .filter(|&pt| map.is_empty(pt))
                .collect();
            rng.shuffle(&mut adjacents);

            for &adj in &adjacents {
                parent_map.insert(adj, node);
            }

            adjacents
        });

        // Insert new passageway tiles
        for pt in seen {
            map.place_tile(pt, TileType::Passageway);
        }

        // Place all of the found paths onto the tiles
        for (pt1, pt2) in parent_map {
            // Open the walls between these two cells
            map.open_between(pt1, pt2);
        }
    }

    /// Connects each room to a passage
    pub(in super) fn connect_rooms_passages(&self, rng: &mut StdRng, map: &mut FloorMap, rooms: &[(RoomId, Room)]) {
        for &(room_id, ref room) in rooms {
            let mut doors = self.doors;
            while doors > 0 {
                // Pick a random point on one of the edges of the room
                let pos = if rng.gen() {
                    room.random_horizontal_edge_tile(rng)
                } else {
                    room.random_vertical_edge_tile(rng)
                };

                debug_assert!(map.is_room_id(pos, room_id),
                    "bug: tried to connect a passage to a room with the wrong ID");

                let adjacents: Vec<_> = map.adjacent_positions(pos)
                    .filter(|&pt| map.is_passageway(pt))
                    .collect();
                let passage = match rng.choose(&adjacents) {
                    Some(&pt) => pt,
                    // No passage adjacent to this room tile
                    None => continue,
                };

                // Already opened this tile
                if map.is_open_between(pos, passage) {
                    continue;
                }

                // This check only works if we are putting fewer than 4 doors on every room
                if self.doors <= 4 {
                    // Don't put a door on the same side of a room as another door

                    // We check for this by scanning horizontally and vertically for any other
                    // doors. This approach isn't perfect though because in addition to disallowing
                    // what we don't want, it also makes it impossible for two doors to be directly
                    // opposite from each other in a room. This is not great, so in the future it
                    // would be good to improve this to be a more sophisticated check--taking
                    // taking corners into account properly. (TODO)

                    // Scan horizontally and vertically in the same row and column for this
                    // position to see if there are any open passages
                    let mut same_row_col = room.row_positions(pos.row).chain(room.col_positions(pos.col));
                    if same_row_col.any(|pos| map.adjacent_open_passages(pos).next().is_some()) {
                        continue;
                    }
                }

                map.open_between(pos, passage);
                doors -= 1;
            }
        }
    }

    pub(in super) fn reduce_dead_ends(&self, map: &mut FloorMap) {
        for pos in map.tile_positions() {
            if map.is_dead_end(pos) {
                self.reduce_dead_ends_search(map, pos);
            }
        }
    }

    fn reduce_dead_ends_search(&self, map: &mut FloorMap, pos: TilePos) {
        map.depth_first_search_mut(pos, |map, node, adjacents| {
            map.remove_passageway(node);

            adjacents.into_iter().filter(|&pt| map.is_dead_end(pt)).collect()
        });
    }
}
