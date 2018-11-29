use std::cmp;
use std::iter::once;

use sdl2::{
    self,
    Sdl,
    TimerSubsystem,
    EventPump,
    image::{Sdl2ImageContext, INIT_PNG},
    pixels::Color,
    rect::{Point, Rect},
    render::{TextureCreator, Canvas, RenderTarget},
    video::{Window, WindowContext},
};
use specs::{
    Join,
    ReadStorage,
    World,
    Resources,
    SystemData,
    ReadExpect,
};

use components::{Position, Sprite, CameraFocus};
use map::{FloorMap, Tile};
use sprites::MapSprites;
use super::TextureManager;
use super::SDLError;

#[derive(SystemData)]
struct RenderData<'a> {
    map: ReadExpect<'a, FloorMap>,
    camera_focuses: ReadStorage<'a, CameraFocus>,
    positions: ReadStorage<'a, Position>,
    sprites: ReadStorage<'a, Sprite>,
}

pub fn setup(res: &mut Resources) {
    RenderData::setup(res);
}

pub fn render<T: RenderTarget, U>(
    data: RenderData,
    canvas: &mut Canvas<T>,
    textures: &TextureManager<U>,
    map_sprites: &MapSprites,
) -> Result<(), SDLError> {
    canvas.clear();

    let RenderData {map, positions, sprites, camera_focuses} = data;
    let mut camera_focuses = (&positions, &camera_focuses).join();
    let (&Position(camera_focus), _) = camera_focuses.next()
        .expect("Renderer was not told which entity to focus on");
    assert!(camera_focuses.next().is_none(),
        "Renderer was asked to focus on more than one thing");

    let (screen_width, screen_height) = canvas.logical_size();
    let screen_center = Point::new(screen_width as i32 / 2, screen_height as i32 / 2);

    // The position on the map of the screen's top left corner
    // Adding this point to the position of the camera_focus would make it render in the center
    // of the screen
    let render_top_left = camera_focus - screen_center;

    // Need to make sure the camera stays within the level boundary
    let level_boundary = map.level_boundary();

    // The valid ranges for the top-left corner of the screen
    let (min_x, max_x) = (0, level_boundary.x() + level_boundary.width() as i32 - screen_width as i32);
    let (min_y, max_y) = (0, level_boundary.y() + level_boundary.height() as i32 - screen_height as i32);
    let clamp = |min, x, max| cmp::min(cmp::max(min, x), max);
    let render_top_left = Point::new(
        clamp(min_x, render_top_left.x, max_x),
        clamp(min_y, render_top_left.y, max_y),
    );

    // Get the tiles surrounding the camera focus
    let screen = Rect::from_center(render_top_left + screen_center, screen_width, screen_height);

    // Only render tiles that are visible to the camera focus.
    let focus_pos = map.world_to_tile_pos(camera_focus);

    // The returned set will contain all tiles that are directly visible to the camera focus
    // without passing through entrances that have still not been opened.
    let visible_tiles = map.grid().depth_first_search(focus_pos, |node, _| {
        let grid = map.grid();

        // Stop searching at walls or closed entrances (but still include them in the result)
        let tile = grid.get(node);
        !tile.is_wall() && !(tile.has_entrance() && !tile.is_traversable())
    });

    let is_visible = |pt, tile: &Tile| {
        visible_tiles.contains(&pt) ||
        // Need to specially handle wall corners because they are not *directly* visible.
        // A corner is a wall tile with at least two visible walls
        tile.is_wall() && map.grid().adjacent_positions(pt)
            .filter(|pt| visible_tiles.contains(pt)).count() >= 2
    };

    map.render(screen, canvas, render_top_left, map_sprites, textures,
        is_visible)?;

    for (&Position(pos), Sprite(ref sprite)) in (&positions, &sprites).join() {
        let pos = pos - render_top_left;
        let texture = textures.get(sprite.texture_id);
        let source_rect = sprite.region;
        let mut dest_rect = source_rect.clone();
        dest_rect.center_on(pos);

        canvas.copy_ex(
            texture,
            source_rect,
            dest_rect,
            0.0,
            None,
            sprite.flip_horizontal,
            sprite.flip_vertical,
        ).map_err(SDLError)?;
    }

    canvas.present();

    Ok(())
}

/// Renders the tiles of the map within the given region
pub fn render_map<T: RenderTarget, U>(
    map: &FloorMap,
    region: Rect,
    canvas: &mut Canvas<T>,
    render_top_left: Point,
    sprites: &MapSprites,
    textures: &TextureManager<U>,
    mut should_render: impl FnMut(TilePos, &Tile) -> bool,
) -> Result<(), SDLError> {
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
    pos: Point,
    sprite: &SpriteImage,
    canvas: &mut Canvas<T>,
    render_top_left: Point,
    textures: &TextureManager<U>,
) -> Result<(), SDLError> {
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
        source_rect,
        dest_rect,
        0.0,
        None,
        sprite.flip_horizontal,
        sprite.flip_vertical,
    ).map_err(SDLError)
}
