use std::collections::{HashMap, HashSet, VecDeque};

use rand::{rngs::StdRng, Rng};

use super::{GameGenerator, RanOutOfAttempts};
use map_sprites::{FloorSprite, WallSprite};
use map::*;

impl<'a> GameGenerator<'a> {
    pub(in super) fn generate_rooms(
        &self,
        rng: &mut StdRng,
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

            // Remove rooms that aren't a valid size anymore
            self.remove_invalid_rooms(&mut room_rects);
            // Remove rooms that are adjacent to each other since that can end up in cases where
            // some rooms are not reachable: https://github.com/sunjay/caves/issues/87
            self.remove_adjacent_rooms(&mut room_rects);
            // Only keep the largest group of connected rooms
            // Need to remove invalid first because that may result in more disconnected rooms
            self.remove_disconnected(&mut room_rects);
        }

        // Add the generated rooms
        for rect in room_rects {
            let room_id = map.add_room(rect);

            self.place_rect(map, room_id);
        }
        self.assign_special_rooms(rng, map, level);

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
                // Room cannot only overlap at a corner.
                // Example invalid cases:
                //
                //        ooooooo               ooooooo
                //        o     o         xxxxxxxx    o
                //  xxxxxxxoooooo         x     oxooooo
                //  x     x               xxxxxxxx
                //  xxxxxxx
                //
                //        ooooooo              ooooooo
                //        o     o        xxxxxxx     o
                //  xxxxxxxxooooo        x     xoooooo
                //  x      x             xxxxxxx
                //  xxxxxxxx
                //
                // Notice that any intersection of area within 4 falls into these cases
                if common.area() <= 4 {
                    return None;
                }

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
                let component = self.rect_graph_breadth_first_search(&graph, i);
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

    /// Removes rooms that are directly adjacent to each other
    /// This avoids some edge cases that can result in unreachable rooms.
    /// https://github.com/sunjay/caves/issues/87
    fn remove_adjacent_rooms(&self, room_rects: &mut Vec<TileRect>) {
        let mut room_i = 0;
        while room_i < room_rects.len() {
            let mut remove = false;
            for other_room in &*room_rects {
                let room = room_rects[room_i];
                // Want to remove non-intersecting adjacent rooms
                if room.has_intersection(*other_room) {
                    continue;
                }

                for edge in room.edge_positions() {
                    for other_edge in other_room.edge_positions() {
                        // Check if adjacent
                        match other_edge.difference(edge) {
                            (-1, 0) | (1, 0) | (0, -1) | (0, 1) => remove = true,
                            _ => {},
                        }
                    }
                }
            }

            if remove {
                room_rects.remove(room_i);
            } else {
                room_i += 1;
            }
        }
    }

    /// Removes rooms that have too few leftover inner tiles or rooms that have been split up by
    /// other rooms.
    ///
    /// This can happen because other rooms overlapped with the majority of tiles in this room.
    fn remove_invalid_rooms(&self, room_rects: &mut Vec<TileRect>) {
        // Using a counter + while loop to avoid iterator invalidation problems
        let mut i = 0;
        while i < room_rects.len() {
            if self.is_invalid_rect(&room_rects[i], room_rects) {
                // Room has become too small or has been split up, get rid of it
                room_rects.remove(i);
            } else {
                // Keep the room and move on
                i += 1;
            }
        }
    }

