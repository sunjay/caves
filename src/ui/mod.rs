mod window;
mod renderer;
mod texture_manager;
mod game_screen;

pub use self::window::*;
pub use self::renderer::setup;
pub use self::texture_manager::*;
pub use self::game_screen::*;

#[derive(Debug, Clone)]
pub struct SDLError(String);
