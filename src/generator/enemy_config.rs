use rand::{Rng, seq::SliceRandom};

use crate::components::{AnimationManager, BoundingBox};

/// The stats + animations for one enemy
#[derive(Clone)]
pub struct EnemyValues {
    pub animations: AnimationManager,
    pub attack: usize, // HP
    pub speed: i32, // movements per second
    pub health_points: usize, // HP
    pub hit_wait: usize, // frames
    pub bounding_box: BoundingBox,
}

/// Each type of enemy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyType {
    Rat,
}

/// Configuration for each type of enemy
#[derive(Clone)]
pub struct EnemyConfig {
    pub rat: EnemyValues,
    /// The choices for enemies to be generated on each level
    /// Array must be the same size as the number of levels
    pub levels: &'static [&'static [EnemyType]],
}

impl EnemyConfig {
    /// Generates a random enemy for the given level
    pub fn random_enemy<R: Rng>(&self, rng: &mut R, level: usize) -> EnemyValues {
        // Levels start at 1
        let types = self.levels.get(level - 1)
            .expect("bug: enemy config must have as many items as levels");
        let enemy_type = *types.choose(rng)
            .expect("bug: every level must have at least one type of enemy that can be generated");
        self.values(enemy_type)
    }

    /// Returns the values for the enemy of the given type
    pub fn values(&self, enemy: EnemyType) -> EnemyValues {
        use self::EnemyType::*;
        match enemy {
            Rat => self.rat.clone(),
        }
    }
}
