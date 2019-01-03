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

pub struct EnemyAnimations {
    pub rat: AnimationManager,
}

pub struct AssetManager<'a, T> {
    pub textures: TextureManager<'a, T>,
    pub map_sprites: MapSprites,
    pub player_animations: AnimationManager,
    pub enemy_animations: EnemyAnimations,
    pub sprites: SpriteManager,
}

impl<'a, T> AssetManager<'a, T> {
    pub fn load(texture_creator: &'a TextureCreator<T>, fps: usize, tile_size: u32) -> Result<Self, SDLError> {
        let mut textures = TextureManager::new(&texture_creator);
        let mut sprites = SpriteManager::default();

        let map_texture = textures.create_png_texture("assets/dungeon.png")?;
        let map_sprites = MapSprites::from_dungeon_spritesheet(map_texture, &mut sprites, tile_size);

        let mut character_animations = |path| {
            let texture = textures.create_png_texture(path)?;
            Ok(AnimationManager::standard_character_animations(fps, texture, &mut sprites))
        };

        let player_animations = character_animations("assets/hero.png")?;
        let rat = character_animations("assets/enemies/rat.png")?;

        Ok(Self {
            textures,
            map_sprites,
            player_animations,
            enemy_animations: EnemyAnimations {
                rat,
            },
            sprites,
        })
    }
}
