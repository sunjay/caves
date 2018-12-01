use std::borrow::Cow;

use specs::{System, Join, ReadExpect, WriteExpect, ReadStorage, WriteStorage, Entities};

use components::{Movement, MovementDirection::*, Sprite, Animation, AnimationManager, Wait};
use resources::{ActionQueue, Action::*, FramesElapsed};

/// The number of frames that an entity can be idle before the idle animation starts
const IDLE_LENGTH: usize = 300;

#[derive(SystemData)]
pub struct AnimatorData<'a> {
    entities: Entities<'a>,
    action_queue: ReadExpect<'a, ActionQueue>,
    frames: ReadExpect<'a, FramesElapsed>,
    movements: ReadStorage<'a, Movement>,
    sprites: WriteStorage<'a, Sprite>,
    animations: WriteStorage<'a, Animation>,
    animation_managers: WriteStorage<'a, AnimationManager>,
    waits: WriteStorage<'a, Wait>,
}

pub struct Animator;

impl<'a> System<'a> for Animator {
    type SystemData = AnimatorData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let AnimatorData {
            entities,
            action_queue,
            frames,
            movements,
            mut sprites,
            mut animations,
            mut animation_managers,
            mut waits,
        } = data;

        let FramesElapsed(frames_elapsed) = *frames;
        let ActionQueue(ref action_queue) = *action_queue;

        // NOTE: This code often needs to compare the frames in the animation for equality. If we
        // could either name each animation or store a specialized frame list that keeps around a
        // hash of its contents, we could make that comparision much faster.

        // Set the current animation based on an entity's movements or based on actions that have
        // occurred during this frame
        for (entity, movement, animation, manager) in (&entities, &movements, &mut animations, &mut animation_managers).join() {
            // No point in continuing if we can't interrupt the animation that is currently running
            // This also prevents the idle counter from being incremented during an animation
            if !animation.can_interrupt && !animation.is_complete() {
                continue;
            }

            let direction = movement.direction;

            // Don't want to copy the events that occurred but also don't want to deal with the
            // option type
            let actions: Cow<Vec<_>> = action_queue.get(&entity).map(|q| Cow::Borrowed(q)).unwrap_or_default();

            // Update the idle counter so we can decide whether to play the idle animation
            match (movement.is_moving(), &actions[..]) {
                // We are idle as long as we are not moving and no actions have occurred
                (false, []) => {
                    manager.idle_counter += frames_elapsed;

                    // Start the idle animation if we have passed the threshold and if we are not
                    // already playing this animation
                    if manager.idle_counter >= IDLE_LENGTH {
                        animation.update_if_different(&manager.idle);
                    } else {
                        // This code needs to be in this else so that it does not conflict with the
                        // idle animation

                        // No longer moving, so stop that animation
                        match direction {
                            North => animation.update_if_different(&manager.stopped_up),
                            East => animation.update_if_different(&manager.stopped_right),
                            South => animation.update_if_different(&manager.stopped_down),
                            West => animation.update_if_different(&manager.stopped_left),
                        }
                    }

                    continue;
                },
                // If we are moving, actions have occurred, or both of those are happening, we are
                // no longer idle
                _ => manager.idle_counter = 0,
            };

            // The order of this code is important: movement animations are overridden by actions

            if movement.is_moving() {
                match direction {
                    North => animation.update_if_different(&manager.move_up),
                    East => animation.update_if_different(&manager.move_right),
                    South => animation.update_if_different(&manager.move_down),
                    West => animation.update_if_different(&manager.move_left),
                }
            }

            for action in actions.iter() {
                let action_animation = match action {
                    Interact => None,
                    Attack => Some(match direction {
                        North => &manager.attack_up,
                        East => &manager.attack_right,
                        South => &manager.attack_down,
                        West => &manager.attack_left,
                    }),
                    Hit => Some(match direction {
                        North => &manager.hit_up,
                        East => &manager.hit_right,
                        South => &manager.hit_down,
                        West => &manager.hit_left,
                    }),
                    Victory => Some(&manager.victory),
                    Defeat => unimplemented!(), //TODO
                };
                if let Some(action_animation) = action_animation {
                    if animation.has_same_steps(action_animation) {
                        continue;
                    }

                    *animation = action_animation.clone();
                    if !animation.can_interrupt && !animation.should_loop {
                        // If another wait was already there, this will overwrite it
                        waits.insert(entity, Wait::new(animation.len()))
                            .expect("bug: unable to insert wait for animation");
                    }
                }
            }
        }

        // Update the sprites based on the current animation frame
        for (sprite, animation) in (&mut sprites, &mut animations).join() {
            animation.frame_counter += frames_elapsed;

            // This code should work regardless of how many frames have elapsed
            while animation.frame_counter >= animation.steps[animation.current_step].duration {
                // Only loop if the animation is configured that way
                if animation.is_complete() && !animation.should_loop {
                    break;
                }
                // Start at the number of frames that have passed since the end of this step
                animation.frame_counter -= animation.steps[animation.current_step].duration;
                // Completed this frame, move on (and loop if necessary)
                animation.current_step = (animation.current_step + 1) % animation.steps.len();
            }

            // Update the sprite with the current step
            sprite.0 = animation.current_sprite();
        }
    }
}
