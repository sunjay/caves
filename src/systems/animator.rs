use specs::{System, Join, ReadExpect, ReadStorage, WriteStorage};

use components::{Velocity, Sprite, MovementAnimation};
use resources::FramesElapsed;

#[derive(SystemData)]
pub struct AnimatorData<'a> {
    frames: ReadExpect<'a, FramesElapsed>,
    velocities: ReadStorage<'a, Velocity>,
    sprites: WriteStorage<'a, Sprite>,
    animations: WriteStorage<'a, MovementAnimation>,
}

pub struct Animator;

impl<'a> System<'a> for Animator {
    type SystemData = AnimatorData<'a>;

    fn run(&mut self, AnimatorData {frames, velocities, mut sprites, mut animations}: Self::SystemData) {
        let FramesElapsed(frames_elapsed) = *frames;

        for (vel, sprite, animation) in (&velocities, &mut sprites, &mut animations).join() {
            if vel.x > 0 {
                // The assumption is that the sprite begins facing right
                sprite.flip_horizontal = false;
            }
            else if vel.x < 0 {
                sprite.flip_horizontal = true;
            }
            else { // No horizontal movement
                // Only continue to animate if moving
                continue;
            }

            animation.frame_counter += frames_elapsed;
            let current_step = animation.frame_counter % (animation.steps.len() * animation.frames_per_step) / animation.frames_per_step;

            let (current_texture_id, current_region) = animation.steps[current_step];
            sprite.texture_id = current_texture_id;
            sprite.region = current_region;
        }
    }
}
