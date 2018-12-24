//! Manages interactions between entities and adjacent tiles

use sdl2::rect::{Point, Rect};
use specs::{Entity, System, Join, ReadExpect, WriteExpect, ReadStorage, WriteStorage, Entities};

use components::{
    Position,
    BoundingBox,
    Movement,
    MovementDirection,
    Player,
    Stairs,
    Door,
    HealthPoints,
};
use resources::{ActionQueue, Action, ChangeGameState, GameState};
use map::FloorMap;

#[derive(SystemData)]
pub struct InteractionsData<'a> {
    entities: Entities<'a>,
    change_game_state: WriteExpect<'a, ChangeGameState>,
    actions: ReadExpect<'a, ActionQueue>,
    map: ReadExpect<'a, FloorMap>,
    positions: ReadStorage<'a, Position>,
    bounding_boxes: ReadStorage<'a, BoundingBox>,
    movements: ReadStorage<'a, Movement>,
    players: ReadStorage<'a, Player>,
    stairs: ReadStorage<'a, Stairs>,
    doors: WriteStorage<'a, Door>,
    healths: WriteStorage<'a, HealthPoints>,
}

impl<'a> InteractionsData<'a> {
    /// Attempts to interact with an entity adjacent to this entity in the given direction
    pub fn interact_with_adjacent(&mut self, entity: Entity) {
        let (pos, direction, bounds) = self.position_movement_bounds(entity);
        // Want to be very close when interacting
        let range = self.map.tile_size() as i32 / 4;
        for (other_entity, _) in self.nearest_in_direction(entity, pos, direction, bounds, range) {
            if self.doors.get(other_entity).is_some() {
                self.entities.delete(other_entity)
                    .expect("bug: unable to delete door");
                break; // stop at the first interaction
            }
        }
    }

    /// Attempts to attack an entity adjacent to this entity in the given direction
    pub fn attack_adjacent(&mut self, entity: Entity) {
        let (pos, direction, bounds) = self.position_movement_bounds(entity);
        // Most attacks take up an entire tile length in a given direction
        let range = self.map.tile_size() as i32;
        for (other_entity, other_pos) in self.nearest_in_direction(entity, pos, direction, bounds, range) {
            if self.doors.get(other_entity).is_some() {
                self.entities.delete(other_entity)
                    .expect("bug: unable to delete door");
            }

            //TODO: Attack any nearby entities in the given direction. Lower the HealthPoints
            // component of anything that gets hit. Anyone nearby in the direction of the method
            // should be hit.
        }
    }

    fn position_movement_bounds(&self, entity: Entity) -> (Point, MovementDirection, BoundingBox) {
        match (self.positions.get(entity), self.movements.get(entity), self.bounding_boxes.get(entity)) {
            (Some(&Position(pos)), Some(movement), Some(&bounds)) => (pos, movement.direction, bounds),
            _ => unreachable!("bug: only entities with positions, movement directions, and a bounding box can interact"),
        }
    }

