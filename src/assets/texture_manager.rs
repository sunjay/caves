use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use sdl2::{
    image::LoadTexture,
    render::{TextureCreator, Texture},
};

use crate::ui::SDLError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureId(usize);

// NOTE: Ideally, this would just be managed in the Window, but we can't do that because
// we can't have a field in a struct that refers to another field. Textures are dependent
// on the TextureCreator and they need to be stored separately in order for this to work.
pub struct TextureManager<'a, T> {
    texture_creator: &'a TextureCreator<T>,
    textures: Vec<Texture<'a>>,
    /// Memoized textures for each path so we don't end up loading a path twice for no reason.
    /// Path is canonicalized so that slight differences in the path get normalized.
    path_textures: HashMap<PathBuf, TextureId>,
}

impl<'a, T> TextureManager<'a, T> {
    pub fn new(texture_creator: &'a TextureCreator<T>) -> Self {
        Self {
            texture_creator,
            textures: Default::default(),
            path_textures: Default::default(),
        }
    }

    pub fn get(&self, TextureId(index): TextureId) -> &Texture<'a> {
        &self.textures[index]
    }

    pub fn create_png_texture<P: AsRef<Path>>(&mut self, path: P) -> Result<TextureId, SDLError> {
        let path = path.as_ref();
        if self.path_textures.contains_key(path) {
            return Ok(self.path_textures[path])
        }

        let texture = self.texture_creator.load_texture(path).map_err(SDLError)?;
        self.textures.push(texture);

        let id = TextureId(self.textures.len() - 1);
        let path = path.canonicalize()
            .expect("Failed to canonicalize path for loaded texture");
        self.path_textures.insert(path, id);

        Ok(id)
    }
}
