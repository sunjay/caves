use std::path::Path;
use std::iter::once;

use sdl2::{
    image::{SaveSurface},
    pixels::{PixelFormatEnum},
    surface::Surface,
    rect::{Point, Rect},
    render::{Canvas, RenderTarget},
};

use super::{FloorMap, MapSprites};
use texture_manager::TextureManager;

impl FloorMap {
    pub fn render_to_file<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<(), String> {
        let level_boundary = self.level_boundary();
        let mut canvas = Surface::new(level_boundary.width(), level_boundary.height(),
            PixelFormatEnum::RGBA8888)?.into_canvas()?;
        let texture_creator = canvas.texture_creator();

        let mut textures = TextureManager::new(&texture_creator);
        let map_texture = textures.create_png_texture("assets/dungeon.png")?;
        let tile_size = 16;
        let sprites = MapSprites::from_dungeon_spritesheet(map_texture, tile_size);

        self.render(level_boundary, &mut canvas, level_boundary.top_left(), &sprites, &textures)?;

        canvas.into_surface().save(path)?;
        Ok(())
    }

    /// Renders the tiles of the map within the given region
    pub fn render<T: RenderTarget, U>(
        &self,
        region: Rect,
        canvas: &mut Canvas<T>,
        render_top_left: Point,
        sprites: &MapSprites,
        textures: &TextureManager<U>,
    ) -> Result<(), String> {
        let tiles = self.tiles_within(region);
        for (pos, _, tile) in tiles {
            let tile_layers = once(tile.background_sprite(sprites))
                .chain(tile.object_sprite(sprites));
            for sprite in tile_layers {
                let texture = textures.get(sprite.texture_id);
                let source_rect = sprite.region.clone();
                let dest_rect = Rect::new(
                    // Need to subtract the position (world coordinates) of this tile from the position
                    // in world coordinates of the top-left corner of the screen so that we are left
                    // with the position of this sprite on the screen in screen coordinates
                    pos.x() - render_top_left.x(),
                    pos.y() - render_top_left.y(),
                    sprite.region.width(),
                    sprite.region.height()
                );

                canvas.copy_ex(
                    texture,
                    Some(source_rect),
                    Some(dest_rect),
                    0.0,
                    None,
                    sprite.flip_horizontal,
                    sprite.flip_vertical,
                )?;
            }
        }

        Ok(())
    }
}
