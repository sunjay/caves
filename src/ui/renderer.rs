use std::cmp;
use std::iter::once;
use std::collections::HashSet;

use sdl2::{
    rect::{Point, Rect},
    render::{Canvas, RenderTarget},
};
use rusttype::Font;
use specs::{Join, ReadStorage, Resources, SystemData, ReadExpect};

use crate::assets::{TextureManager, SpriteManager, SpriteImage};
use crate::components::{Position, Sprite, CameraFocus, Door, Ghost};
use crate::map::{FloorMap, TileGrid, Tile, TilePos};
use crate::map_sprites::MapSprites;
use super::{SDLError, Text, TextLayout};

pub struct RenderContext<'a, T: RenderTarget> {
    pub font: Font<'static>,
    pub canvas: &'a mut Canvas<T>,
    pub textures: &'a TextureManager<'a, <T as RenderTarget>::Context>,
    pub sprites: &'a SpriteManager,
    pub map_sprites: &'a MapSprites,
}

impl<'a, T: RenderTarget> RenderContext<'a, T> {
    pub fn new(
        canvas: &'a mut Canvas<T>,
        textures: &'a TextureManager<'a, <T as RenderTarget>::Context>,
        sprites: &'a SpriteManager,
        map_sprites: &'a MapSprites,
    ) -> Self {
        Self {font: super::text::load_font(), canvas, textures, sprites, map_sprites}
    }
}

#[derive(SystemData)]
pub(in super) struct RenderData<'a> {
    map: ReadExpect<'a, FloorMap>,
    camera_focuses: ReadStorage<'a, CameraFocus>,
    positions: ReadStorage<'a, Position>,
    doors: ReadStorage<'a, Door>,
    sprites: ReadStorage<'a, Sprite>,
    ghosts: ReadStorage<'a, Ghost>,
}

impl<'a> AsRef<RenderData<'a>> for RenderData<'a> {
    fn as_ref(&self) -> &Self {
        &self
    }
}

pub fn setup(res: &mut Resources) {
    RenderData::setup(res);
}

pub struct DebugInfo {
    pub fps: u32,
}

/// Renders a debug view
pub fn render_debug_view<T: RenderTarget>(
    ctx: &mut RenderContext<T>,
    debug_info: DebugInfo,
) -> Result<(), SDLError> {
    let text = Text::new(&ctx.font, format!("{}FPS", debug_info.fps), 10.0);
    let padding = 3;
    let (canvas_width, canvas_height) = ctx.canvas.logical_size();

    let box_width = text.width().ceil() as u32 + padding * 2;
    let box_height = text.line_height().ceil() as u32 + padding * 2;
    let box_x = (canvas_width - box_width) as i32;
    let box_y = (canvas_height - box_height) as i32;
    ctx.canvas.set_draw_color((60, 60, 60));
    ctx.canvas.fill_rect(Rect::new(box_x, box_y, box_width, box_height)).map_err(SDLError)?;

    text.render(ctx.canvas, (128, 128, 128), TextLayout::TopLeftAt(Point::new(
        box_x + padding as i32,
        box_y + padding as i32,
    )))?;

    Ok(())
}

/// Renders the area of the world that is visible to the player
pub(in super) fn render_player_visible<T: RenderTarget>(
    data: RenderData<'_>,
    ctx: &mut RenderContext<T>,
) -> Result<(), SDLError> {
    let RenderData {map, positions, camera_focuses, doors, ..} = &data;
    let tile_size = map.tile_size() as i32;
    let grid = map.grid();

    let mut camera_focuses = (positions, camera_focuses).join();
    let (&Position(camera_focus), _) = camera_focuses.next()
        .expect("Renderer was not told which entity to focus on");
    assert!(camera_focuses.next().is_none(),
        "Renderer was asked to focus on more than one thing");

    let (screen_width, screen_height) = ctx.canvas.logical_size();
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
    let screen = Rect::new(
        render_top_left.x(),
        render_top_left.y(),
        screen_width,
        screen_height,
    );

    // Only render tiles that are visible to the camera focus.

    // The tile that the camera focus is currently standing on
    let focus_pos = map.world_to_tile_pos(camera_focus);

    // The returned set will contain all tiles that are directly visible to the camera focus
    // without passing through entrances that have still not been opened.
    let visible_tiles = find_visible_tiles(grid, focus_pos, tile_size, positions, doors);

    let should_render = |pt, tile: &Tile| {
        visible_tiles.contains(&pt) ||
        // Need to specially handle wall corners because they are not *directly* visible.
        // A corner is a wall tile with at least two visible walls
        tile.is_wall() && grid.adjacent_positions(pt)
            .filter(|pt| visible_tiles.contains(pt)).count() >= 2
    };

    render_area(&data, screen, ctx, should_render)
}

