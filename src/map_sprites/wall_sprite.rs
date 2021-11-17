use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

/// Used to decouple SpriteImage from a specific SpriteTable
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct WallSprite {
    /// true if there is another wall tile to the north of this one
    pub wall_north: bool,
    /// true if there is another wall tile to the east of this one
    pub wall_east: bool,
    /// true if there is another wall tile to the south of this one
    pub wall_south: bool,
    /// true if there is another wall tile to the west of this one
    pub wall_west: bool,
    /// the variant of the sprite to use
    pub alt: WallSpriteAlternate,
}

/// Different alternate wall styles for some of the wall sprites
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallSpriteAlternate {
    Alt0,
    Alt1,
    Alt2,
    BrickPillar,
    TorchLit,
    EntranceLeft,
    EntranceRight,
}

impl Default for WallSpriteAlternate {
    fn default() -> Self {
        WallSpriteAlternate::Alt0
    }
}

impl Distribution<WallSpriteAlternate> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> WallSpriteAlternate {
        use self::WallSpriteAlternate::*;
        match rng.gen_range(0, 3) {
            0 => Alt0,
            1 => Alt1,
            2 => Alt2,
            _ => unreachable!(),
        }
    }
}
