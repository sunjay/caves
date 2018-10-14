use std::iter::once;

use sdl2::rect::{Rect, Point};
use specs::{VecStorage, NullStorage, HashMapStorage};

use texture_manager::TextureId;

/// Represents the XY world coordinates of the center of an entity.
///
/// This is distinct from the screen coordinates which are bounded by the size of the display.
///
/// Not to be modified outside of the physics system.
#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Position(pub Point);

/// Represents the direction of movement that a given entity would like to move in
///
/// Used in the physics system to update position every frame
#[derive(Debug, Component)]
#[storage(HashMapStorage)]
pub struct Movement {
    /// The most recent direction that the entity was moving in
    pub direction: MovementDirection,
    /// This is true if the entity should move in the given direction
    pub is_moving: bool,
}

impl Default for Movement {
    fn default() -> Self {
        Self {
            direction: MovementDirection::East,
            is_moving: false,
        }
    }
}

impl Movement {
    pub fn move_in_direction(&mut self, direction: MovementDirection) {
        self.is_moving = true;
        self.direction = direction;
    }
}

/// Represents the direction that an entity would like to move in
///
/// This may not always be possible if there is no way to move further in a given direction (e.g.
/// because of something in the way or a wall or something)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementDirection {
    North,
    South,
    East,
    West,
}

/// An entity that is unable to move until the given duration has elapsed
#[derive(Debug, Default, Component)]
#[storage(HashMapStorage)]
pub struct Wait {
    pub duration: usize, // frames
    pub frames_elapsed: usize, // frames
}

/// Represents the bounding box centered around an entity's position. BoundingBox alone doesn't
/// mean much without a Position also attached to the entity.
///
/// Modifying this after it is initially set is currently NOT supported.
#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct BoundingBox {
    pub width: u32,
    pub height: u32,
}

/// The keyboard controlled player. Only one entity should hold this at a given time.
#[derive(Debug, Default, Component)]
#[storage(NullStorage)]
pub struct KeyboardControlled;

/// The entity with this component and a Position component will be centered in the camera
/// when the scene is rendered.
/// Only one entity should hold this at a given time.
#[derive(Debug, Default, Component)]
#[storage(NullStorage)]
pub struct CameraFocus;

/// Renders a sprite from a surface (spritesheet image).
///
/// The sprite is rendered with the region centered on the entity's Position
///
/// The convention is that the sprite begins pointing to the right and flipping it horizontally
/// results in it facing left
#[derive(Debug, Clone, PartialEq, Eq, Component)]
#[storage(VecStorage)]
pub struct Sprite {
    /// The spritesheet to pull the image from
    pub texture_id: TextureId,
    /// The region of the spritesheet to use, unrelated to the actual bounding box
    pub region: Rect,
    /// Whether to flip the sprite along the horizontal axis
    pub flip_horizontal: bool,
    /// Whether to flip the sprite along the vertical axis
    pub flip_vertical: bool,
}

impl Sprite {
    pub fn update_from_frame(&mut self, frame: &Frame) {
        *self = frame.sprite.clone();
    }
}

/// Used to modify the Sprite component every frame
#[derive(Debug, Clone, Component)]
#[storage(HashMapStorage)]
pub struct Animation {
    pub steps: Vec<Frame>,
    /// The current step of the animation
    pub current_step: usize,
    /// The number of frames that have elapsed during the current step
    pub frame_counter: usize,
    /// Used by the animation manager to avoid interrupting certain animations
    /// Idle and movement animations can be interrupted, but other animations like attack and hit
    /// animations should usually not be interrupted
    pub can_interrupt: bool,
    /// Set to true if the animation should loop once it is complete
    pub should_loop: bool,
}

impl Animation {
    /// Create a new animation from the given steps
    pub fn new(steps: Vec<Frame>, can_interrupt: bool, should_loop: bool) -> Self {
        Animation {
            steps,
            current_step: 0,
            frame_counter: 0,
            can_interrupt,
            should_loop,
        }
    }

    /// Returns true if the animation has reached its last frame
    pub fn is_complete(&self) -> bool {
        self.current_step == self.steps.len() - 1
    }

    /// Returns true if this animation has the same frames as the given animation
    pub fn has_same_steps(&self, other: &Self) -> bool {
        self.steps == other.steps
    }

    /// Only updates the animation if the provided animation has different steps
    pub fn update_if_different(&mut self, other: &Self) {
        if self.has_same_steps(&other) {
            return;
        }
        *self = other.clone();
    }
}