    /// Returns the nearest entities in the given direction. Only entities that are up to `range`
    /// away are returned. Result is sorted nearest to farthest.
    fn nearest_in_direction(
        &self,
        entity: Entity,
        pos: Point,
        direction: MovementDirection,
        bounds: BoundingBox,
        range: i32,
    ) -> impl Iterator<Item=(Entity, Point)> {
        //TODO: Maybe instead of a (tile_size)x(tile_size) box we should consider a custom radius.
        // This might be useful because we know that attacks don't necessary take up the entire
        // adjacent tile. We also don't want to interact with things that are too far away.
        //TODO: Filter by entity != other_entity so the entity being searched for isn't returned.
        //TODO: If entity has a bounding box, start from the `direction` side of that box and
        // construct a Rect of dimensions (tile_size)x(tile_size) in the given direction
        //TODO: If both entity and other_entity have bounding boxes, we need to use those to find
        // the distance instead of just the point itself. The algorithm will find the distance
        // between two rectangles instead of just two points
        let bounds = bounds.to_rect(pos);

        // Generate the rectangle that the other bounding box must intersect with
        // Assumption: bounding boxes do not intersect (due to the physics engine)
        use self::MovementDirection::*;
        let direction_box = match direction {
            North => Rect::from_center(
                Point::new(pos.x(), bounds.top() - range / 2),
                range as u32,
                range as u32,
            ),
            South => Rect::from_center(
                Point::new(pos.x(), bounds.bottom() + range / 2),
                range as u32,
                range as u32,
            ),
            East => Rect::from_center(
                Point::new(bounds.right() + range / 2, pos.y()),
                range as u32,
                range as u32,
            ),
            West => Rect::from_center(
                Point::new(bounds.left() - range / 2, pos.y()),
                range as u32,
                range as u32,
            ),
        };

        let mut near = Vec::new();
        for (other, &Position(other_pos)) in (&self.entities, &self.positions).join() {
            if entity == other {
                continue;
            }

            // Using the full boundary (regardless of the bounding box type) because we want
            // entities to be found regardless of whether their full height is used in collision
            // detection
            let other_bounds = self.bounding_boxes.get(other)
                .map(|b| b.to_full_rect(other_pos))
                .unwrap_or_else(|| Rect::from_center(other_pos, 0, 0));

            if direction_box.has_intersection(other_bounds) {
                near.push((other, other_pos, other_bounds));
            }
        }

        // Return result sorted by the distance *between* the boundary rectangles in the given
        // direction
        match direction {
            North => near.sort_unstable_by_key(|(_, _, other_bounds)| {
                (bounds.top() - other_bounds.bottom()).abs()
            }),
            South => near.sort_unstable_by_key(|(_, _, other_bounds)| {
                (other_bounds.top() - bounds.bottom()).abs()
            }),
            East => near.sort_unstable_by_key(|(_, _, other_bounds)| {
                (other_bounds.left() - bounds.right()).abs()
            }),
            West => near.sort_unstable_by_key(|(_, _, other_bounds)| {
                (bounds.left() - other_bounds.right()).abs()
            }),
        }

        near.into_iter().map(|(other, other_pos, _)| (other, other_pos))
    }
}

#[derive(Default)]
pub struct Interactions;

impl<'a> System<'a> for Interactions {
    type SystemData = InteractionsData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        // Cloning this isn't great, but it's the only way to get around borrowing issues since
        // Rust doesn't do per-field mutability
        let actions = data.actions.0.clone();
        for (entity, actions) in actions.into_iter() {
            for action in actions {
                use self::Action::*;
                match action {
                    Interact => data.interact_with_adjacent(entity),
                    Attack => data.attack_adjacent(entity),
                    // None of these require interaction with an adjacent tile
                    Hit | Victory | Defeat => {},
                }
            }
        }

        let InteractionsData {
            entities,
            mut change_game_state,
            positions,
            bounding_boxes,
            players,
            stairs,
            ..
        } = data;

        // If the player is intersecting with anything interesting, we may be need to do something
        for (&Position(pos), bounds, _) in (&positions, &bounding_boxes, &players).join() {
            let player_box = bounds.to_rect(pos);
            for (other_entity, &Position(other_pos), other_bounds, ()) in (&*entities, &positions, &bounding_boxes, !&players).join() {
                let other_box = other_bounds.to_rect(other_pos);
                if player_box.has_intersection(other_box) {
                    // If player entered a staircase, we need to move to the next/prev level
                    if let Some(staircase) = stairs.get(other_entity) {
                        let change = match staircase {
                            &Stairs::ToNextLevel {id} => GameState::GoToNextLevel {id},
                            &Stairs::ToPrevLevel {id} => GameState::GoToPrevLevel {id},
                        };
                        change_game_state.replace(change);
                    }
                }
            }
        }
    }
}
