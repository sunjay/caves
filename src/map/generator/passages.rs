use std::collections::HashMap;

use rand::{StdRng, Rng};

use super::MapGenerator;
use map::*;

impl MapGenerator {
    pub(in super) fn fill_passages(&self, rng: &mut StdRng, map: &mut FloorMap) {
        for row_i in 0..self.rows {
            for col_i in 0..self.cols {
                if map.is_empty((row_i, col_i)) {
                    self.generate_maze(rng, map, (row_i, col_i));
                }
            }
        }
    }

    fn generate_maze(&self, rng: &mut StdRng, map: &mut FloorMap, (row_i, col_i): (usize, usize)) {
        assert_eq!(self.passage_size, 1, "only a passage_size of 1 is supported for now");

        let mut parent_map = HashMap::new();
        let seen = map.depth_first_search_mut((row_i, col_i), |map, node, adjacents| {
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
                let (row, col) = if rng.gen() {
                    // Random horizontal edge
                    (
                        room.y() + *rng.choose(&[0, room.height()-1]).unwrap(),
                        room.x() + rng.gen_range(0, room.width()),
                    )
                } else {
                    (
                        room.y() + rng.gen_range(0, room.height()),
                        room.x() + *rng.choose(&[0, room.width()-1]).unwrap(),
                    )
                };

                debug_assert!(map.is_room_id((row, col), room_id),
                    "bug: tried to connect a passage to a room with the wrong ID");

                let adjacents: Vec<_> = map.adjacent_positions((row, col))
                    .filter(|&pt| map.is_passageway(pt))
                    .collect();
                let passage = match rng.choose(&adjacents) {
                    Some(&pt) => pt,
                    // No passage adjacent to this room tile
                    None => continue,
                };

                // Already opened this tile
                if map.is_open_between((row, col), passage) {
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

                    // Search the horizontal edge
                    if (0..room.width()).any(|col| map.adjacent_open_passages((row, room.x() + col)).next().is_some()) {
                        continue;
                    }
                    // Search the vertical edge
                    if (0..room.height()).any(|row| map.adjacent_open_passages((room.y() + row, col)).next().is_some()) {
                        continue;
                    }
                }

                map.open_between((row, col), passage);
                doors -= 1;
            }
        }
    }

    pub(in super) fn reduce_dead_ends(&self, map: &mut FloorMap) {
        for row_i in 0..self.rows {
            for col_i in 0..self.cols {
                if map.is_dead_end((row_i, col_i)) {
                    self.reduce_dead_ends_search(map, (row_i, col_i));
                }
            }
        }
    }

    fn reduce_dead_ends_search(&self, map: &mut FloorMap, (row_i, col_i): (usize, usize)) {
        map.depth_first_search_mut((row_i, col_i), |map, node, adjacents| {
            map.remove_passageway(node);

            adjacents.into_iter().filter(|&pt| map.is_dead_end(pt)).collect()
        });
    }
}
