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
    }

    fn layout_floor_sprites(&self, rng: &mut StdRng, map: &mut FloorMap) {
        //TODO
    }
}