fn find_visible_tiles(
    grid: &TileGrid,
    pos: TilePos,
    tile_size: i32,
    positions: &ReadStorage<'_, Position>,
    doors: &ReadStorage<'_, Door>,
) -> HashSet<TilePos> {
    let find_door = |target: TilePos| {
        let target_center = target.center(tile_size);
        (positions, doors).join()
            .find(|(&Position(pos), Door {..})| pos == target_center)
    };

    // If the position center is at a door, start one tile back away from it
    let pos = match find_door(pos) {
        //TODO: This code is fragile. It only works because we have two bounding boxes: full and
        // bottom half. If we were to one day add another type, it would no longer work.
        // Reason: This code is meant to handle the special case where the top of a bottom half
        // bounding box is toching a door north of its position. Since the center of the bounding
        // box is at the top, we can run into a situation where the search below only results in
        // a single tile. We need to start the search one tile below for everything to workout.
        // Ideally, we would calculate the position one tile "away" from the door and use that as
        // an exact point to start. This works because we only have two bounding box types.
        Some(_) => pos.adjacent_south(grid.rows_len()).unwrap(),
        None => pos,
    };

    grid.depth_first_search(pos, |node, _| {
        // Stop searching at walls or closed entrances (but still include them in the result)
        !grid.get(node).is_wall() && find_door(node).is_none()
    })
}

pub(in super) fn render_area<'a, T: RenderTarget>(
    data: impl AsRef<RenderData<'a>>,
    region: Rect,
    ctx: &mut RenderContext<T>,
    should_render: impl Fn(TilePos, &Tile) -> bool + Clone,
) -> Result<(), SDLError> {
    let RenderData {map, positions, sprites: esprites, ghosts, ..} = data.as_ref();
    let render_top_left = region.top_left();

    // Rendering strategy: For each row, first render all the backgrounds, then render all of
    // entities that should be rendered under other entities, then render all other entities.
    // This allows an object to overlap the background of the tile on its right.
    render_background(&*map, region, ctx, should_render.clone())?;

    let grid = map.grid();
    let should_render_pos = |pos| {
        let tile_pos = map.world_to_tile_pos(pos);

        // Do not want to render the wall decoration if we are not going to render the
        // tile south of this wall. Reason: Objects within a room should only be visible
        // when that room is visible
        if grid.get(tile_pos).is_wall() {
            let should_render_south = tile_pos.adjacent_south(grid.rows_len())
                .map(|south| should_render(south, grid.get(south)))
                .unwrap_or(false);
            if !should_render_south {
                return false;
            }
        }

        should_render(tile_pos, grid.get(tile_pos))
    };

    render_entities((positions, esprites, ghosts).join().map(|(p, s, _)| (p, s)),
        map.tile_size(), render_top_left, ctx, should_render_pos)?;
    render_entities((positions, esprites, !ghosts).join().map(|(p, s, _)| (p, s)),
        map.tile_size(), render_top_left, ctx, should_render_pos)?;

    Ok(())
}

