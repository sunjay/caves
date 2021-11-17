mod game_screen;
mod level_screen;
mod renderer;
mod text;
mod window;

pub mod debug;

pub use self::game_screen::*;
pub use self::level_screen::*;
pub use self::renderer::*;
pub use self::text::*;
pub use self::window::*;

#[derive(Debug, Clone)]
pub struct SDLError(pub String);
