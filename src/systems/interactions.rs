//! Manages interactions between entities and adjacent tiles

use std::borrow::Cow;

use sdl2::rect::Point;
use specs::{System, Join, ReadExpect, WriteExpect, ReadStorage, WriteStorage, Entities};

use components::{Position, Movement, MovementDirection, CameraFocus};
use resources::{ActionQueue, Action};
use map::{GameMap, TileObject, Door};

#[derive(SystemData)]
pub struct InteractionsData<'a> {
    entities: Entities<'a>,
    actions: ReadExpect<'a, ActionQueue>,
    map: WriteExpect<'a, GameMap>,
    positions: WriteStorage<'a, Position>,
    movements: ReadStorage<'a, Movement>,
    camera_focuses: ReadStorage<'a, CameraFocus>,
}

#[derive(Default)]
pub struct Interactions;

impl<'a> System<'a> for Interactions {
    type SystemData = InteractionsData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let InteractionsData {entities, actions, mut map, mut positions, movements, camera_focuses} = data;

        for (entity, &Position(pos), &Movement {direction, ..}) in (&*entities, &positions, &movements).join() {
            let actions = actions.0.get(&entity).map(Cow::Borrowed).unwrap_or_default();

            // Process any requests to interact with an adjacent tile
            if !actions.iter().any(|&a| a == Action::Interact) {
                continue;
            }

            self.interact_with_tile(&mut map, pos, direction);
        }

        // If the camera focus has entered a special tile, we may need to perform an action

        // Used to center the camera focus at its destination tile
        let half_tile_size = map.current_level_map().tile_size() as i32 / 2;
        let tile_center = Point::new(half_tile_size, half_tile_size);
        for (Position(ref mut pos), _) in (&mut positions, &camera_focuses).join() {
            {
                // Check if we stepped on a ToNextLevel tile
                let next_level_id = {
                    let level = map.current_level_map();
                    let pos = level.world_to_tile_pos(*pos);
                    level.grid().get(pos).to_next_level_id()
                };
                if let Some(id) = next_level_id {
                    map.to_next_level();

                    let level = map.current_level_map();
                    // Find the opposite gate with the same ID
                    let tile_pos = level.grid().find_to_prev_level(id)
                        .expect("bug: could not find matching previous level gate");
                    // Find the only traversable adjacent
                    let target_pos = level.grid().adjacent_positions(tile_pos)
                        .find(|&pt| level.grid().get(pt).is_traversable())
                        .expect("bug: no traversable tile beside ToPrevLevel gate");
                    *pos = target_pos.to_point(level.tile_size() as i32) + tile_center;
                }
            }

            {
                // Check if we stepped on a ToPrevLevel tile
                let prev_level_id = {
                    let level = map.current_level_map();
                    let pos = level.world_to_tile_pos(*pos);
                    level.grid().get(pos).to_prev_level_id()
                };
                if let Some(id) = prev_level_id {
                    map.to_prev_level();

                    let level = map.current_level_map();
                    // Find the opposite gate with the same ID
                    let tile_pos = level.grid().find_to_next_level(id)
                        .expect("bug: could not find matching next level gate");
                    // Find the only traversable adjacent
                    let target_pos = level.grid().adjacent_positions(tile_pos)
                        .find(|&pt| level.grid().get(pt).is_traversable())
                        .expect("bug: no traversable tile beside ToNextLevel gate");
                    *pos = target_pos.to_point(level.tile_size() as i32) + tile_center;
                }
            }
        }
    }
}

impl Interactions {
    fn interact_with_tile(&mut self, map: &mut GameMap, pos: Point, direction: MovementDirection) {
        let level = map.current_level_map_mut();

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
