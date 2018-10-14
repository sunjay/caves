use specs::{System, Join, ReadExpect, ReadStorage, WriteStorage};

use components::{Movement, MovementDirection, KeyboardControlled};
use resources::GameKeys;

#[derive(SystemData)]
pub struct KeyboardData<'a> {
    keys: ReadExpect<'a, GameKeys>,
    keyboard_controlled: ReadStorage<'a, KeyboardControlled>,
    movements: WriteStorage<'a, Movement>,
}

pub struct Keyboard;

impl<'a> System<'a> for Keyboard {
    type SystemData = KeyboardData<'a>;

    fn run(&mut self, KeyboardData {keys, keyboard_controlled, mut movements}: Self::SystemData) {
        for (movement, _) in (&mut movements, &keyboard_controlled).join() {
            // We only want the user to be able to move in one of the cardinal directions at once.
            // If they press multiple keys, we just choose whatever direction matches first below.
            match (keys.up_arrow, keys.right_arrow, keys.down_arrow, keys.left_arrow) {
                (true, false, false, false) => movement.move_in_direction(MovementDirection::North),
                (false, true, false, false) => movement.move_in_direction(MovementDirection::East),
                (false, false, true, false) => movement.move_in_direction(MovementDirection::South),
                (false, false, false, true) => movement.move_in_direction(MovementDirection::West),
                _ => movement.is_moving = false,
            }
        }
    }
}
