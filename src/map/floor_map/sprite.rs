use sdl2::rect::Rect;
use rand::{
    Rng,
    distributions::{
        Distribution,
        Standard,
    },
};

use texture_manager::TextureId;
use super::Orientation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Anchor {
    North,
    East,
    South,
    West,
    Center,
}

/// Represents an image/texture that will be renderered
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteImage {
    /// The spritesheet to pull the image from
    pub texture_id: TextureId,
    /// The region of the spritesheet to use, unrelated to the actual bounding box
    pub region: Rect,
    /// Whether to flip the sprite along the horizontal axis
    pub flip_horizontal: bool,
    /// Whether to flip the sprite along the vertical axis
    pub flip_vertical: bool,
    /// The position within the region at which this sprite is anchored
    pub anchor: Anchor,
}

impl SpriteImage {
    /// Creates a new SpriteImage that is not flipped either horizontally or vertically
    pub fn new_unflipped(texture_id: TextureId, region: Rect) -> Self {
        SpriteImage {
            texture_id,
            region,
            flip_horizontal: false,
            flip_vertical: false,
            anchor: Anchor::Center,
        }
    }

    /// Returns this sprite image flipped horizontally
    pub fn flip_horizontally(self) -> Self {
        Self {
            flip_horizontal: !self.flip_horizontal,
            ..self
        }
    }

    /// Returns this sprite image flipped vertically
    pub fn flip_vertically(self) -> Self {
        Self {
            flip_vertical: !self.flip_vertical,
            ..self
        }
    }

    /// Returns this sprite image anchored from its south side
    pub fn anchor_south(self) -> Self {
        Self {
            anchor: Anchor::South,
            ..self
        }
    }
}

/// Used to decouple SpriteImage from a specific SpriteTable
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloorSprite {
    Type1,
}

impl Default for FloorSprite {
    fn default() -> Self {
        // Need a default floor tile sprite because we can't determine the actual floor tile sprite to
        // use until after all of the tiles are placed.
        FloorSprite::Type1
    }
}

/// Used to decouple SpriteImage from a specific SpriteTable
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct WallSprite {
    /// true if there is another wall tile to the north of this one
    pub wall_north: bool,
    /// true if there is another wall tile to the east of this one
    pub wall_east: bool,
    /// true if there is another wall tile to the south of this one
    pub wall_south: bool,
    /// true if there is another wall tile to the west of this one
    pub wall_west: bool,
    /// the variant of the sprite to use
    pub alt: WallSpriteAlternate,
}

/// Different alternate wall styles for some of the wall sprites
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallSpriteAlternate {
    Alt0,
    Alt1,
    Alt2,
}

impl Default for WallSpriteAlternate {
    fn default() -> Self {
        WallSpriteAlternate::Alt0
    }
}

impl Distribution<WallSpriteAlternate> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> WallSpriteAlternate {
        use self::WallSpriteAlternate::*;
        match rng.gen_range(0, 3) {
            0 => Alt0,
            1 => Alt1,
            2 => Alt2,
            _ => unreachable!(),
        }
    }
}

/// A lookup table for all map sprites
/// Used to avoid having to manage sprites in each tile
#[derive(Debug, Clone)]
pub struct MapSprites {
    /// Sprite for Tile::Empty
    empty_tile_sprite: SpriteImage,
    /// Sprites for each type of floor tile. Each of these must map to a FloorSprite variant
    floor_tiles: Vec<SpriteImage>,
    /// Sprites for each type of wall tile. Each of these must map to a WallSprite variant
    wall_tiles: Vec<SpriteImage>,
    /// Sprites for each orientation of staircase
    staircase_up_tiles: Vec<SpriteImage>,
    /// Sprites for each orientation of staircase
    staircase_down_tiles: Vec<SpriteImage>,
}

