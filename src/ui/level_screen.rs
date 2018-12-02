use std::path::Path;

use sdl2::{
    image::{SaveSurface},
    pixels::{PixelFormatEnum},
    surface::Surface,
    render::{Canvas, RenderTarget},
};
use specs::{Dispatcher, World};

use assets::{AssetManager, TextureManager, SpriteManager};
use map_sprites::MapSprites;
use generator::GenLevel;
use map::FloorMap;
use resources::{FramesElapsed, Event, ChangeGameState, ActionQueue, EventQueue};

use super::renderer::{RenderData, render_area, render_player_visible};
use super::SDLError;

pub struct LevelScreen<'a, 'b> {
    dispatcher: Dispatcher<'a, 'b>,
    world: World,
}

impl<'a, 'b> From<GenLevel<'a, 'b>> for LevelScreen<'a, 'b> {
    fn from(GenLevel {dispatcher, world}: GenLevel<'a, 'b>) -> Self {
        Self {dispatcher, world}
    }
}

impl<'a, 'b> LevelScreen<'a, 'b> {
    /// Dispatch the given events and update the state based on the frames that have elapsed
    pub fn dispatch(&mut self, frames_elapsed: FramesElapsed, events: Vec<Event>) {
        //NOTE: All resources here must already be added when the world is created
        *self.world.write_resource::<FramesElapsed>() = frames_elapsed;
        *self.world.write_resource::<ChangeGameState>() = ChangeGameState::default();
        *self.world.write_resource::<ActionQueue>() = ActionQueue::default();
        *self.world.write_resource::<EventQueue>() = EventQueue(events);

        self.dispatcher.dispatch(&mut self.world.res);

        // Register any updates
        self.world.maintain();
    }

    /// Render the entire state of the level (the entire map) to the given filename.
    ///
    /// Useful for debugging. This function is fairly "slow", so use sparingly.
    pub fn render_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), SDLError> {
        //TODO: This code is super fragile and bad. The SpriteIds generated do not necessarily
        // correspond to valid IDs in the code below. We try to mimic the order in which the
        // SpriteManager is populated, but this makes this code prone to breaking. We should
        // probably just get rid of this? (Or otherwise unify the loading of all of our resources
        // in one place with a struct that is generic to the texture context type)

        let map = self.world.read_resource::<FloorMap>();

        let level_boundary = map.level_boundary();
        let mut canvas = Surface::new(level_boundary.width(), level_boundary.height(),
            PixelFormatEnum::RGBA8888).and_then(|c| c.into_canvas()).map_err(SDLError)?;
        let texture_creator = canvas.texture_creator();

        let tile_size = 16;
        let AssetManager {
            textures,
            map_sprites,
            sprites,
            ..
        } = AssetManager::load(&texture_creator, 30, tile_size)?;

        let data: RenderData = self.world.system_data();
        render_area(data, level_boundary, &mut canvas, &map_sprites, &textures,
            &sprites, |_, _| true)?;

        canvas.into_surface().save(path).map_err(SDLError)?;
        Ok(())
    }

    pub fn render<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        textures: &TextureManager<<T as RenderTarget>::Context>,
        sprites: &SpriteManager,
        map_sprites: &MapSprites,
    ) -> Result<(), SDLError> {
        render_player_visible(self.world.system_data(), canvas, textures, sprites, map_sprites)
    }
}
