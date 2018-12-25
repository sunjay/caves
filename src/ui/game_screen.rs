use std::path::Path;

use sdl2::render::{Canvas, RenderTarget};
use component_group::ComponentGroup;

use crate::map_sprites::MapSprites;
use crate::generator::GenLevel;
use crate::components::PlayerComponents;
use crate::resources::{FramesElapsed, Event, GameState};
use crate::assets::{TextureManager, SpriteManager};

use super::{SDLError, LevelScreen};

pub struct GameScreen<'a, 'b> {
    levels: Vec<LevelScreen<'a, 'b>>,
    current_level: usize,
}

impl<'a, 'b> GameScreen<'a, 'b> {
    pub fn new(player: PlayerComponents, mut levels: Vec<GenLevel<'a, 'b>>) -> Self {
        // Add player
        {
            let first_world = &mut levels.first_mut()
                .expect("bug: should be at least one level")
                .world;
            player.create(first_world);
        }

        Self {
            levels: levels.into_iter().map(Into::into).collect(),
            current_level: 0,
        }
    }

    /// Returns the current level screen
    pub fn current_level(&self) -> &LevelScreen<'a, 'b> {
        &self.levels[self.current_level]
    }

    /// Returns an iterator of the level screens
    pub fn levels(&self) -> impl Iterator<Item=&LevelScreen<'a, 'b>> {
        self.levels.iter()
    }

    /// Dispatch the given events and update the state based on the frames that have elapsed
    pub fn dispatch(&mut self, frames_elapsed: FramesElapsed, events: Vec<Event>) {
        let newstate = self.levels[self.current_level].dispatch(frames_elapsed, events);
        if let Some(newstate) = newstate {
            use self::GameState::*;
            match newstate {
                GoToNextLevel {id} => self.to_next_level(id),
                GoToPrevLevel {id} => self.to_prev_level(id),
                Pause => unimplemented!(),
            }
        }
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

    /// Advances to the next level. Panics if there is no next level
    fn to_next_level(&mut self, gate_id: usize) {
        // Fetch the player as-is from the current world
        let mut player = self.current_level().player_components();

        // Go to the next level
        self.current_level += 1;
        assert!(self.current_level < self.levels.len(), "bug: advanced too many levels");

        // When going to the next level, we need to connect back to the corresponding gate that
        // will take you back to the previous level
        player.position.0 = self.current_level().find_to_prev_level_adjacent(gate_id);
        // Move the player from the previous level to the next level
        self.levels[self.current_level].update_player(player);
    }

    /// Goes back to the previous level. Panics if there is no previous level.
    fn to_prev_level(&mut self, gate_id: usize) {
        // Fetch the player as-is from the current world
        let mut player = self.current_level().player_components();

        // Go the previous level
        self.current_level = self.current_level.checked_sub(1)
            .expect("bug: went back too many levels");

        // When going to the previous level, we need to connect back to the corresponding gate that
        // will take you to the next level
        player.position.0 = self.current_level().find_to_next_level_adjacent(gate_id);
        // Move the player from the next level to the previous level
        self.levels[self.current_level].update_player(player);
    }
}
