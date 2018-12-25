mod texture_manager;
mod sprite_manager;
mod sprite;

pub use self::texture_manager::*;
pub use self::sprite_manager::*;
pub use self::sprite::*;

use sdl2::render::TextureCreator;

use crate::components::AnimationManager;
use crate::map_sprites::MapSprites;
use crate::ui::SDLError;

pub struct AssetManager<'a, T: 'a> {
    pub textures: TextureManager<'a, T>,
    pub map_sprites: MapSprites,
    pub player_animations: AnimationManager,
    pub sprites: SpriteManager,
}

impl<'a, T> AssetManager<'a, T> {
    pub fn load(texture_creator: &'a TextureCreator<T>, fps: usize, tile_size: u32) -> Result<Self, SDLError> {
        let mut textures = TextureManager::new(&texture_creator);
        let mut sprites = SpriteManager::default();

        let map_texture = textures.create_png_texture("assets/dungeon.png")?;
        let map_sprites = MapSprites::from_dungeon_spritesheet(map_texture, &mut sprites, tile_size);

        let player_texture = textures.create_png_texture("assets/hero.png")?;
        let player_animations = AnimationManager::standard_character_animations(fps, player_texture, &mut sprites);

        Ok(Self {
            textures,
            map_sprites,
            player_animations,
            sprites,
        })
    }
}
