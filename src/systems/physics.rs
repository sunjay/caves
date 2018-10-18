use sdl2::rect::Rect;
use specs::{System, Join, ReadExpect, ReadStorage, WriteStorage, Entities, LazyUpdate};

use components::{Movement, Position, Wait, BoundingBox};
use resources::FramesElapsed;
use map::GameMap;

// Collisions within this threshold will be *ignored*
const COLLISION_THRESHOLD: i32 = 2;

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

        for (entity, Position(pos), &Movement {direction, speed}) in (&entities, &mut positions, &movements).join() {
            // Entity is waiting for a given amount of frames to elapse
            if let Some(wait) = waits.get_mut(entity) {
                wait.frames_elapsed += frames_elapsed;
                if wait.frames_elapsed >= wait.duration {
                    updater.remove::<Wait>(entity);
                }
                continue;
            }

            let frames_elapsed = frames_elapsed as i32;

            // Try to move as much as possible
            for speed in (0..=speed).rev() {
                let next_pos = *pos + direction.to_vector() * speed * frames_elapsed;

                if let Some(&BoundingBox {width, height}) = bounding_boxes.get(entity) {
                    let bounds = Rect::from_center(
                        next_pos,
                        width - COLLISION_THRESHOLD as u32 * 2,
                        height - COLLISION_THRESHOLD as u32 * 2,
                    );

                    // Check if any of the tiles that this new position intersects with is a wall
                    if level.tiles_within(bounds).find(|(_, pt, _)| level.grid().is_wall(*pt)).is_some() {
                        // Do not update the position
                        continue;
                    }

                    //TODO: Check for collisions with other bounding boxes
                }

                *pos = next_pos;
                break;
            }
        }
    }
}
