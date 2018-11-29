use std::env;

use sdl2::{
    self,
    Sdl,
    TimerSubsystem,
    EventPump,
    image::{Sdl2ImageContext, INIT_PNG},
    pixels::Color,
    render::{TextureCreator, Canvas},
    video::{Window as SDLWindow, WindowContext},
};

use super::SDLError;

pub struct Window {
    sdl_context: Sdl,
    /// Required to use images, but not used for anything after it is created
    _image_context: Sdl2ImageContext,
    canvas: Canvas<SDLWindow>,
}

impl Window {
    pub fn init(width: u32, height: u32) -> Result<Self, SDLError> {
        let sdl_context = sdl2::init().map_err(SDLError)?;
        let video_subsystem = sdl_context.video().map_err(SDLError)?;
        let _image_context = sdl2::image::init(INIT_PNG).unwrap();

        // Scale display if a certain environment variable is set
        let display_scale = env::var("DISPLAY_SCALE")
            .map(|x| x.parse().expect("DISPLAY_SCALE must be a number"))
            .unwrap_or(1.0);

        //FIXME: Remove this unwrap() when we start using proper error types
        let window_width = (width as f32 * display_scale) as u32;
        let window_height = (height as f32 * display_scale) as u32;
        let window = video_subsystem.window("Caves", window_width, window_height)
            .position_centered()
            .resizable()
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

        // Scales the game *within* the window so it is easier to see things
        let zoom = 2;

        //FIXME: Remove this unwrap() when we start using proper error types
        canvas.set_logical_size(width / zoom, height / zoom).unwrap();

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

    pub fn timer(&self) -> Result<TimerSubsystem, SDLError> {
        self.sdl_context.timer().map_err(SDLError)
    }

    pub fn event_pump(&self) -> Result<EventPump, SDLError> {
        self.sdl_context.event_pump().map_err(SDLError)
    }

    pub fn canvas_mut(&mut self) -> &mut Canvas<SDLWindow> {
        &mut self.canvas
    }
}
