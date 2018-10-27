use specs::{System, Join, ReadExpect, WriteExpect, ReadStorage, WriteStorage, Entities};

use components::{Movement, MovementDirection, KeyboardControlled};
use resources::{EventQueue, Event, ActionQueue, Action, Key};

const MOVEMENT_SPEED: i32 = 3;

#[derive(SystemData)]
pub struct KeyboardData<'a> {
    entities: Entities<'a>,
    events: ReadExpect<'a, EventQueue>,
    actions: WriteExpect<'a, ActionQueue>,
    keyboard_controlled: ReadStorage<'a, KeyboardControlled>,
    movements: WriteStorage<'a, Movement>,
}

#[derive(Default)]
pub struct Keyboard {
    /// Used to keep track of which directions were pressed most recently and which directions have
    /// still not been released. When the most recent direction is released, it is superceeded by
    /// its next most recent direction that is still pressed. When all directions have been
    /// released, the player stops.
    direction_stack: Vec<MovementDirection>,
}

// NOTE: These methods assume that KeyUp and KeyDown act as they are expected to (i.e. you can't
// have two KeyUp events for the same key before a KeyDown for that key)
impl Keyboard {
    /// Returns the current direction that movement should proceed in (if any)
    fn current_direction(&self) -> Option<MovementDirection> {
        self.direction_stack.last().cloned()
    }

    /// Adds a direction to the stack. Can be overridden by later directions.
    /// Will be kept in case the later keys are released while this one is still held.
    fn push_direction(&mut self, direction: MovementDirection) {
        self.direction_stack.push(direction);
    }

    /// Removes a direction from the direction stack and panics if the given direction was not
    /// found. If the KeyUp and KeyDown events are fired in their logical sequence, this should
    /// never happen.
    fn remove_direction(&mut self, direction: MovementDirection) {
        let index = self.direction_stack.iter()
            .position(|&d| d == direction)
            .expect("bug: attempt to remove a direction that was never added to the stack");
        self.direction_stack.remove(index);
    }
}

impl<'a> System<'a> for Keyboard {
    type SystemData = KeyboardData<'a>;

    fn run(&mut self, KeyboardData {entities, events, mut actions, keyboard_controlled, mut movements}: Self::SystemData) {
        use self::MovementDirection::*;
        use self::Event::*;
        use self::Key::*;

        // Set to true if the user has requested to interact with the tile it is facing
        let mut interact = false;

        for event in &*events {
            match event {
                KeyUp(A) => interact = true,

                // We only want the user to be able to move in one of the cardinal directions at
                // once. We override each movement based on the order in which the events arrive.
                KeyDown(UpArrow) => self.push_direction(North),
                KeyDown(RightArrow) => self.push_direction(East),
                KeyDown(DownArrow) => self.push_direction(South),
                KeyDown(LeftArrow) => self.push_direction(West),

                KeyUp(UpArrow) => self.remove_direction(North),
                KeyUp(RightArrow) => self.remove_direction(East),
                KeyUp(DownArrow) => self.remove_direction(South),
                KeyUp(LeftArrow) => self.remove_direction(West),

                _ => {},
            }
        }

        for (entity, movement, _) in (&entities, &mut movements, &keyboard_controlled).join() {
            if interact {
                actions.0.entry(entity).or_default().push(Action::Interact);
            }

            if let Some(direction) = self.current_direction() {
                movement.direction = direction;
                movement.speed = MOVEMENT_SPEED;
            } else {
                // Since the key events do not indicate that we need to move anywhere, stop moving
                movement.speed = 0;
            }
        }
    }
}
