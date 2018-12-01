use std::path::Path;

use sdl2::{rect::Point, render::{Canvas, RenderTarget}};

use map_sprites::MapSprites;
use generator::GenLevel;
use resources::{FramesElapsed, Event};
use assets::{TextureManager, SpriteManager};

use super::{SDLError, LevelScreen};

pub struct GameScreen<'a, 'b> {
    player_start: Point,
    levels: Vec<LevelScreen<'a, 'b>>,
    current_level: usize,
}

impl<'a, 'b> GameScreen<'a, 'b> {
    pub fn new(player_start: Point, levels: Vec<GenLevel<'a, 'b>>) -> Self {
        Self {
            player_start,
            levels: levels.into_iter().map(Into::into).collect(),
            current_level: 0,
        }
    }

    /// Returns the current level screen
    pub fn current_level(&self) -> &LevelScreen<'a, 'b> {
        &self.levels[self.current_level]
    }

    /// Advances to the next level. Panics if there is no next level
    pub fn to_next_level(&mut self) {
        self.current_level += 1;
        assert!(self.current_level < self.levels.len(), "bug: advanced too many levels");
    }

    /// Goes back to the previous level. Panics if there is no previous level.
    pub fn to_prev_level(&mut self) {
        self.current_level = self.current_level.checked_sub(1)
            .expect("bug: went back too many levels");
    }

    /// Returns an iterator of the level screens
    pub fn levels(&self) -> impl Iterator<Item=&LevelScreen<'a, 'b>> {
        self.levels.iter()
    }

    /// Dispatch the given events and update the state based on the frames that have elapsed
    pub fn dispatch(&mut self, frames_elapsed: FramesElapsed, events: Vec<Event>) {
        self.current_level().dispatch(frames_elapsed, events);
    }

    /// Render the entire state of the current level (the entire map) to the given filename.
    ///
    /// Useful for debugging. This function is fairly "slow", so use sparingly.
    pub fn render_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), SDLError> {
        self.current_level().render_to_file(path)
    }

    pub fn render<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        textures: &TextureManager<<T as RenderTarget>::Context>,
        sprites: &SpriteManager,
        map_sprites: &MapSprites,
    ) -> Result<(), SDLError> {
        self.current_level().render(canvas, textures, sprites, map_sprites)
    }
}
