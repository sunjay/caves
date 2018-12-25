use specs::{World, ReadStorage, Join};
use sdl2::rect::Rect;

use crate::components::Position;

//TODO: These functions are just utility methods. Maybe it would be better to wrap World in
// a struct and provide these methods on it directly.

/// Returns true if the given boundary contains any entity
pub(in super) fn world_contains_any_entity(world: &World, bounds: Rect) -> bool {
    world.system_data::<ReadStorage<Position>>().join()
        .any(|&Position(pos)| bounds.contains_point(pos))
}
