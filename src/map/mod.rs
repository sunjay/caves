mod generator;
mod floor_map;

pub use self::generator::*;
pub use self::floor_map::*;

#[derive(Debug, Clone)]
pub struct GameMap {
    key: MapKey,
    levels: Vec<FloorMap>,
}
