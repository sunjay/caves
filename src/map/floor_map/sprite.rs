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

/// A table that keeps track of a group of sprites
/// Used to avoid having to separately keep a static sprite in each tile
#[derive(Debug, Clone)]
pub struct SpriteTable {
    pub floor_tiles: Vec<SpriteImage>,
    pub wall_tiles: Vec<SpriteImage>,
    pub empty_tile_sprite: SpriteImage,
    /// Need a default floor tile sprite because we can't determine the actual floor tile sprite to
    /// use until after all of the tiles are placed.
    pub default_floor_tile_index: usize,
    /// Need a default wall tile sprite because we can't determine the actual wall tile sprite to
    /// use until after all of the tiles are placed.
    pub default_wall_tile_index: usize,
}
