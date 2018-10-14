use specs::{System, Join, ReadExpect, ReadStorage, WriteStorage, Entities, LazyUpdate};

use components::{Movement, Position, Wait, BoundingBox};
use resources::FramesElapsed;

#[derive(SystemData)]
pub struct PhysicsData<'a> {
    entities: Entities<'a>,
    frames: ReadExpect<'a, FramesElapsed>,
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
        let PhysicsData {entities, frames, movements, bounding_boxes, mut positions, mut waits, updater} = data;
        let FramesElapsed(frames_elapsed) = *frames;

        for (entity, Position(pos), movement) in (&entities, &mut positions, &movements).join() {
            // Entity is waiting for a given amount of frames to elapse
            if let Some(wait) = waits.get_mut(entity) {
                wait.frames_elapsed += frames_elapsed;
                if wait.frames_elapsed >= wait.duration {
                    updater.remove::<Wait>(entity);
                }
                continue;
            }

            let frames_elapsed = frames_elapsed as i32;
            //TODO: Move one tile in the direction of movement (over a certain period based on a speed)
            // If we partially move on to a tile, finish the movement (or otherwise move back on to
            // the current tile if we haven't moved enough yet)
            // *pos = pos.offset(x * frames_elapsed, y * frames_elapsed);

            if let Some(BoundingBox {width: _, height: _}) = bounding_boxes.get(entity) {
                //TODO: Ensure that entity does not leave boundary
            }
        }
    }
}