    fn is_invalid_rect(&self, room: &TileRect, room_rects: &[TileRect]) -> bool {
        let GridSize {rows, cols} = room.dimensions();
        // true = tile is uncovered by another room.
        // false = tile is covered by another room.
        // All positions in this 2D vector are offset by the top-left of the ith room. That
        // means you need to subtract the top-left to get the position in this array.
        let mut room_tiles = vec![vec![true; cols]; rows];

        let top_left = room.top_left();
        for &other_room in &*room_rects {
            if *room == other_room {
                continue;
            }
            if let Some(common) = room.intersection(other_room) {
                for pos in common.tile_positions() {
                    let local_pos = pos - top_left;
                    room_tiles[local_pos.row][local_pos.col] = false;
                }
            }
        }

        // Ensure that a room has not accidentally become segrated into chunks that are too
        // small. We can check for this by searching through the uncovered tiles and finding
        // the minimum and maximum reachable row/columns in that room.
        //
        //  0123456789
        // 0TTT_______
        // 1TTT_______
        // 2__T____TTT
        // 3TTTTTT_TTT
        //
        // Notice that the T values from the top left corner (0, 0) reach all the way to (5. 3)
        // via the path in column 2. That means that the number of rows between these points is
        // 4 and the number of columns is 6. Meanwhile, the rectangle in the bottom right
        // corner formed by (7, 2) and (9, 3) has 2 rows and 3 columns. The places marked "_"
        // are tiles where another room has overlapped. If the number of rows of either of
        // these (4 and 2) is not in the room_rows bounds or if the number of columns of either
        // of these (6 and 3) is not in the room_cols bounds, we want to remove it. The reason
        // for this is because currently, by being separated like this by overlapping rooms,
        // this room has essentially become two rooms. If either of those rooms is not the
        // right size, we want to get rid of the whole thing.
        //
        // NOTE: As implemented right now, we actually just get rid fo the whole thing if it
        // is split up like this. This was determined to be better behaviour because the room
        // rectangle doesn't really represent anything meaningful if it has been split up like
        // this. Most of the explanation above still applies so it has been left as is.

        // To find out if there are two separate components in room_tiles, we do a BFS and set
        // any true items found to false. If there are already no true items, we remove the
        // room since it has been completely overlapped. If after removing the first component
        // of true items there are still remaining tiles with true in them, there must be two
        // components since the remaining ones weren't reachable during the search.

        let first_uncovered = room_tiles.iter().enumerate()
            .find_map(|(row, r)| r.iter().enumerate()
                .find_map(|(col, &t)| if t { Some(TilePos {row, col}) } else { None }));
        if let Some(first_uncovered) = first_uncovered {
            // Find the first connected component of uncovered tiles
            let rows = room_tiles.len();
            let cols = room_tiles[0].len();

            let mut open = VecDeque::new();
            open.push_back(first_uncovered);

            let mut seen = HashSet::new();
            while let Some(node) = open.pop_front() {
                if seen.contains(&node) {
                    continue;
                }
                seen.insert(node);

                let adjacents = node.adjacent_north().into_iter()
                    .chain(node.adjacent_east(cols))
                    .chain(node.adjacent_south(rows))
                    .chain(node.adjacent_west());
                for adj in adjacents {
                    if room_tiles[adj.row][adj.col] {
                        open.push_back(adj);
                    }
                }
            }

            for pos in &seen {
                // Pretend all of these tiles are covered
                room_tiles[pos.row][pos.col] = false;
            }
            // If there are still any uncovered tiles, there must have been more than one component
            // so return invalid
            if room_tiles.iter().any(|r| r.iter().any(|&t| t)) {
                return true; // invalid room
            }

            // There is only one component. If its dimensions are too small, return invalid.
            let min_row = seen.iter().map(|pt| pt.row).min().unwrap();
            let max_row = seen.iter().map(|pt| pt.row).max().unwrap();
            let min_col = seen.iter().map(|pt| pt.col).min().unwrap();
            let max_col = seen.iter().map(|pt| pt.col).max().unwrap();
            if (max_row - min_row + 1) < self.room_rows.min || (max_col - min_col + 1) < self.room_cols.min {
                return true; // invalid room
            }

            //TODO: Should we consider resizing the room to be the size of its smallest component?
        } else {
            // No uncovered items at all
            return true; // invalid room
        }

        false // room is valid
    }

    fn rect_graph_breadth_first_search(&self, graph: &HashMap<usize, Vec<usize>>, start: usize) -> HashSet<usize> {
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

    fn assign_special_rooms(&self, rng: &mut StdRng, map: &mut FloorMap, level: usize) {
        // If we're on the first level, pick a random room for the player to start
        if level == 1 {
            let room_id = {
                let room_index = rng.gen_range(0, map.nrooms());
                let (room_id, room) = map.rooms_mut().nth(room_index).unwrap();
                room.become_player_start();
                room_id
            };
            // Put this room's tiles on top
            self.place_rect(map, room_id);
        }

        // If we're on the last level, pick the biggest room as the treasure chamber
        if level == self.levels {
            // Adjacency list representation
            let mut graph: HashMap<_, Vec<_>> = HashMap::new();

            // Create an undirected graph based on intersections
            // NOTE: Since all rooms are connected at this point, the graph should have as many
            // keys as there are rooms. All rooms should be accounted for.
            for (id1, r1) in map.rooms() {
                for (id2, r2) in map.rooms() {
                    if id1 != id2 && r1.boundary().has_intersection(*r2.boundary()) {
                        graph.entry(id1).or_default().push(id2);
                    }
                }
            }

            assert_eq!(graph.len(), map.nrooms(),
                "bug: not all rooms were added to the graph even though there should no longer be any disconnected rooms");

            // Since the treasure room is the final room of the game, it is possible for it to
            // accidentally make another room unreachable if we aren't careful in choosing it. To
            // avoid this, we first try to find the largest room that has only one other adjacent.
            // This can never make another room unreachable because it is already the end of a
            // path. If that doesn't work, all rooms must have at least 2 adjacents, so we can pick
            // the largest room and every other room will always have at least one way to get to it.
            let largest_room = graph.into_iter()
                .filter_map(|(id, adjacents)| if adjacents.len() == 1 { Some(id) } else { None })
                .max_by_key(|&id| map.room(id).boundary().area());

            let room_id = match largest_room {
                Some(room_id) => room_id,
                None => {
                    // All rooms have at least two adjacents, return the largest
                    map.rooms()
                        .max_by_key(|(_, r)| r.boundary().area())
                        .map(|(id, _)| id)
                        .expect("bug: should be at least one room")
                }
            };

            map.room_mut(room_id).become_treasure_chamber();

            // Put this room's tiles on top
            self.place_rect(map, room_id);
        }
    }

    /// Places a TileRect on the map and properly assigns its edges to be wall tiles
    fn place_rect(&self, map: &mut FloorMap, room_id: RoomId) {
        // First cover the room in floor tiles
        for pos in map.room(room_id).boundary().tile_positions() {
            map.grid_mut().place_tile(pos, Tile::new_floor(room_id, FloorSprite::default()));
        }

        // Turn the edges of the room into walls
        for edge in map.room(room_id).boundary().edge_positions() {
            map.grid_mut().get_mut(edge).become_wall(WallSprite::default());
        }
    }
}
