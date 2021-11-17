use std::ops::Add;

use rand::{
    distributions::{uniform::SampleUniform, Distribution, Standard},
    Rng,
};

/// Represents the minimum and maximum boundary for a given type
/// Both boundaries are inclusive
#[derive(Debug, Clone)]
pub struct Bounds<T> {
    pub min: T,
    pub max: T,
}

impl<T: PartialOrd + SampleUniform + Copy> Bounds<T> {
    pub fn gen<R: Rng>(&self, rng: &mut R) -> T
    where
        Standard: Distribution<T>,
        T: Add<Output = T> + From<u8>,
    {
        // Need to add 1 for this to be an inclusive range. These fancy type bounds allow for that.
        // From<u8> was chosen because a lot of types support From<u8>.
        rng.gen_range(self.min, self.max + 1.into())
    }
}

impl<T> From<(T, T)> for Bounds<T> {
    fn from((min, max): (T, T)) -> Self {
        Bounds { min, max }
    }
}
