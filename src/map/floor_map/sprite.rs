use sdl2::rect::Rect;

use texture_manager::TextureId;

/// Represents an image/texture that will be renderered
///
/// The convention is that the sprite begins pointing to the right and flipping it horizontally
/// results in it facing left
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
}

impl SpriteImage {
    /// Creates a new SpriteImage that is not flipped either horizontally or vertically
    pub fn new_unflipped(texture_id: TextureId, region: Rect) -> Self {
        SpriteImage {
            texture_id,
            region,
            flip_horizontal: false,
            flip_vertical: false,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallSprite {
    Type1,
}

impl Default for WallSprite {
    fn default() -> Self {
        // Need a default wall tile sprite because we can't determine the actual wall tile sprite to
        // use until after all of the tiles are placed.
        WallSprite::Type1
    }
}

/// A lookup table for all map sprites
/// Used to avoid having to manage sprites in each tile
#[derive(Debug, Clone)]
pub struct MapSprites {
    /// Sprites for each type of floor tile. Each of these must map to a FloorSprite variant
    floor_tiles: Vec<SpriteImage>,
    /// Sprites for each type of wall tile. Each of these must map to a WallSprite variant
    wall_tiles: Vec<SpriteImage>,
    /// Sprite for Tile::Empty
    empty_tile_sprite: SpriteImage,
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
            floor_tiles: vec![tile_sprite(0, 0)],
            wall_tiles: vec![tile_sprite(8, 0)],
            empty_tile_sprite: tile_sprite(0, 3),
        }
    }

    pub fn floor_sprite(&self, sprite: FloorSprite) -> &SpriteImage {
        use self::FloorSprite::*;
        match sprite {
            Type1 => &self.floor_tiles[0],
        }
    }

    pub fn wall_sprite(&self, sprite: WallSprite) -> &SpriteImage {
        use self::WallSprite::*;
        match sprite {
            Type1 => &self.wall_tiles[0],
        }
    }

    pub fn empty_tile_sprite(&self) -> &SpriteImage {
        &self.empty_tile_sprite
    }
}
