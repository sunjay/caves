use specs::{System, Join, ReadExpect, ReadStorage, WriteStorage};

use components::{Movement, MovementDirection, KeyboardControlled};
use resources::GameKeys;

const MOVEMENT_SPEED: i32 = 2;

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
            movement.speed = MOVEMENT_SPEED;
            match (keys.up_arrow, keys.right_arrow, keys.down_arrow, keys.left_arrow) {
                (true, false, false, false) => movement.direction = MovementDirection::North,
                (false, true, false, false) => movement.direction = MovementDirection::East,
                (false, false, true, false) => movement.direction = MovementDirection::South,
                (false, false, false, true) => movement.direction = MovementDirection::West,
                _ => movement.speed = 0,
            }
        }
    }
}
