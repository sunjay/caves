use specs::{System, Join, ReadExpect, ReadStorage, WriteStorage};

use components::{Movement, MovementDirection, KeyboardControlled};
use resources::{EventQueue, Event, Key};

const MOVEMENT_SPEED: i32 = 2;

#[derive(SystemData)]
pub struct KeyboardData<'a> {
    events: ReadExpect<'a, EventQueue>,
    keyboard_controlled: ReadStorage<'a, KeyboardControlled>,
    movements: WriteStorage<'a, Movement>,
}

pub struct Keyboard;

impl<'a> System<'a> for Keyboard {
    type SystemData = KeyboardData<'a>;

    fn run(&mut self, KeyboardData {events, keyboard_controlled, mut movements}: Self::SystemData) {
        for (movement, _) in (&mut movements, &keyboard_controlled).join() {
            use self::MovementDirection::*;
            use self::Event::*;
            use self::Key::*;
            macro_rules! move_towards {
                ($direction:ident) => {
                    {
                        movement.direction = $direction;
                        movement.speed = MOVEMENT_SPEED;
                    }
                };
            }

            // We only want the user to be able to move in one of the cardinal directions at once.
            // We override each movement based on the order in which the events arrive.
            for event in &*events {
                match event {
                    KeyDown(UpArrow) => move_towards!(North),
                    KeyDown(RightArrow) => move_towards!(East),
                    KeyDown(DownArrow) => move_towards!(South),
                    KeyDown(LeftArrow) => move_towards!(West),
                    // Only stop if we are still moving in the direction of the key that was
                    // released
                    KeyUp(UpArrow) if movement.direction == North => movement.speed = 0,
                    KeyUp(RightArrow) if movement.direction == East => movement.speed = 0,
                    KeyUp(DownArrow) if movement.direction == South => movement.speed = 0,
                    KeyUp(LeftArrow) if movement.direction == West => movement.speed = 0,
                    _ => {},
                }
            }
        }
    }
}
