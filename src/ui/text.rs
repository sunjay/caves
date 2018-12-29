use rusttype::{point, Font, FontCollection, PositionedGlyph, Scale};
use sdl2::{
    rect::Point,
    render::{Canvas, RenderTarget, BlendMode},
    pixels::Color,
};

use super::SDLError;

pub fn load_font() -> Font<'static> {
    let font_data = include_bytes!("../../assets/fonts/Kenney Pixel Square.ttf");
    let collection = FontCollection::from_bytes(font_data as &[u8]).unwrap_or_else(|e| {
        panic!("bug: unable to construct a FontCollection from bytes: {}", e);
    });
    // only succeeds if collection consists of one font
    collection.into_font().unwrap_or_else(|e| {
        panic!("bug: unable to turn FontCollection into a Font: {}", e);
    })
}

/// The way the text layout will be calculated on the screen
#[derive(Debug, Clone)]
pub enum TextLayout {
    /// Centered in the middle of the screen
    Centered,
    /// Top-left corner of text rect will be at the given point
    TopLeftAt(Point),
}

/// Text that can be rendered
#[derive(Debug, Clone)]
pub struct Text<'a> {
    glyphs: Vec<PositionedGlyph<'a>>,
    width: f32,
    line_height: f32,
}

impl<'a> Text<'a> {
    pub fn new<S: AsRef<str>>(font: &'a Font, text: S, height: f32) -> Self {
        let text = text.as_ref();

        // Adapted from: https://github.com/redox-os/rusttype/blob/master/examples/simple.rs

        // Use this to adjust the x:y aspect ratio of the rendered text
        let scale = Scale {x: height, y: height};

        // The origin of a line of text is at the baseline (roughly where
        // non-descending letters sit). We don't want to clip the text, so we shift
        // it down with an offset when laying it out. v_metrics.ascent is the
        // distance between the baseline and the highest edge of any glyph in
        // the font. That's enough to guarantee that there's no clipping.
        let v_metrics = font.v_metrics(scale);
        // If we use the ascent as the offset and add the descent (typically negative) to its
        // value, we can put the font right on the baseline.
        let line_height = v_metrics.ascent - v_metrics.descent;
        let offset = point(0.0, v_metrics.ascent);

        // Glyphs to draw the given text
        let glyphs: Vec<PositionedGlyph<'a>> = font.layout(text, scale, offset).collect();

        let width = glyphs.iter()
            .map(|g| g.position().x as f32 + g.unpositioned().h_metrics().advance_width)
            .fold(0.0, f32::max);

        Self {glyphs, width, line_height}
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn line_height(&self) -> f32 {
        self.line_height
    }

    pub fn render<T: RenderTarget, C: Into<Color>>(
        &self,
        canvas: &mut Canvas<T>,
        color: C,
        layout: TextLayout,
    ) -> Result<(), SDLError> {
        let width = self.width.ceil() as u32;
        let line_height = self.line_height().ceil() as u32;

        use self::TextLayout::*;
        let layout_offset = match layout {
            Centered => {
                let (canvas_width, canvas_height) = canvas.logical_size();
                point(
                    canvas_width / 2 - width / 2,
                    canvas_height / 2 - line_height / 2,
                )
            },
            TopLeftAt(top_left) => {
                assert!(top_left.x() >= 0 && top_left.y() >= 0,
                    "bug: attempt to layout text off the screen");
                point(top_left.x() as u32, top_left.y() as u32)
            },
        };

        canvas.set_blend_mode(BlendMode::Blend);

        let mut color = color.into();
        let alpha_start = color.a as f32;
        for glyph in &self.glyphs {
            if let Some(bb) = glyph.pixel_bounding_box() {
                let mut result = Ok(());
                glyph.draw(|x, y, v| {
                    if result.is_err() {
                        return;
                    }

                    let x = x as i32 + bb.min.x + layout_offset.x as i32;
                    let y = y as i32 + bb.min.y + layout_offset.y as i32;

                    color.a = (alpha_start * v) as u8;
                    canvas.set_draw_color(color);
                    result = canvas.draw_point((x, y));
                });
                result.map_err(SDLError)?;
            }
        }

        Ok(())
    }
}
