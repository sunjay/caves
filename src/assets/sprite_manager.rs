use super::SpriteImage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpriteId(usize);

#[derive(Default)]
pub struct SpriteManager {
    sprites: Vec<SpriteImage>,
}

impl SpriteManager {
    pub fn get(&self, SpriteId(index): SpriteId) -> &SpriteImage {
        &self.sprites[index]
    }

    pub fn add(&mut self, image: SpriteImage) -> SpriteId {
        self.sprites.push(image);
        SpriteId(self.sprites.len() - 1)
    }
}
