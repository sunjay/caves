use sdl2::rect::{Point, Rect};

use crate::assets::TextureId;

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
