mod floor_sprite;
mod wall_sprite;

pub use self::floor_sprite::*;
pub use self::wall_sprite::*;

use sdl2::rect::Rect;

use crate::assets::{SpriteId, SpriteImage, SpriteManager, TextureId};
use crate::components::Animation;

/// A lookup table for all map sprites
/// Used to avoid having to manage sprites in each tile
#[derive(Debug, Clone)]
pub struct MapSprites {
    /// Sprites for each type of floor tile. Each of these must map to a FloorSprite variant
    floor_tiles: Vec<SpriteId>,
    /// Sprites for each type of wall tile. Each of these must map to a WallSprite variant
    wall_tiles: Vec<SpriteId>,
    /// Sprites for each orientation of staircase
    staircase_up_tiles: Vec<SpriteId>,
    /// Sprites for each orientation of staircase
    staircase_down_tiles: Vec<SpriteId>,
    /// Sprites for each orientation of a door
    door_tiles: Vec<SpriteId>,
    /// The torch animation
    torch_animation: Animation,
}

impl MapSprites {
    /// Creates a table of sprites from the standard layout of the dungeon spritesheet
    pub fn from_dungeon_spritesheet(
        texture_id: TextureId,
        sprites: &mut SpriteManager,
        tile_size: u32,
    ) -> Self {
        // Adds all of the sprites to the sprite manager and returns a vector of the produced sprite IDs
        macro_rules! add_sprites {
            ($($sp:expr),* $(,)*) => (
                vec![
                    $(sprites.add($sp)),*
                ]
            );
        }

        // Returns the (tile_size)x(tile_size) sprite for the given row and column of the spritesheet
        macro_rules! tile_sprite {
            (x: $x:expr, y: $y:expr, width: $width:expr, height: $height:expr) => {
                SpriteImage::new_unflipped(texture_id, Rect::new($x, $y, $width, $height))
            };
            (row: $row:expr, col: $col:expr, width: $width:expr, height: $height:expr) => {
                tile_sprite!(
                    x: $col as i32 * tile_size as i32,
                    y: $row as i32 * tile_size as i32,
                    width: $width,
                    height: $height
                )
            };
            (row: $row:expr, col: $col:expr) => {
                tile_sprite!(row: $row, col: $col, width: tile_size, height: tile_size)
            };
        }

        Self {
            floor_tiles: add_sprites![
                tile_sprite!(row: 0, col: 0), // 1
                tile_sprite!(row: 0, col: 1), // 2
                tile_sprite!(row: 0, col: 2), // 3
                tile_sprite!(row: 0, col: 3), // 4
                tile_sprite!(row: 1, col: 0), // 5
                tile_sprite!(row: 1, col: 1), // 6
                tile_sprite!(row: 1, col: 2), // 7
                tile_sprite!(row: 1, col: 3), // 8
                tile_sprite!(row: 2, col: 0), // 9
                tile_sprite!(row: 2, col: 1), // 10
                tile_sprite!(row: 2, col: 2), // 11
                tile_sprite!(row: 2, col: 3), // 12
            ],
            wall_tiles: add_sprites![
                tile_sprite!(row: 8, col: 0),
                tile_sprite!(row: 8, col: 1),
                tile_sprite!(row: 8, col: 2),
                tile_sprite!(row: 8, col: 3),
                tile_sprite!(row: 9, col: 0),
                tile_sprite!(row: 9, col: 1),
                tile_sprite!(row: 9, col: 2),
                tile_sprite!(row: 9, col: 3),
                tile_sprite!(row: 10, col: 0),
                tile_sprite!(row: 10, col: 1),
                tile_sprite!(row: 10, col: 2),
                tile_sprite!(row: 10, col: 3),
                tile_sprite!(row: 11, col: 0),
                tile_sprite!(row: 11, col: 1),
                tile_sprite!(row: 11, col: 2),
                tile_sprite!(row: 11, col: 3),
                // Alternates

                // EW
                tile_sprite!(row: 8, col: 4),
                tile_sprite!(row: 9, col: 4),
                // NS
                tile_sprite!(row: 10, col: 4),
                tile_sprite!(row: 11, col: 4),
                // Special wall tiles

                // Brick pillar
                tile_sprite!(row: 17, col: 7, width: tile_size, height: tile_size*2).anchor_south(),
                // Torch wall (lit up/bright)
                tile_sprite!(row: 15, col: 5),
                // Entrance walls
                tile_sprite!(row: 10, col: 12), // Left
                tile_sprite!(row: 10, col: 13), // Right
            ],
            staircase_up_tiles: add_sprites![
                // bottom step faces right
                tile_sprite!(row: 15, col: 8, width: tile_size, height: tile_size*2)
                    .anchor_south()
                    .flip_horizontally(),
                // bottom step faces left
                tile_sprite!(row: 15, col: 8, width: tile_size, height: tile_size*2).anchor_south(),
            ],
            staircase_down_tiles: add_sprites![
                // top step faces right
                tile_sprite!(row: 16, col: 7),
                // top step faces left
                tile_sprite!(row: 16, col: 7).flip_horizontally(),
            ],
            door_tiles: add_sprites![
                // horizontal door (closed)
                tile_sprite!(row: 11, col: 14),
                // vertical door (closed)
                tile_sprite!(row: 10, col: 15, width: tile_size, height: tile_size*2)
                    .anchor_south(),
            ],
            torch_animation: Animation::with_constant_delay(
                &add_sprites![
                    tile_sprite!(row: 15, col: 0),
                    tile_sprite!(row: 15, col: 1),
                    tile_sprite!(row: 15, col: 2),
                    tile_sprite!(row: 15, col: 3),
                ],
                3,
                false,
                true,
            ),
        }
    }

