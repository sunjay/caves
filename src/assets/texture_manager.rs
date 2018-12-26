use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use rusttype::{point, Font, FontCollection, PositionedGlyph, Scale};
use sdl2::{
    image::LoadTexture,
    render::{TextureCreator, Texture},
    surface::Surface,
    pixels::{Color, PixelFormatEnum},
};

use crate::ui::SDLError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureId(usize);

// NOTE: Ideally, this would just be managed in the Window, but we can't do that because
// we can't have a field in a struct that refers to another field. Textures are dependent
// on the TextureCreator and they need to be stored separately in order for this to work.
pub struct TextureManager<'a, T> {
    font: Font<'static>,
    texture_creator: &'a TextureCreator<T>,
    textures: Vec<Texture<'a>>,
    /// Memoized textures for each path so we don't end up loading a path twice for no reason.
    /// Path is canonicalized so that slight differences in the path get normalized.
    path_textures: HashMap<PathBuf, TextureId>,
}

impl<'a, T> TextureManager<'a, T> {
    pub fn new(texture_creator: &'a TextureCreator<T>) -> Self {
        let font_data = include_bytes!("../../assets/fonts/Kenney Pixel Square.ttf");
        let collection = FontCollection::from_bytes(font_data as &[u8]).unwrap_or_else(|e| {
            panic!("bug: unable to construct a FontCollection from bytes: {}", e);
        });
        // only succeeds if collection consists of one font
        let font = collection.into_font().unwrap_or_else(|e| {
            panic!("bug: unable to turn FontCollection into a Font: {}", e);
        });

        Self {
            font,
            texture_creator,
            textures: Default::default(),
            path_textures: Default::default(),
        }
    }

    /// Retrieves the texture for the given ID
    pub fn get(&self, TextureId(index): TextureId) -> &Texture<'a> {
        &self.textures[index]
    }

    /// Creates a texture from the given path
    pub fn create_png_texture<P: AsRef<Path>>(&mut self, path: P) -> Result<TextureId, SDLError> {
        let path = path.as_ref();
        if self.path_textures.contains_key(path) {
            return Ok(self.path_textures[path])
        }

        let texture = self.texture_creator.load_texture(path).map_err(SDLError)?;
        let id = self.add_texture(texture);
        let path = path.canonicalize()
            .expect("Failed to canonicalize path for loaded texture");
        self.path_textures.insert(path, id);

        Ok(id)
    }

    /// Creates a texture with the given text painted on it
    ///
    /// `height` is the desired font pixel height.
    pub fn create_text_texture(
        &mut self,
        text: impl AsRef<str>,
        height: f32,
        color: impl Into<Color>,
    ) -> Result<TextureId, SDLError> {
        let text = text.as_ref();

        // Adapted from: https://github.com/redox-os/rusttype/blob/master/examples/simple.rs

        // Use this to adjust the x:y aspect ratio of the rendered text
        let scale = Scale {x: height, y: height};

        // The origin of a line of text is at the baseline (roughly where
        // non-descending letters sit). We don't want to clip the text, so we shift
        // it down with an offset when laying it out. v_metrics.ascent is the
        // distance between the baseline and the highest edge of any glyph in
        // the font. That's enough to guarantee that there's no clipping.
        let v_metrics = self.font.v_metrics(scale);
        // If we use the ascent as the offset and add the descent (typically negative) to its
        // value, we can put the font right on the baseline.
        let line_height = v_metrics.ascent - v_metrics.descent;
        let offset = point(0.0, v_metrics.ascent);

        // Glyphs to draw the given text
        let glyphs: Vec<PositionedGlyph<'_>> = self.font.layout(text, scale, offset).collect();

        let width = glyphs.iter()
            .map(|g| g.position().x as f32 + g.unpositioned().h_metrics().advance_width)
            .fold(0.0, f32::max)
            .ceil() as u32;
        let mut canvas = Surface::new(width, line_height as u32, PixelFormatEnum::RGBA32)
            .and_then(Surface::into_canvas).map_err(SDLError)?;
        let mut color = color.into();
        let alpha_start = color.a as f32;

        for glyph in glyphs {
            if let Some(bb) = glyph.pixel_bounding_box() {
                let mut result = Ok(());
                glyph.draw(|x, y, v| {
                    if result.is_err() {
                        return;
                    }

                    let x = x as i32 + bb.min.x;
                    let y = y as i32 + bb.min.y;

                    color.a = (alpha_start * v) as u8;
                    canvas.set_draw_color(color);
                    result = canvas.draw_point((x, y));
                });
                result.map_err(SDLError)?;
            }
        }

        let texture = self.texture_creator.create_texture_from_surface(canvas.into_surface())
            .map_err(|e| SDLError(e.to_string()))?;
        Ok(self.add_texture(texture))
    }

    fn add_texture(&mut self, texture: Texture<'a>) -> TextureId {
        self.textures.push(texture);
        TextureId(self.textures.len() - 1)
    }
}