/// Modifies the Animation components every frame based on the current movement of the player or
/// based on events that have occurred (e.g. attacks or gets hit by something)
#[derive(Debug, Component)]
#[storage(HashMapStorage)]
pub struct AnimationManager {
    // Animations for various scenarios
    pub idle: Animation,
    pub victory: Animation,
    pub move_up: Animation,
    pub move_right: Animation,
    pub move_left: Animation,
    pub move_down: Animation,
    pub attack_up: Animation,
    pub attack_right: Animation,
    pub attack_left: Animation,
    pub attack_down: Animation,
    pub hit_up: Animation,
    pub hit_right: Animation,
    pub hit_left: Animation,
    pub hit_down: Animation,
    pub stopped_up: Animation,
    pub stopped_right: Animation,
    pub stopped_left: Animation,
    pub stopped_down: Animation,

    /// The number of frames since this entity last moved, attacked, or been hit
    pub idle_counter: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    /// The sprite that this frame represents
    pub sprite: Sprite,
    /// The duration of this animation step (in frames)
    pub duration: usize,
}

impl AnimationManager {
    /// Returns the standard character animations based on how most of our character spritesheets
    /// are laid out
    pub fn standard_character_animations(fps: usize, texture_id: TextureId) -> Self {
        /// row_i = the index of the row in the spritesheet
        /// pattern = the pattern of frame indexes within the row
        /// durations = the repeating pattern of durations to use for each
        fn animation(
            texture_id: TextureId,
            row_i: i32,
            pattern: impl Iterator<Item=i32>,
            flip_horizontal: bool,
            durations: &[usize],
            can_interrupt: bool,
            should_loop: bool,
        ) -> Animation {
            // The size of each frame box in the spritesheet
            let frame_size = 48;

            let steps = pattern.zip(durations.iter().cycle()).map(|(j, &duration)| Frame {
                sprite: Sprite {
                    texture_id,
                    region: Rect::new(
                        j * frame_size,
                        frame_size * row_i,
                        frame_size as u32,
                        frame_size as u32,
                    ),
                    flip_horizontal,
                    flip_vertical: false,
                },
                duration,
            }).collect();

            Animation::new(steps, can_interrupt, should_loop)
        }

        let ms_to_frames = |ms| ms / (1000 / fps);

        AnimationManager {
            // Animations are configured based on the character animation guide provided with the
            // asset pack

            idle: animation(texture_id, 0, 0..3, false, &[ms_to_frames(640), ms_to_frames(80)], true, true),
            victory: animation(texture_id, 1, 0..3, false, &[ms_to_frames(640), ms_to_frames(80)], true, true),
            move_down: animation(texture_id, 2, 0..4, false, &[ms_to_frames(100)], true, true),
            move_right: animation(texture_id, 3, 0..4, false, &[ms_to_frames(100)], true, true),
            move_left: animation(texture_id, 3, 0..4, true, &[ms_to_frames(100)], true, true),
            move_up: animation(texture_id, 4, 0..4, false, &[ms_to_frames(100)], true, true),
            attack_down: animation(texture_id, 5, 0..4, false,
                &[ms_to_frames(300), ms_to_frames(100), ms_to_frames(100), ms_to_frames(200)],
                false, false),
            attack_right: animation(texture_id, 6, 0..4, false,
                &[ms_to_frames(300), ms_to_frames(100), ms_to_frames(100), ms_to_frames(200)],
                false, false),
            attack_left: animation(texture_id, 6, 0..4, true,
                &[ms_to_frames(300), ms_to_frames(100), ms_to_frames(100), ms_to_frames(200)],
                false, false),
            attack_up: animation(texture_id, 7, 0..4, false,
                &[ms_to_frames(300), ms_to_frames(100), ms_to_frames(100), ms_to_frames(200)],
                false, false),
            hit_down: animation(texture_id, 8, (0..3).chain(once(0)), false, &[ms_to_frames(100)],
                false, false),
            hit_right: animation(texture_id, 9, (0..3).chain(once(0)), false, &[ms_to_frames(100)],
                false, false),
            hit_left: animation(texture_id, 9, (0..3).chain(once(0)), true, &[ms_to_frames(100)],
                false, false),
            hit_up: animation(texture_id, 10, (0..3).chain(once(0)), false, &[ms_to_frames(100)],
                false, false),
            stopped_down: animation(texture_id, 8, 3..4, false, &[ms_to_frames(1)],
                true, false),
            stopped_right: animation(texture_id, 9, 3..4, false, &[ms_to_frames(1)],
                true, false),
            stopped_left: animation(texture_id, 9, 3..4, true, &[ms_to_frames(1)],
                true, false),
            stopped_up: animation(texture_id, 10, 3..4, false, &[ms_to_frames(1)],
                true, false),

            idle_counter: 0,
        }
    }

    /// Returns the default sprite that should be used at the start
    pub fn default_sprite(&self) -> Sprite {
        let stopped = &self.stopped_down.steps[0];
        stopped.sprite.clone()
    }

    /// Returns the default animation that should be used at the start
    pub fn default_animation(&self) -> Animation {
        self.stopped_down.clone()
    }
}
