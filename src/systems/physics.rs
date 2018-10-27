use sdl2::rect::Rect;
use specs::{System, Join, ReadExpect, ReadStorage, WriteStorage, Entities, LazyUpdate};

use components::{Movement, Position, Wait, BoundingBox};
use resources::FramesElapsed;
use map::GameMap;

// Collisions within this threshold will be *ignored*
const COLLISION_THRESHOLD: u32 = 1;

#[derive(SystemData)]
pub struct PhysicsData<'a> {
    entities: Entities<'a>,
    frames: ReadExpect<'a, FramesElapsed>,
    map: ReadExpect<'a, GameMap>,
    movements: ReadStorage<'a, Movement>,
    bounding_boxes: ReadStorage<'a, BoundingBox>,
    waits: WriteStorage<'a, Wait>,
    positions: WriteStorage<'a, Position>,
    updater: ReadExpect<'a, LazyUpdate>,
}

pub struct Physics;

impl<'a> System<'a> for Physics {
    type SystemData = PhysicsData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let PhysicsData {entities, frames, map, movements, bounding_boxes, mut positions, mut waits, updater} = data;
        let FramesElapsed(frames_elapsed) = *frames;
        let level = map.current_level_map();
        let tile_size = level.tile_size();

        // Need to do updating in a separate phase so we can read all the positions in a nested loop
        let mut updates = Vec::new();
        for (entity, Position(pos), &Movement {direction, speed}) in (&entities, &positions, &movements).join() {
            // Entity is waiting for a given amount of frames to elapse
            if let Some(wait) = waits.get_mut(entity) {
                wait.frames_elapsed += frames_elapsed;
                if wait.frames_elapsed >= wait.duration {
                    updater.remove::<Wait>(entity);
                }
                continue;
            }

            let frames_elapsed = frames_elapsed as i32;

            let mut next_pos = *pos + direction.to_vector() * speed * frames_elapsed;

            if let Some(&bounds_box) = bounding_boxes.get(entity) {
                // Shrink by the threshold so we don't detect collisions too eagerly
                let bounds_box = bounds_box.shrink(COLLISION_THRESHOLD);
                let bounds = bounds_box.to_rect(next_pos);

                // Check if any of the tiles that this new position intersects with is a wall or
                // any other tile that should not be traversed
                let potential_collisions = level.tiles_within(bounds)
                    .filter(|(_, _, tile)| !tile.is_traversable())
                    .map(|(pos, _, _)| Rect::new(
                        pos.x(),
                        pos.y(),
                        tile_size,
                        tile_size,
                    ));
                let potential_collisions = potential_collisions
                    .chain((&entities, &positions, &bounding_boxes).join()
                    .filter_map(|(other, &Position(other_pos), &bounds_box)| {
                        // Do not collide with self
                        if entity == other { return None; }

                        // Shrink by the threshold so we don't detect collisions too eagerly
                        let bounds_box = bounds_box.shrink(COLLISION_THRESHOLD);

                        Some(bounds_box.to_rect(other_pos))
                    }));

                for other in potential_collisions {
                    // Recalculate bounds based on latest next_pos
                    let bounds = bounds_box.to_rect(next_pos);

                    // Need to recalculate the intersection since we are changing next_pos in each
                    // iteration. Would not make sense to precalculate the intersections when
                    // collecting potential collision objects
                    if let Some(rect) = bounds.intersection(other) {
                        // Do the minimal amount of movement in one direction to avoid the collision
                        if rect.width() <= rect.height() {
                            let adjustment = rect.width() as i32;
                            if rect.x() > next_pos.x() {
                                // Collision was on the right so we'll move left
                                next_pos = next_pos.offset(-adjustment, 0);
                            } else {
                                // Collision was on the left so we'll move right
                                next_pos = next_pos.offset(adjustment, 0);
                            }
                        } else {
                            let adjustment = rect.height() as i32;
                            // Need to make sure to use > instead of >= here or else we will fly
                            // through walls when moving up into them. Do not want to move up by
                            // the given adjustment when the colliding object is already above us.
                            if rect.y() > next_pos.y() {
                                // Collision was below so we'll move up
                                next_pos = next_pos.offset(0, -adjustment);
                            } else {
                                // Collision was above so we'll move down
                                next_pos = next_pos.offset(0, adjustment);
                            }
                        }
                    }
                }

                updates.push((entity, next_pos));
                break;
            }
        }

        for (entity, next_pos) in updates {
            if let Some(Position(pos)) = positions.get_mut(entity) {
                *pos = next_pos;
            }
        }
    }
}
