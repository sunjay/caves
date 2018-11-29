use std::path::Path;

use sdl2::{
    image::{SaveSurface},
    pixels::{PixelFormatEnum},
    surface::Surface,
    render::{Canvas, RenderTarget},
};

use sprites::MapSprites;
use game::Game;

use super::renderer;
use super::{TextureManager, SDLError};

pub struct GameScreen {
    game: Game,
}

impl GameScreen {
    pub fn new(game: Game) -> Self {
        GameScreen {game}
    }

    /// Render the entire state of the level (the entire map) to the given filename.
    ///
    /// Useful for debugging. This function is fairly "slow", so use sparingly.
    pub fn render_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), SDLError> {
        let level_boundary = self.level_boundary();
        let mut canvas = Surface::new(level_boundary.width(), level_boundary.height(),
            PixelFormatEnum::RGBA8888)?.into_canvas()?;
        let texture_creator = canvas.texture_creator();

        let mut textures = TextureManager::new(&texture_creator);
        let map_texture = textures.create_png_texture("assets/dungeon.png")?;
        let tile_size = 16;
        let sprites = MapSprites::from_dungeon_spritesheet(map_texture, tile_size);

        self.render(level_boundary, &mut canvas, level_boundary.top_left(), &sprites, &textures,
            |_, _| true)?;

        canvas.into_surface().save(path)?;
        Ok(())
    }

    pub fn render<T: RenderTarget>(&self, canvas: &mut Canvas<T>) -> Result<(), SDLError> {
        renderer::render(self.game.current_level().system_data())
    }
}