impl MapSprites {
    /// Creates a table of sprites from the standard layout of the dungeon spritesheet
    pub fn from_dungeon_spritesheet(texture_id: TextureId, tile_size: u32) -> Self {
        // Returns the (tile_size)x(tile_size) sprite for the given row and column of the spritesheet
        let tile_sprite = |row, col| SpriteImage::new_unflipped(
            texture_id,
            Rect::new(
                col as i32 * tile_size as i32,
                row as i32 * tile_size as i32,
                tile_size,
                tile_size,
            ),
        );

        Self {
            empty_tile_sprite: tile_sprite(0, 3),
            floor_tiles: vec![tile_sprite(0, 0)],
            wall_tiles: vec![
                tile_sprite(8, 0),
                tile_sprite(8, 1),
                tile_sprite(8, 2),
                tile_sprite(8, 3),

                tile_sprite(9, 0),
                tile_sprite(9, 1),
                tile_sprite(9, 2),
                tile_sprite(9, 3),

                tile_sprite(10, 0),
                tile_sprite(10, 1),
                tile_sprite(10, 2),
                tile_sprite(10, 3),

                tile_sprite(11, 0),
                tile_sprite(11, 1),
                tile_sprite(11, 2),
                tile_sprite(11, 3),

                // Alternates

                // EW
                tile_sprite(8, 4),
                tile_sprite(9, 4),
                // NS
                tile_sprite(10, 4),
                tile_sprite(11, 4),
            ],
            staircase_up_tiles: vec![
                // bottom step faces east
                tile_sprite(16, 8).anchor_south().flip_horizontally(),
                // bottom step faces west
                tile_sprite(16, 8).anchor_south(),
            ],
            staircase_down_tiles: vec![
                // bottom step faces east
                tile_sprite(16, 7).flip_horizontally(),
                // bottom step faces west
                tile_sprite(16, 7),
            ],
        }
    }

    pub fn empty_tile_sprite(&self) -> &SpriteImage {
        &self.empty_tile_sprite
    }

    pub fn floor_sprite(&self, sprite: FloorSprite) -> &SpriteImage {
        use self::FloorSprite::*;
        match sprite {
            Type1 => &self.floor_tiles[0],
        }
    }

    pub fn wall_sprite(&self, sprite: WallSprite) -> &SpriteImage {
        macro_rules! w {
            (N: $n:pat, E: $e:pat, S: $s:pat, W: $w:pat, alt: $a:pat) => {
                WallSprite {wall_north: $n, wall_east: $e, wall_south: $s, wall_west: $w, alt: $a}
            };
            (N: $n:pat, E: $e:pat, S: $s:pat, W: $w:pat) => {
                w!{N: $n, E: $e, S: $s, W: $w, alt: _}
            };
        }

        let s = |n| &self.wall_tiles[n];

        use self::WallSpriteAlternate::*;
        match sprite {
            w!{N: false, E: false, S: false, W: false} => s(0), // no walls adjacent

            w!{N: true, E: false, S: false, W: false} => s(12), // N
            w!{N: false, E: true, S: false, W: false} => s(1), // E
            w!{N: false, E: false, S: true, W: false} => s(4), // S
            w!{N: false, E: false, S: false, W: true} => s(3), // W

            w!{N: true, E: true, S: false, W: false} => s(9), // NE
            w!{N: false, E: true, S: true, W: false} => s(5), // SE
            w!{N: false, E: false, S: true, W: true} => s(6), // SW
            w!{N: true, E: false, S: false, W: true} => s(10), // NW

            w!{N: true, E: false, S: true, W: false, alt: Alt0} => s(8), // NS
            w!{N: true, E: false, S: true, W: false, alt: Alt1} => s(18), // NS, Alt 1
            w!{N: true, E: false, S: true, W: false, alt: Alt2} => s(19), // NS, Alt 2

            w!{N: false, E: true, S: false, W: true, alt: Alt0} => s(2), // EW
            w!{N: false, E: true, S: false, W: true, alt: Alt1} => s(16), // EW, Alt 1
            w!{N: false, E: true, S: false, W: true, alt: Alt2} => s(17), // EW, Alt 2

            w!{N: true, E: true, S: true, W: false} => s(15), // NES
            w!{N: false, E: true, S: true, W: true} => s(14), // ESW
            w!{N: true, E: false, S: true, W: true} => s(13), // NSW
            w!{N: true, E: true, S: false, W: true} => s(11), // NEW

            w!{N: true, E: true, S: true, W: true} => s(7), // NESW
        }
    }

    pub fn staircase_up_sprite(&self, orientation: Orientation) -> &SpriteImage {
        use self::Orientation::*;
        match orientation {
            FaceEast => &self.staircase_up_tiles[0],
            FaceWest => &self.staircase_up_tiles[1],
            FaceNorth | FaceSouth => unreachable!("bug: no sprites for staircases facing north/south"),
        }
    }

    pub fn staircase_down_sprite(&self, orientation: Orientation) -> &SpriteImage {
        use self::Orientation::*;
        match orientation {
            FaceEast => &self.staircase_down_tiles[0],
            FaceWest => &self.staircase_down_tiles[1],
            FaceNorth | FaceSouth => unreachable!("bug: no sprites for staircases facing north/south"),
        }
    }
}
