use specs::{System, Join, ReadExpect, ReadStorage, WriteStorage};

use components::{Velocity, KeyboardControlled};
use resources::GameKeys;

#[derive(SystemData)]
pub struct KeyboardData<'a> {
    keys: ReadExpect<'a, GameKeys>,
    keyboard_controlled: ReadStorage<'a, KeyboardControlled>,
    velocities: WriteStorage<'a, Velocity>,
}

pub struct Keyboard;

impl<'a> System<'a> for Keyboard {
    type SystemData = KeyboardData<'a>;

    fn run(&mut self, KeyboardData {keys, keyboard_controlled, mut velocities}: Self::SystemData) {
        for (Velocity {x, y}, _) in (&mut velocities, &keyboard_controlled).join() {
            *x = cancel_opposites(keys.right_arrow, keys.left_arrow, 2);
            *y = cancel_opposites(keys.down_arrow, keys.up_arrow, 2);
        }
    }
}

/// Returns +value if only pos is true, -value if only neg is true
/// and 0 if both or neither are true. This models the "cancelling"
/// of two opposites
fn cancel_opposites(pos: bool, neg: bool, value: i32) -> i32 {
    match (pos, neg) {
        (true, false) => value,
        (false, true) => -value,
        _ => 0,
    }
}
