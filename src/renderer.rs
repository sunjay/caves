use std::env;
use std::cmp;

use sdl2::{
    self,
    Sdl,
    TimerSubsystem,
    EventPump,
    image::{Sdl2ImageContext, INIT_PNG},
    pixels::Color,
    rect::{Point, Rect},
    render::{TextureCreator, Canvas},
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

use texture_manager::TextureManager;
use components::{Position, Sprite, CameraFocus};
use map::GameMap;

#[derive(SystemData)]
struct RenderData<'a> {
    map: ReadExpect<'a, GameMap>,
    camera_focuses: ReadStorage<'a, CameraFocus>,
    positions: ReadStorage<'a, Position>,
    sprites: ReadStorage<'a, Sprite>,
}

pub struct Renderer {
    sdl_context: Sdl,
    /// Required to use images, but not used for anything after it is created
    _image_context: Sdl2ImageContext,
    canvas: Canvas<Window>,
}

impl Renderer {
    pub fn setup(res: &mut Resources) {
        RenderData::setup(res);
    }

    pub fn init(width: u32, height: u32) -> Result<Self, String> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
        let _image_context = sdl2::image::init(INIT_PNG).unwrap();

        // Scale display if a certain environment variable is set
        let display_scale = env::var("DISPLAY_SCALE")
            .map(|x| x.parse().expect("DISPLAY_SCALE must be a number"))
            .unwrap_or(1.0);

        //FIXME: Remove this unwrap() when we start using proper error types
        let window_width = (width as f32 * display_scale) as u32;
        let window_height = (height as f32 * display_scale) as u32;
        let window = video_subsystem.window("Robo Quest", window_width, window_height)
            .position_centered()
            .build()
            .unwrap();

        //FIXME: Remove this unwrap() when we start using proper error types
        let mut canvas = window.into_canvas()
            .accelerated()
            .present_vsync()
            .build()
            .unwrap();

        // The background color
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));

        //FIXME: Remove this unwrap() when we start using proper error types
        canvas.set_logical_size(width, height).unwrap();

        Ok(Self {
            sdl_context,
            _image_context,
            canvas,
        })
    }

    pub fn dimensions(&self) -> (u32, u32) {
        self.canvas.logical_size()
    }

    pub fn texture_creator(&self) -> TextureCreator<WindowContext> {
        self.canvas.texture_creator()
    }

    pub fn timer(&self) -> Result<TimerSubsystem, String> {
        self.sdl_context.timer()
    }

    pub fn event_pump(&self) -> Result<EventPump, String> {
        self.sdl_context.event_pump()
    }

    pub fn render(&mut self, world: &World, textures: &TextureManager) -> Result<(), String> {
        self.canvas.clear();

        let RenderData {map, positions, sprites, camera_focuses} = world.system_data();
        let mut camera_focuses = (&positions, &camera_focuses).join();
        let (&Position(camera_focus), _) = camera_focuses.next()
            .expect("Renderer was not told which entity to focus on");
        assert!(camera_focuses.next().is_none(),
            "Renderer was asked to focus on more than one thing");

        let (screen_width, screen_height) = self.dimensions();
        let screen_center = Point::new(screen_width as i32 / 2, screen_height as i32 / 2);

        // The position on the map of the screen's top left corner
        // Adding this point to the position of the camera_focus would make it render in the center
        // of the screen
        let render_center = camera_focus - screen_center;

        // Need to make sure the camera stays within the level boundary
        let level_boundary = map.level_boundary();
        // The valid ranges for the top-left corner of the screen
        let (min_x, max_x) = (0, level_boundary.x() + level_boundary.width() as i32 - screen_width as i32);
        let (min_y, max_y) = (0, level_boundary.y() + level_boundary.height() as i32 - screen_height as i32);
        let clamp = |min, x, max| cmp::min(cmp::max(min, x), max);
        let render_center = Point::new(
            clamp(min_x, render_center.x, max_x),
            clamp(min_y, render_center.y, max_y),
        );

        // Get the tiles surrounding the camera focus
        let screen = Rect::from_center(render_center + screen_center, screen_width, screen_height);

        //self.render_tiles(map.background_within(screen), render_center, textures)?;
        //self.render_tiles(map.background_items_within(screen), render_center, textures)?;

        for (&Position(pos), ref sprite) in (&positions, &sprites).join() {
            let pos = pos - render_center;
            let texture = textures.get(sprite.texture_id);
            let source_rect = sprite.region;
            let mut dest_rect = source_rect.clone();
            dest_rect.center_on(pos);

            self.canvas.copy_ex(
                texture,
                Some(source_rect),
                Some(dest_rect),
                0.0,
                None,
                sprite.flip_horizontal,
                sprite.flip_vertical,
            )?;
        }

        //self.render_tiles(map.map_within(screen), render_center, textures)?;

        self.canvas.present();

        Ok(())
    }

    //fn render_tiles<'a, I: Iterator<Item=&'a Tile>>(&mut self, tiles: I, render_center: Point, textures: &TextureManager) -> Result<(), String> {
    //    for &Tile {x, y, texture_id, image_width, image_height} in tiles {
    //        let texture = textures.get(texture_id);
    //        let source_rect = Rect::new(0, 0, image_width, image_height);
    //        let dest_rect = Rect::new(
    //            x - render_center.x(),
    //            y - render_center.y(),
    //            image_width,
    //            image_height,
    //        );
//
    //        self.canvas.copy_ex(
    //            texture,
    //            Some(source_rect),
    //            Some(dest_rect),
    //            0.0,
    //            None,
    //            false,
    //            false
    //        )?;
    //    }
//
    //    Ok(())
    //}
}
