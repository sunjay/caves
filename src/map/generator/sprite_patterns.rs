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

            let mut wall_sprite = WallSprite::default();
            wall_sprite.alt = rng.gen();

            for adj in map.grid().adjacent_positions(pos) {
                if !map.grid().get(adj).is_wall() {
                    continue;
                }

                use self::Orientation::*;
                match Orientation::face_target(pos, adj) {
                    FaceNorth => wall_sprite.wall_north = true,
                    FaceEast => wall_sprite.wall_east = true,
                    FaceSouth => wall_sprite.wall_south = true,
                    FaceWest => wall_sprite.wall_west = true,
                }
            }

            map.grid_mut().get_mut(pos).set_wall_sprite(wall_sprite);
        }
    }

    fn layout_floor_sprites(&self, rng: &mut StdRng, map: &mut FloorMap) {
        //TODO
    }
}
