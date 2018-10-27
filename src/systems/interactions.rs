//! Manages interactions between entities and adjacent tiles

use std::borrow::Cow;

use specs::{System, Join, ReadExpect, WriteExpect, ReadStorage, Entities};

use components::{Position, Movement, MovementDirection};
use resources::{ActionQueue, Action};
use map::{GameMap, TileObject, Door};

#[derive(SystemData)]
pub struct InteractionsData<'a> {
    entities: Entities<'a>,
    actions: ReadExpect<'a, ActionQueue>,
    map: WriteExpect<'a, GameMap>,
    positions: ReadStorage<'a, Position>,
    movements: ReadStorage<'a, Movement>,
}

#[derive(Default)]
pub struct Interactions;

impl<'a> System<'a> for Interactions {
    type SystemData = InteractionsData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let InteractionsData {entities, actions, mut map, positions, movements} = data;
        let level = map.current_level_map_mut();

        for (entity, &Position(pos), &Movement {direction, ..}) in (&*entities, &positions, &movements).join() {
            let actions = actions.0.get(&entity).map(Cow::Borrowed).unwrap_or_default();

            // Process any requests to interact with an adjacent tile
            if !actions.iter().any(|&a| a == Action::Interact) {
                continue;
            }

            // The tile underneath this position
            let pos_tile = level.world_to_tile_pos(pos);

            // The tile adjacent to the current position that we will be interacting with
            use self::MovementDirection::*;
            let adjacent = match direction {
                North => pos_tile.adjacent_north(),
                South => pos_tile.adjacent_south(level.grid().rows_len()),
                East => pos_tile.adjacent_east(level.grid().cols_len()),
                West => pos_tile.adjacent_west(),
            };

            match adjacent.and_then(|adj| level.grid_mut().get_mut(adj).object_mut()) {
                // Open a door that was previously closed
                Some(TileObject::Door {state: state@Door::Closed, ..}) => *state = Door::Open,
                _ => {},
            }
        }
    }
}
