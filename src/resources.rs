//! ECS Resources for use by various systems

use std::collections::HashMap;

use sdl2::keyboard::Scancode;
use specs::Entity;

/// Resource that represents the number of frames elapsed since the last time all of the systems
/// were run. Value is guaranteed to be greater than or equal to 1.
/// Often this will be just 1 but it may be greater if there is lag or if a system takes too long.
pub struct FramesElapsed(pub usize);

/// Resource that represents any events that have taken place before the current frame.
///
/// This queue resets every frame
#[derive(Debug, Default)]
pub struct EventQueue(pub Vec<Event>);

impl<'a> IntoIterator for &'a EventQueue {
    type Item = &'a Event;
    type IntoIter = ::std::slice::Iter<'a, Event>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

/// Represents an event from the user of the application
#[derive(Debug, Clone)]
pub enum Event {
    KeyDown(Key),
    KeyUp(Key),
}

/// Represents the key that was pressed/released
#[derive(Debug, Clone, Copy)]
pub enum Key {
    UpArrow,
    DownArrow,
    LeftArrow,
    RightArrow,
    Menu,
    Select,
    Start,
    VolumeDown,
    VolumeUp,
    X,
    Y,
    A,
    B,
    LightKey1,
    LightKey2,
    // LightKey3, //FIXME: No way to detect this yet
    LightKey4,
    LightKey5,
}

impl Key {
    /// Attempts to convert the given scan code to a key. Returns None if the key was not one of
    /// the supported keys.
    pub fn from_scancode(code: Scancode) -> Option<Self> {
        // From mapping: https://github.com/clockworkpi/Keypad#keymaps
        use self::Key::*;
        Some(match code {
            Scancode::Up => UpArrow,
            Scancode::Down => DownArrow,
            Scancode::Left => LeftArrow,
            Scancode::Right => RightArrow,
            Scancode::Escape => Menu,
            Scancode::Space => Select,
            Scancode::Return => Start,
            Scancode::KpMinus => VolumeDown,
            Scancode::KpPlus => VolumeUp,
            Scancode::I => X,
            Scancode::U => Y,
            Scancode::K => A,
            Scancode::J => B,
            Scancode::H => LightKey1,
            Scancode::Y => LightKey2,
            //?? => LightKey3, //FIXME: No way to check if Shift key pressed
            Scancode::O => LightKey4,
            Scancode::L => LightKey5,
            _ => return None,
        })
    }
}

/// Resource that represents an intention to change the game state
#[derive(Debug, Default)]
pub struct ChangeGameState(Option<GameState>);

impl ChangeGameState {
    /// Replaces the
    pub fn replace(&mut self, state: GameState) -> Option<GameState> {
        //TODO: Can replace this with a single call to Option::replace once that is stablized.
        // https://doc.rust-lang.org/std/option/enum.Option.html#method.replace
        let prev = self.0.take();
        self.0 = Some(state);
        prev
    }
}

/// Changes to the game state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    /// Game should change to the next level (and move the player and its components there)
    GoToNextLevel {id: usize},
    /// Game should change to the previous level (and move the player and its components there)
    GoToPrevLevel {id: usize},
    /// Game should pause, but stay on the same level
    Pause,
    //TODO: PauseToShowMessage or something for when we want to show some info
}

/// Resource that represents any actions that have happened during the current frame.
///
/// This queue resets every frame
#[derive(Debug, Default)]
pub struct ActionQueue(pub HashMap<Entity, Vec<Action>>);

/// Actions that an entity can take or have happen to them during a frame
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// The entity requested to interact with the tile/item it is currently facing
    Interact,
    /// The entity performed its attack
    Attack,
    /// The entity was hit by something and took damage
    Hit,
    /// The entity completed something
    Victory,
    /// The entity was defeated in battle (0 HP)
    Defeat,
}
