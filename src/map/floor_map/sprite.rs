use sdl2::rect::{Point, Rect};
use rand::{
    Rng,
    distributions::{
        Distribution,
        Standard,
    },
};

use texture_manager::TextureId;
use super::*;

/// Defines how a sprite is aligned (or "anchored") relative to its destination rectangle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Anchor {
    /// Sprite is horizontally centered and its top side is aligned with the top side of the
    /// destination rectangle
    N,
    /// The top right corner of the sprite is aligned with the top right corner of the destination
    /// rectangle
    NE,
    /// Sprite is vertically centered and its right side is aligned with the right side of the
    /// destination rectangle
    E,
    /// The bottom right corner of the sprite is aligned with the bottom right corner of the
    /// destination rectangle
    SE,
    /// Sprite is horizontally centered and its bottom side is aligned with the bottom of the
    /// destination rectangle
    S,
    /// The bottom left corner of the sprite is aligned with the bottom left corner of the
    /// destination rectangle
    SW,
    /// Sprite is vertically centered and its left side is aligned with the left side of the
    /// destination rectangle
    W,
    /// The top left corner of the sprite is aligned with the top left corner of the destination
    /// rectangle
    NW,
    /// The center of the sprite is aligned with the center of the destination rectangle
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
    /// An additional amount to offset the destination rectangle
    pub dest_offset: Point,
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
            dest_offset: Point::new(0, 0),
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

    /// Returns this sprite image anchored from its west side
    pub fn anchor_west(self) -> Self {
        Self {
            anchor: Anchor::W,
            ..self
        }
    }

    /// Returns this sprite image anchored from its south side
    pub fn anchor_south(self) -> Self {
        Self {
            anchor: Anchor::S,
            ..self
        }
    }

    /// Returns this sprite image with the given destination offset
    pub fn dest_offset(self, x: i32, y: i32) -> Self {
        Self {
            dest_offset: Point::new(x, y),
            ..self
        }
    }

    /// Given the top left coordinates of where this sprite may be placed, returns the region where
    /// the sprite should really be placed based on its anchor setting
    pub fn apply_anchor(&self, dest: Rect) -> Rect {
        let width = self.region.width() as i32;
        let height = self.region.height() as i32;
        let center = dest.center();

        // Each of these calculations is calculating the anchor point on dest and then offsetting
        // by the width and height of the sprite to get the top left corner of the result rectangle
        let top_left = match self.anchor {
            Anchor::N => Point::new(center.x(), dest.top()).offset(-width/2, 0),
            Anchor::NE => dest.top_right().offset(-width, 0),
            Anchor::E => Point::new(dest.right(), center.y()).offset(-width, -height/2),
            Anchor::SE => dest.bottom_right().offset(-width, -height),
            Anchor::S => Point::new(center.x(), dest.bottom()).offset(-width/2, -height),
            Anchor::SW => dest.bottom_left().offset(0, -height),
            Anchor::W => Point::new(dest.left(), center.y()).offset(0, -height/2),
            Anchor::NW => dest.top_left(),
            Anchor::Center => center.offset(-width/2, -height/2),
        };

        Rect::new(
            top_left.x(),
            top_left.y(),
            width as u32,
            height as u32,
        )
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

impl WallSprite {
    pub fn with_alternate(alt: WallSpriteAlternate) -> Self {
        Self {
            alt,
            ..Default::default()
        }
    }
}

/// Different alternate wall styles for some of the wall sprites
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallSpriteAlternate {
    Alt0,
    Alt1,
    Alt2,
    BrickColumn,
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
    /// Sprites for each orientation of a door
    door_tiles: Vec<SpriteImage>,
}

impl MapSprites {
    /// Creates a table of sprites from the standard layout of the dungeon spritesheet
    pub fn from_dungeon_spritesheet(texture_id: TextureId, tile_size: u32) -> Self {
        // Returns the (tile_size)x(tile_size) sprite for the given row and column of the spritesheet
        macro_rules! tile_sprite {
            (x: $x:expr, y: $y:expr, width: $width:expr, height: $height:expr) => (
                SpriteImage::new_unflipped(
                    texture_id,
                    Rect::new(
                        $x,
                        $y,
                        $width,
                        $height,
                    ),
                )
            );
            (row: $row:expr, col: $col:expr, width: $width:expr, height: $height:expr) => (
                tile_sprite!(
                    x: $col as i32 * tile_size as i32,
                    y: $row as i32 * tile_size as i32,
                    width: $width,
                    height: $height
                )
            );
            (row: $row:expr, col: $col:expr) => (
                tile_sprite!(row: $row, col: $col, width: tile_size, height: tile_size);
            )
        }

        Self {
            empty_tile_sprite: tile_sprite!(row: 0, col: 3),
            floor_tiles: vec![tile_sprite!(row: 0, col: 0)],
            wall_tiles: vec![
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
                tile_sprite!(row: 17, col: 7, width: tile_size, height: tile_size*2).anchor_south(),
            ],
            staircase_up_tiles: vec![
                // bottom step faces right
                tile_sprite!(row: 15, col: 8, width: tile_size, height: tile_size*2).anchor_south().flip_horizontally(),
                // bottom step faces left
                tile_sprite!(row: 15, col: 8, width: tile_size, height: tile_size*2).anchor_south(),
            ],
            staircase_down_tiles: vec![
                // top step faces right
                tile_sprite!(row: 16, col: 7),
                // top step faces left
                tile_sprite!(row: 16, col: 7).flip_horizontally(),
            ],
            door_tiles: vec![
                // horizontal door (closed)
                tile_sprite!(row: 11, col: 14),
                // vertical door (closed)
                tile_sprite!(row: 10, col: 15, width: tile_size, height: tile_size*2).anchor_south(),
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
            (alt: $a:pat) => {
                w!{N: _, E: _, S: _, W: _, alt: $a}
            };
        }

        let s = |n| &self.wall_tiles[n];

        use self::WallSpriteAlternate::*;
        match sprite {
            w!{alt: BrickColumn} => s(20),

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

    pub fn staircase_up_sprite(&self, direction: StairsDirection) -> &SpriteImage {
        use self::StairsDirection::*;
        match direction {
            Right => &self.staircase_up_tiles[0],
            Left => &self.staircase_up_tiles[1],
        }
    }

    pub fn staircase_down_sprite(&self, direction: StairsDirection) -> &SpriteImage {
        use self::StairsDirection::*;
        match direction {
            Right => &self.staircase_down_tiles[0],
            Left => &self.staircase_down_tiles[1],
        }
    }

    pub fn door_sprite(&self, state: Door, orientation: HoriVert) -> Option<&SpriteImage> {
        match (state, orientation) {
            (Door::Locked, HoriVert::Horizontal) |
            (Door::Closed, HoriVert::Horizontal) => Some(&self.door_tiles[0]),
            (Door::Locked, HoriVert::Vertical) |
            (Door::Closed, HoriVert::Vertical) => Some(&self.door_tiles[1]),
            // Just hide open doors (tried rendering a sprite for this but it didn't work out)
            (Door::Open, _) => None,
        }
    }
}
