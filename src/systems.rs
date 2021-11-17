mod ai;
mod animator;
mod interactions;
mod physics;
mod shared;

pub use self::ai::*;
pub use self::animator::*;
pub use self::interactions::*;
pub use self::physics::*;
pub use self::shared::*;

mod keyboard;
pub type Keyboard = SharedSystem<keyboard::Keyboard>;
