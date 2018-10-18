use super::MapGenerator;
use map::*;

impl MapGenerator {
    /// Performs certain validation checks on the map
    pub(in super) fn validate_map(&self, map: &FloorMap) {
        let grid = map.grid();
        for pos in grid.tile_positions() {
            if let Some(tile) = grid.get(pos) {
                self.validate_tile(tile);
            }
        }
    }

    fn validate_tile(&self, tile: &Tile) {
        match tile.ttype {
            TileType::Wall(_) | TileType::PassagewayWall => {
                assert!(tile.object.is_none(), "bug: walls should not contain objects");
            },
            _ => {},
        }
    }
}
