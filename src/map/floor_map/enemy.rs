use components as c;

/// Components that represent parts of an enemy entity
///
/// Used to cache enemies between levels
#[derive(Clone)]
pub struct EnemyState {
    pub position: c::Position,
    pub health: c::HealthPoints,
    pub bounding_box: c::BoundingBox,
    pub movement: c::Movement,
    pub sprite: c::Sprite,
    pub animation: c::Animation,
    pub animation_manager: c::AnimationManager,
}
