use rand::{StdRng, Rng};

use super::{MapGenerator, RanOutOfAttempts};
use map::*;

impl MapGenerator {
    pub(in super) fn partition_into_rooms(
        &self,
        rng: &mut StdRng,
        sprites: &SpriteTable,
        map: &mut FloorMap,
        level: usize,
    ) -> Result<(), RanOutOfAttempts> {
        // Rectangles that must still be split
        let mut open = vec![TileRect::new(
            TilePos {row: 0, col: 0},
            GridSize {rows: self.rows, cols: self.cols},
        )];
        // Rectangles that can no longer be split
        let mut finished = Vec::new();

        let mut attempts = 0;
        while !open.is_empty() {
            if attempts > self.attempts {
                return Err(RanOutOfAttempts);
            }
            attempts += 1;

            // Find a random rectangle to split
            let rect_i = rng.gen_range(0, open.len());
            let rect = open.remove(rect_i);
            let (r1, r2) = if rng.gen() {
                rect.split_horizontal()
            } else {
                rect.split_vertical()
            };

            let GridSize {rows, cols} = r1.dimensions();
            if (self.room_rows.contains(rows) && self.room_cols.contains(cols))
                // Check if we should no longer split
                || (rows/2 < self.room_rows.min || cols/2 < self.room_cols.min) {
                finished.push(r1);
            } else {
                open.push(r1);
            }

            let GridSize {rows, cols} = r2.dimensions();
            if (self.room_rows.contains(rows) && self.room_cols.contains(cols))
                // Check if we should no longer split
                || (rows/2 < self.room_rows.min || cols/2 < self.room_cols.min) {
                finished.push(r2);
            } else {
                open.push(r2);
            }
        }

        for rect in finished {
            self.place_rect(sprites, map, rect);
        }
        self.assign_special_rooms(rng, map, level);

        Ok(())
    }

    fn assign_special_rooms(&self, rng: &mut StdRng, map: &mut FloorMap, level: usize) {
        // If we're on the first level, pick a random room for the player to start
        if level == 1 {
            let room = rng.choose_mut(map.rooms_mut()).expect("bug: should be at least one room");
            room.become_player_start();
        }

        // If we're on the last level, pick the biggest room as the treasure chamber
        if level == self.levels {
            let room = map.rooms_mut().iter_mut().max_by_key(|r| r.rect().area())
                .expect("bug: should be at least one room");
            room.become_treasure_chamber();
        }
    }

    /// Places a TileRect on the map and properly assigns its edges to be wall tiles
    pub fn place_rect(&self, sprites: &SpriteTable, map: &mut FloorMap, rect: TileRect) {
        let room_id = map.add_room(rect);

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
