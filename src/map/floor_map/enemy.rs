use components as c;

/// Components that represent parts of an enemy entity
#[derive(Clone)]
pub struct EnemyState {
    position: c::Position,
    bounding_box: c::BoundingBox,
    movement: c::Movement,
    sprite: c::Sprite,
    animation: c::Animation,
    animation_manager: c::AnimationManager,
}
