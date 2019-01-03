mod shared;
mod animator;
mod physics;
mod interactions;
mod ai;

pub use self::shared::*;
pub use self::animator::*;
pub use self::physics::*;
pub use self::interactions::*;
pub use self::ai::*;

mod keyboard;
pub type Keyboard = SharedSystem<keyboard::Keyboard>;
