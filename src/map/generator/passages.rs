use std::iter::once;
use std::collections::HashMap;

use rand::{StdRng, Rng};

use super::{MapGenerator, RanOutOfAttempts, TileGrid};
use map::*;

impl MapGenerator {
    /// Fills the map with passages by generating a maze, treating each "cell" as a
    /// (passage_size)x(passage_size) square.
    pub(in super) fn fill_passages(&self, rng: &mut StdRng, map: &mut FloorMap,
        passage_sprite: SpriteImage, passage_wall_sprite: SpriteImage) {

        assert!(self.rows % self.passage_size == 0 && self.cols % self.passage_size == 0,
            "Passage size must divide evenly into the number of rows and cols in order for maze to cover entire map");

        let passage_grid_size = GridSize {
            rows: map.grid().rows_len() / self.passage_size,
            cols: map.grid().cols_len() / self.passage_size,
        };
        let mut passages = TileGrid::new(passage_grid_size);

        let start = TilePos {row: 0, col: 0};
        let mut parent_map = HashMap::new();
        let seen = passages.depth_first_search_mut(start, |grid, node, adjacents| {
            let mut adjacents: Vec<_> = adjacents.into_iter()
                .filter(|&pt| grid.is_empty(pt))
                .collect();
            rng.shuffle(&mut adjacents);

            for &adj in &adjacents {
                parent_map.insert(adj, node);
            }

            adjacents
        });

        // Insert new passageway tiles
        let grid = map.grid_mut();
        let passage_tile_size = GridSize::square(self.passage_size);
        for pt in seen {
            // Transform the pt to be on the original grid
            let pt = pt * self.passage_size;
            for pos in grid.tile_positions_within(pt, passage_tile_size) {
                grid.place_tile(pos, TileType::Passageway, passage_sprite);
            }

            // Turn edges into walls
            for pos in grid.tile_positions_on_edges(pt, passage_tile_size) {
                grid.get_mut(pos)
                    .expect("bug: should have just placed passage tile here")
                    .become_wall(passage_wall_sprite);
            }
        }

        // Connect the paths together
        for (pt1, pt2) in parent_map {
            // Transform the pt to be on the original grid
            let pt1 = pt1 * self.passage_size;
            let pt2 = pt2 * self.passage_size;

            // There will be two sets of walls to open:
            //
            //    oooooooooooooooo
            //    o pt1  oo pt2  o
            //    oooooooooooooooo
            //
            // The two walls between pt1 and pt2 need to be removed.

            // NOTE: There is a lot of room for optimization in the following code since it does a
            // ton of redundant work.

            // Take the inner, non-wall portion of each passage square and try to open a wall
            // between it and the nearest tile in the other passage that is exactly
            // `wall_thickness` distance away
            let wall_thickness = 1;
            for (pt1, pt2) in once((pt1, pt2)).chain(once((pt2, pt1))) {
                let inner_top_left = pt1 + GridSize::square(wall_thickness);
                let inner_dimensions = passage_tile_size - GridSize::square(wall_thickness * 2);
                for inner1 in grid.tile_positions_within(inner_top_left, inner_dimensions) {
                    for pos2 in grid.tile_positions_within(pt2, passage_tile_size) {
                        // Want the other tile to be one tile after a wall
                        if inner1.is_orthogonal_difference(pos2, wall_thickness + 1) {
                            grid.open_between(inner1, pos2, passage_sprite);
                        }
                    }
                }
            }
        }
    }

    /// Connects each room to a passage
    pub(in super) fn connect_rooms_passages(
        &self,
        rng: &mut StdRng,
        map: &mut FloorMap,
        rooms: &[(RoomId, Room)],
        room_sprite: SpriteImage,
    ) -> Result<(), RanOutOfAttempts> {
        let grid = map.grid_mut();
        for &(room_id, ref room) in rooms {
            let mut doors = self.doors;
            let mut attempts = 0;
            while doors > 0 {
                if attempts > self.attempts {
                    return Err(RanOutOfAttempts);
                }
                attempts += 1;

                // Pick a random point on one of the edges of the room
                let pos = if rng.gen() {
                    room.random_horizontal_edge_tile(rng)
                } else {
                    room.random_vertical_edge_tile(rng)
                };

                debug_assert!(grid.is_room_id(pos, room_id),
                    "bug: tried to connect a passage to a room with the wrong ID");

                let adjacents: Vec<_> = grid.adjacent_positions(pos)
                    .filter(|&pt| grid.is_passageway(pt))
                    .collect();
                let passage = match rng.choose(&adjacents) {
                    Some(&pt) => pt,
                    // No passage adjacent to this room tile
                    None => continue,
                };

                // Already opened this tile
                if grid.is_open_between(pos, passage) {
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
                    if same_row_col.any(|pos| grid.adjacent_open_passages(pos).next().is_some()) {
                        continue;
                    }
                }

                grid.open_between(pos, passage, room_sprite);
                doors -= 1;
            }
        }

        Ok(())
    }

    pub(in super) fn reduce_dead_ends(&self, map: &mut FloorMap, wall_sprite: SpriteImage) {
        let grid = map.grid_mut();
        for pos in grid.tile_positions() {
            if grid.is_dead_end(pos) {
                self.reduce_dead_ends_search(grid, pos, wall_sprite);
            }
        }
    }

    fn reduce_dead_ends_search(&self, map: &mut TileGrid, pos: TilePos, wall_sprite: SpriteImage) {
        map.depth_first_search_mut(pos, |grid, node, adjacents| {
            grid.remove_passageway(node, wall_sprite);

            adjacents.into_iter().filter(|&pt| grid.is_dead_end(pt)).collect()
        });
    }
}
