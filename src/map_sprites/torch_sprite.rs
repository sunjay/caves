use rand::{
    thread_rng,
    Rng,
    distributions::{
        Distribution,
        Standard,
    },
};

/// Each step of the (lit) torch animation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TorchSprite {
    Torch1,
    Torch2,
    Torch3,
    Torch4,
}

impl Distribution<TorchSprite> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> TorchSprite {
        use self::TorchSprite::*;
        match rng.gen_range(0, 4) {
            0 => Torch1,
            1 => Torch2,
            2 => Torch3,
            3 => Torch4,
            _ => unreachable!(),
        }
    }
}


impl TorchSprite {
    /// Returns the next step in the animation sequence
    pub fn next(self) -> Self {
        use self::TorchSprite::*;
        match self {
            Torch1 => Torch2,
            Torch2 => Torch3,
            Torch3 => Torch4,
            Torch4 => Torch1,
        }
    }
}

/// Manages the state of the torch animation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TorchAnimation {
    current_step: TorchSprite,
    frame_counter: usize,
}

impl Default for TorchAnimation {
    fn default() -> Self {
        Self {
            // Able to use the thread rng here because this does NOT need to be deterministic
            current_step: thread_rng().gen(),
            frame_counter: 0,
        }
    }
}

impl TorchAnimation {
    pub fn current_step(&self) -> TorchSprite {
        self.current_step
    }

    pub fn advance(&mut self, frames: usize) {
        self.frame_counter += frames;

        let frames_per_step = 3;

        let steps_elapsed = self.frame_counter / frames_per_step;
        for _ in 0..steps_elapsed {
            self.current_step = self.current_step.next();
        }

        // Leftover frames
        self.frame_counter %= frames_per_step;
    }
}
