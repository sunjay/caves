use std::path::Path;
use std::iter::once;

use sdl2::{
    image::{SaveSurface},
    pixels::{PixelFormatEnum},
    surface::Surface,
    rect::{Point, Rect},
    render::{Canvas, RenderTarget},
};

use super::{FloorMap, MapSprites, TilePos, Tile, SpriteImage};
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

        self.render(level_boundary, &mut canvas, level_boundary.top_left(), &sprites, &textures,
            |_, _| true)?;

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
        mut should_render: impl FnMut(TilePos, &Tile) -> bool,
    ) -> Result<(), String> {
        // Need to paint the default floor under every tile in case the background sprite being
        // used is actually something that doesn't take up the entire space (e.g. a column tile)
        let default_floor = sprites.floor_sprite(Default::default());

        // Rendering strategy: For each row, first render all the backgrounds, then render all of
        // objects at once right after. This allows an object to overlap the background of the tile
        // on its right.

        let (top_left, size) = self.grid_area_within(region);
        for (row, row_tiles) in self.grid().rows().enumerate().skip(top_left.row).take(size.rows) {
            for (col, tile) in row_tiles.iter().enumerate().skip(top_left.col).take(size.cols) {
                let pos = TilePos {row, col};

                if !should_render(pos, tile) {
                    // Render an empty tile
                    let pos = pos.to_point(self.tile_size as i32);
                    let sprite = sprites.empty_tile_sprite();
                    self.render_sprite(pos, sprite, canvas, render_top_left, textures)?;
                    continue;
                }

                let pos = pos.to_point(self.tile_size as i32);
                let tile_layers = once(default_floor)
                    .chain(once(tile.background_sprite(sprites)));

                for sprite in tile_layers {
                    self.render_sprite(pos, sprite, canvas, render_top_left, textures)?;
                }
            }

            for (col, tile) in row_tiles.iter().enumerate().skip(top_left.col).take(size.cols) {
                let pos = TilePos {row, col};

                if !should_render(pos, tile) {
                    // Do not render the tile's object
                    continue;
                }

                // Do not want to render the wall decoration if we are not going to render the
                // tile south of this wall. Reason: Objects within a room should only be visible
                // when that room is visible
                if tile.is_wall() {
                    let should_render_south = pos.adjacent_south(self.grid().rows_len())
                        .map(|south| should_render(south, self.grid().get(south)))
                        .unwrap_or(false);
                    if !should_render_south {
                        continue;
                    }
                }

                let pos = pos.to_point(self.tile_size as i32);
                if let Some(sprite) = tile.foreground_sprite(sprites) {
                    self.render_sprite(pos, sprite, canvas, render_top_left, textures)?;
                }
            }
        }

        Ok(())
    }

    fn render_sprite<T: RenderTarget, U>(
        &self,
        pos: Point,
        sprite: &SpriteImage,
        canvas: &mut Canvas<T>,
        render_top_left: Point,
        textures: &TextureManager<U>,
    ) -> Result<(), String> {
        let texture = textures.get(sprite.texture_id);
        // Source rect should never be modified here because it represents the exact place
        // on the spritesheet of this sprite. No reaosn to modify that.
        let source_rect = sprite.region.clone();

        // The destination rectangle that this sprite should be aligned against. The sprite
        // is not required to be confined to this rectangle. It is only used to decide how
        // the sprite's layout should be calculated.
        let dest = Rect::new(
            // Need to subtract the position (world coordinates) of this tile from the position
            // in world coordinates of the top-left corner of the screen so that we are left
            // with the position of this sprite on the screen in screen coordinates
            pos.x() - render_top_left.x(),
            pos.y() - render_top_left.y(),
            self.tile_size,
            self.tile_size,
        );
        let mut dest_rect = sprite.apply_anchor(dest);

        let dest_offset = sprite.dest_offset;
        dest_rect.offset(dest_offset.x(), dest_offset.y());

        canvas.copy_ex(
            texture,
            Some(source_rect),
            Some(dest_rect),
            0.0,
            None,
            sprite.flip_horizontal,
            sprite.flip_vertical,
        )
    }
}
