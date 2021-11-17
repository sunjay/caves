use sdl2::rect::Rect;
use specs::{Join, ReadStorage, World};

use crate::components::Position;

//TODO: These functions are just utility methods. Maybe it would be better to wrap World in
// a struct and provide these methods on it directly.

/// Returns true if the given boundary contains any entity
pub(super) fn world_contains_any_entity(world: &World, bounds: Rect) -> bool {
    world
        .system_data::<ReadStorage<'_, Position>>()
        .join()
        .any(|&Position(pos)| bounds.contains_point(pos))
}
