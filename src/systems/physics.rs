use specs::{System, Join, ReadExpect, ReadStorage, WriteStorage, Entities};

use components::{Velocity, Position, BoundingBox};
use resources::FramesElapsed;

#[derive(SystemData)]
pub struct PhysicsData<'a> {
    entities: Entities<'a>,
    frames: ReadExpect<'a, FramesElapsed>,
    velocities: ReadStorage<'a, Velocity>,
    bounding_boxes: ReadStorage<'a, BoundingBox>,
    positions: WriteStorage<'a, Position>
}

pub struct Physics;

impl<'a> System<'a> for Physics {
    type SystemData = PhysicsData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let PhysicsData {entities, frames, velocities, bounding_boxes, mut positions} = data;
        let FramesElapsed(frames_elapsed) = *frames;
        let frames_elapsed = frames_elapsed as i32;

        for (entity, Position(pos), &Velocity {x, y}) in (&entities, &mut positions, &velocities).join() {
            *pos = pos.offset(x * frames_elapsed, y * frames_elapsed);

            if let Some(BoundingBox {width: _, height: _}) = bounding_boxes.get(entity) {
                //TODO: Ensure that entity does not leave boundary
            }
        }
    }
}