    pub fn empty_tile_sprite(&self) -> SpriteId {
        self.floor_sprite(FloorSprite::Floor4)
    }

    pub fn floor_sprite(&self, sprite: FloorSprite) -> SpriteId {
        use self::FloorSprite::*;
        match sprite {
            Floor1 => self.floor_tiles[0],
            Floor2 => self.floor_tiles[1],
            Floor3 => self.floor_tiles[2],
            Floor4 => self.floor_tiles[3],
            Floor5 => self.floor_tiles[4],
            Floor6 => self.floor_tiles[5],
            Floor7 => self.floor_tiles[6],
            Floor8 => self.floor_tiles[7],
            Floor9 => self.floor_tiles[8],
            Floor10 => self.floor_tiles[9],
            Floor11 => self.floor_tiles[10],
            Floor12 => self.floor_tiles[11],
        }
    }

    pub fn wall_sprite(&self, sprite: WallSprite) -> SpriteId {
        macro_rules! w {
            (N: $n:pat, E: $e:pat, S: $s:pat, W: $w:pat, alt: $a:pat) => {
                WallSprite {
                    wall_north: $n,
                    wall_east: $e,
                    wall_south: $s,
                    wall_west: $w,
                    alt: $a,
                }
            };
            (N: $n:pat, E: $e:pat, S: $s:pat, W: $w:pat) => {
                w! {N: $n, E: $e, S: $s, W: $w, alt: _}
            };
            (alt: $a:pat) => {
                w! {N: _, E: _, S: _, W: _, alt: $a}
            };
        }

        let s = |n| self.wall_tiles[n];

        use self::WallSpriteAlternate::*;
        match sprite {
            w! {alt: BrickPillar} => s(20),
            w! {alt: TorchLit} => s(21),
            w! {alt: EntranceLeft} => s(22),
            w! {alt: EntranceRight} => s(23),

            w! {N: false, E: false, S: false, W: false} => s(0), // no walls adjacent

            w! {N: true, E: false, S: false, W: false} => s(12), // N
            w! {N: false, E: true, S: false, W: false} => s(1),  // E
            w! {N: false, E: false, S: true, W: false} => s(4),  // S
            w! {N: false, E: false, S: false, W: true} => s(3),  // W

            w! {N: true, E: true, S: false, W: false} => s(9), // NE
            w! {N: false, E: true, S: true, W: false} => s(5), // SE
            w! {N: false, E: false, S: true, W: true} => s(6), // SW
            w! {N: true, E: false, S: false, W: true} => s(10), // NW

            w! {N: true, E: false, S: true, W: false, alt: Alt0} => s(8), // NS
            w! {N: true, E: false, S: true, W: false, alt: Alt1} => s(18), // NS, Alt 1
            w! {N: true, E: false, S: true, W: false, alt: Alt2} => s(19), // NS, Alt 2

            w! {N: false, E: true, S: false, W: true, alt: Alt0} => s(2), // EW
            w! {N: false, E: true, S: false, W: true, alt: Alt1} => s(16), // EW, Alt 1
            w! {N: false, E: true, S: false, W: true, alt: Alt2} => s(17), // EW, Alt 2

            w! {N: true, E: true, S: true, W: false} => s(15), // NES
            w! {N: false, E: true, S: true, W: true} => s(14), // ESW
            w! {N: true, E: false, S: true, W: true} => s(13), // NSW
            w! {N: true, E: true, S: false, W: true} => s(11), // NEW

            w! {N: true, E: true, S: true, W: true} => s(7), // NESW
        }
    }

    pub fn staircase_up_right(&self) -> SpriteId {
        self.staircase_up_tiles[0]
    }

    pub fn staircase_up_left(&self) -> SpriteId {
        self.staircase_up_tiles[1]
    }

    pub fn staircase_down_right(&self) -> SpriteId {
        self.staircase_down_tiles[0]
    }

    pub fn staircase_down_left(&self) -> SpriteId {
        self.staircase_down_tiles[1]
    }

    pub fn door_horizontal(&self) -> SpriteId {
        self.door_tiles[0]
    }

    pub fn door_vertical(&self) -> SpriteId {
        self.door_tiles[1]
    }

    pub fn torch_animation(&self) -> &Animation {
        &self.torch_animation
    }
}
