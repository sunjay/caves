use std::collections::{HashMap, HashSet, VecDeque};

use rand::{StdRng, Rng};

use super::{MapGenerator, RanOutOfAttempts};
use map::*;

impl MapGenerator {
    pub(in super) fn generate_rooms(
        &self,
        rng: &mut StdRng,
        sprites: &SpriteTable,
        map: &mut FloorMap,
        level: usize,
    ) -> Result<(), RanOutOfAttempts> {
        let nrooms = self.rooms.gen(rng);

        let mut room_rects = Vec::new();

        let mut attempts = 0;
        while room_rects.len() < nrooms {
            for _ in 0..nrooms {
                'gen_room: loop {
                    if attempts > self.attempts {
                        return Err(RanOutOfAttempts);
                    }
                    attempts += 1;

                    if let Some(rect) = self.random_room(rng, &room_rects) {
                        room_rects.push(rect);
                        break; // Added 1 room
                    }
                }
            }

            // Only keep the largest group of connected rooms
            self.remove_disconnected(&mut room_rects);
        }

        // Add the generated rooms
        for rect in room_rects {
            let room_id = map.add_room(rect);

            self.place_rect(sprites, map, room_id, rect);
        }
        self.assign_special_rooms(rng, sprites, map, level);

        Ok(())
    }

    // Generates and validates a random room for placement on the map
    // Only returns the room if it could be placed
    fn random_room(&self, rng: &mut StdRng, room_rects: &[TileRect]) -> Option<TileRect> {
        let rect = TileRect::new(
            TilePos {
                row: rng.gen_range(0, self.rows),
                col: rng.gen_range(0, self.cols),
            },
            GridSize {
                rows: self.room_rows.gen(rng),
                cols: self.room_cols.gen(rng),
            },
        );

        // Room cannot be out of bounds
        let bottom_right = rect.bottom_right();
        if bottom_right.row >= self.rows || bottom_right.col >= self.cols {
            return None;
        }

        for &rect2 in room_rects {
            if let Some(common) = rect.intersection(rect2) {
                // Room cannot take over max_overlap% of the area of another room
                if common.area() as f64 / rect2.area() as f64 > self.max_overlap {
                    return None;
                }

                // Other room cannot take over max_overlap% of the room
                if common.area() as f64 / rect.area() as f64 > self.max_overlap {
                    return None;
                }
            }

            // Cannot share an edge with another room
            let tl = rect.top_left();
            let tl2 = rect2.top_left();
            let br = rect.bottom_right();
            let br2 = rect2.bottom_right();
            if tl.row == tl2.row || tl.col == tl2.col
                || br.row == br2.row || br.col == br2.col {
                return None;
            }
        }

        Some(rect)
    }

    /// Removes rooms that are not connected to the other rooms.
    ///
    /// The insight here is that we can build an undirected graph from the rooms based on which
    /// other rooms they intersect with. We can use an algorithm for finding connected components
    /// and keep all of the rooms that are part of the largest connected component.
    fn remove_disconnected(&self, room_rects: &mut Vec<TileRect>) {
        // Adjacency list representation
        let mut graph: HashMap<_, Vec<_>> = HashMap::new();

        // Create an undirected graph based on intersections
        // NOTE: this does not guarantee that all room indexes will get an entry in the graph
        // variable. To deal with that and make sure that all rooms are accounted for, we have to
        // go through 0..room_rects.len() below instead of something like graph.keys().
        for (i, r1) in room_rects.iter().enumerate() {
            for (j, r2) in room_rects.iter().enumerate() {
                if i != j && r1.has_intersection(*r2) {
                    graph.entry(i).or_default().push(j);
                }
            }
        }

        // Find all of the connected components
        let mut components: Vec<HashSet<_>> = Vec::new();
        // Must go through all room indexes to make sure we don't forget to account for a node
        for i in 0..room_rects.len() {
            // If i is not part of any connected component, create one for it
            if !components.iter().any(|c| c.contains(&i)) {
                let component = self.rect_graph_depth_first_search(&graph, i);
                components.push(component);
            }
        }

        // Keep the maximum component
        let (max, _) = components.iter().enumerate().max_by_key(|(_, c)| c.len())
            .expect("bug: expected at least one component");
        components.remove(max);

        // Remove all other components
        let mut remove: Vec<_> = components.into_iter().flat_map(|s| s.into_iter()).collect();
        // NOTE: must do this in descending order so that the indexes stay valid throughout removal
        remove.sort_by(|a, b| b.cmp(a));
        for room_index in remove {
            room_rects.remove(room_index);
        }
    }

    fn rect_graph_depth_first_search(&self, graph: &HashMap<usize, Vec<usize>>, start: usize) -> HashSet<usize> {
        let mut open = VecDeque::new();
        open.push_back(start);

        let mut seen = HashSet::new();

        while let Some(node) = open.pop_front() {
            if seen.contains(&node) {
                continue;
            }
            seen.insert(node);

            // Some nodes may not have any adjacents
            if let Some(adjacents) = graph.get(&node) {
                for &adj in adjacents {
                    open.push_back(adj);
                }
            }
        }

        seen
    }

    fn assign_special_rooms(&self, rng: &mut StdRng, sprites: &SpriteTable, map: &mut FloorMap, level: usize) {
        // If we're on the first level, pick a random room for the player to start
        if level == 1 {
            let (room_id, rect) = {
                let room_index = rng.gen_range(0, map.nrooms());
                let (room_id, room) = map.rooms_mut().nth(room_index).unwrap();
                room.become_player_start();
                (room_id, *room.rect())
            };
            // Put this room's tiles on top
            self.place_rect(sprites, map, room_id, rect);
        }

        // If we're on the last level, pick the biggest room as the treasure chamber
        if level == self.levels {
            let (room_id, rect) = {
                let (room_id, room) = map.rooms_mut()
                    .max_by_key(|(_, r)| r.rect().area())
                    .expect("bug: should be at least one room");
                room.become_treasure_chamber();
                (room_id, *room.rect())
            };
            // Put this room's tiles on top
            self.place_rect(sprites, map, room_id, rect);
        }
    }

    /// Places a TileRect on the map and properly assigns its edges to be wall tiles
    pub fn place_rect(&self, sprites: &SpriteTable, map: &mut FloorMap, room_id: RoomId, rect: TileRect) {
        // First cover the room in floor tiles
        for pos in map.room(room_id).rect().tile_positions() {
            map.grid_mut().place_tile(pos, Tile::new_floor(room_id, sprites.default_floor_tile_index));
        }

        // Turn the edges of the room into walls
        for edge in map.room(room_id).rect().edge_positions() {
            map.grid_mut().get_mut(edge).become_wall(sprites.default_wall_tile_index);
        }
    }
}
