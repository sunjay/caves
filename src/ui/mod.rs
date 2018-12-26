mod window;
mod renderer;
mod game_screen;
mod level_screen;

pub use self::window::*;
pub use self::renderer::{setup, RenderContext};
pub use self::game_screen::*;
pub use self::level_screen::*;

#[derive(Debug, Clone)]
pub struct SDLError(pub String);