/// Renders the tiles of the background (map) within the given region
fn render_entities<'a, T: RenderTarget>(
    components: impl Iterator<Item=(&'a Position, &'a Sprite)>,
    tile_size: u32,
    render_top_left: Point,
    ctx: &mut RenderContext<T>,
    should_render: impl Fn(Point) -> bool,
) -> Result<(), SDLError> {
    for (&Position(pos), &Sprite(sprite)) in components {
        if !should_render(pos) {
            continue;
        }

        let sprite = ctx.sprites.get(sprite);
        // Render the sprite in a (tile_size)x(tile_size) square centered around its position.
        // TODO: If the sprite is bigger than this, it will (currently) still be rendered and not
        // clipped.
        render_sprite(pos, tile_size, sprite, ctx, render_top_left)?;
    }

    Ok(())
}

/// Renders the tiles of the background (map) within the given region
fn render_background<T: RenderTarget>(
    map: &FloorMap,
    region: Rect,
    ctx: &mut RenderContext<T>,
    mut should_render: impl FnMut(TilePos, &Tile) -> bool,
) -> Result<(), SDLError> {
    let render_top_left = region.top_left();
    // Need to paint the default floor under every tile in case the background sprite being
    // used is actually something that doesn't take up the entire space (e.g. a column tile)
    let default_floor = ctx.map_sprites.floor_sprite(Default::default());

    let tile_size = map.tile_size() as i32;
    let grid = map.grid();

    let (top_left, size) = map.grid_area_within(region);
    for (row, row_tiles) in grid.rows().enumerate().skip(top_left.row).take(size.rows) {
        for (col, tile) in row_tiles.iter().enumerate().skip(top_left.col).take(size.cols) {
            let tile_pos = TilePos {row, col};
            let pos = tile_pos.center(tile_size);

            if !should_render(tile_pos, tile) {
                // Render an empty tile
                let sprite = ctx.sprites.get(ctx.map_sprites.empty_tile_sprite());
                render_sprite(pos, tile_size as u32, sprite, ctx, render_top_left)?;
                continue;
            }

            let tile_layers = once(default_floor)
                .chain(once(tile.background_sprite(ctx.map_sprites)));

            for sprite in tile_layers {
                let sprite = ctx.sprites.get(sprite);
                render_sprite(pos, tile_size as u32, sprite, ctx, render_top_left)?;
            }
        }
    }

    Ok(())
}

fn render_sprite<T: RenderTarget>(
    center: Point,
    tile_size: u32,
    sprite: &SpriteImage,
    ctx: &mut RenderContext<T>,
    render_top_left: Point,
) -> Result<(), SDLError> {
    //TODO: This code needs to be way more robust. Currently, we make a bunch of assumptions and
    // there is actually no way that this code will work for sprites larger than one tile once we
    // make the should_render logic more complicated. In reality, what we need to do is run
    // should_render on each tile-sized area of the sprite to be rendered and then clip the parts
    // of the sprite that shouldn't be rendered. This is more complicated behaviour and we will
    // eventually need to do this to continue advancing this code.

    let texture = ctx.textures.get(sprite.texture_id);
    // Source rect should never be modified here because it represents the exact place
    // on the spritesheet of this sprite. No reaosn to modify that.
    let source_rect = sprite.region;

    // The destination rectangle that this sprite should be aligned against. The sprite
    // is not required to be confined to this rectangle. It is only used to decide how
    // the sprite's layout should be calculated.
    let dest = Rect::from_center(
        // Need to subtract the position (world coordinates) of this tile from the position
        // in world coordinates of the top-left corner of the screen so that we are left
        // with the position of this sprite on the screen in screen coordinates
        center - render_top_left,
        tile_size,
        tile_size,
    );
    let mut dest_rect = sprite.apply_anchor(dest);

    let dest_offset = sprite.dest_offset;
    dest_rect.offset(dest_offset.x(), dest_offset.y());

    ctx.canvas.copy_ex(
        texture,
        source_rect,
        dest_rect,
        0.0,
        None,
        sprite.flip_horizontal,
        sprite.flip_vertical,
    ).map_err(SDLError)
}
