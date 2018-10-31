use rand::{StdRng, Rng};

use super::{MapGenerator, WallSprite};
use map::*;

impl MapGenerator {
    pub(in super) fn layout_floor_wall_sprites(&self, rng: &mut StdRng, map: &mut FloorMap) {
        self.layout_wall_sprites(rng, map);
        self.layout_floor_sprites(rng, map);
    }

    fn layout_wall_sprites(&self, rng: &mut StdRng, map: &mut FloorMap) {
        for pos in map.grid().tile_positions() {
            if !map.grid().get(pos).is_wall() {
                continue;
            }
            // Sprite already has a predetermined alternate
            if map.grid().get(pos).wall_sprite().alt != Default::default() {
                continue;
            }

            let mut wall_sprite = WallSprite::default();
            wall_sprite.alt = rng.gen();

            for adj in map.grid().adjacent_positions(pos) {
                if !map.grid().get(adj).is_wall() {
                    continue;
                }

                match pos.difference(adj) {
                    (a, 0) if a > 0 => wall_sprite.wall_north = true,
                    (0, a) if a < 0 => wall_sprite.wall_east = true,
                    (a, 0) if a < 0 => wall_sprite.wall_south = true,
                    (0, a) if a > 0 => wall_sprite.wall_west = true,
                    _ => unreachable!("bug: position and its adjacent were not in the same row/column"),
                }
            }

            map.grid_mut().get_mut(pos).set_wall_sprite(wall_sprite);
        }

        self.layout_wall_torch_sprites(map);
    }

    fn layout_wall_torch_sprites(&self, map: &mut FloorMap) {
        // For every span of wall tiles of this size, we will try to put a torch approximately in
        // the middle of them. Only wall tiles where a torch could actually be placed count towards
        // this total.
        let torch_frequency = 4;
        // No need to add torches to last row of walls
        for row in 0..map.grid().rows_len()-1 {
            // Count of walls that could have a torch
            let mut can_torch = 0;

            for col in 0..map.grid().cols_len() {
                let pos = TilePos {row, col};
                if !map.grid().get(pos).is_wall() {
                    continue;
                }

                let has_south_floor = pos.adjacent_south(map.grid().rows_len())
                    .map(|pt| map.grid().get(pt))
                    .map(|t| t.is_floor() && !t.has_object())
                    .unwrap_or(false);
                if !has_south_floor {
                    continue;
                }

                can_torch += 1;
                if can_torch % torch_frequency == torch_frequency / 2 {
                    map.grid_mut().get_mut(pos).place_wall_torch();
                }
            }
        }
    }

    fn layout_floor_sprites(&self, rng: &mut StdRng, map: &mut FloorMap) {
        //TODO
    }
}
