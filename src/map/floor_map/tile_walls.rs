use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wall {
    Open,
    Closed,
    Locked,
}

#[derive(Debug, Clone)]
pub struct TileWalls {
    pub north: Wall,
    pub east: Wall,
    pub south: Wall,
    pub west: Wall,
}

impl Default for TileWalls {
    fn default() -> Self {
        use self::Wall::*;

        Self {
            north: Closed,
            east: Closed,
            south: Closed,
            west: Closed,
        }
    }
}

impl fmt::Display for TileWalls {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Wall::*;
        write!(f, "{}", match *self {
            TileWalls {north: Closed, east: Closed, south: Closed, west: Closed} => {
                "\u{26ac}" // all closed
            },
            TileWalls {north: Open, east: Closed, south: Closed, west: Closed} => {
                "\u{257d}" // N
            },
            TileWalls {north: Closed, east: Open, south: Closed, west: Closed} => {
                "\u{257e}" // E
            },
            TileWalls {north: Closed, east: Closed, south: Open, west: Closed} => {
                "\u{257f}" // S
            },
            TileWalls {north: Closed, east: Closed, south: Closed, west: Open} => {
                "\u{257c}" // W
            },
            TileWalls {north: Open, east: Open, south: Closed, west: Closed} => {
                "\u{2514}" // NE
            },
            TileWalls {north: Closed, east: Open, south: Open, west: Closed} => {
                "\u{250C}" // SE
            },
            TileWalls {north: Closed, east: Closed, south: Open, west: Open} => {
                "\u{2510}" // SW
            },
            TileWalls {north: Open, east: Closed, south: Closed, west: Open} => {
                "\u{2518}" // NW
            },
            TileWalls {north: Open, east: Closed, south: Open, west: Closed} => {
                "\u{2502}" // NS
            },
            TileWalls {north: Closed, east: Open, south: Closed, west: Open} => {
                "\u{2500}" // EW
            },
            TileWalls {north: Open, east: Open, south: Open, west: Closed} => {
                "\u{251c}" // NES
            },
            TileWalls {north: Closed, east: Open, south: Open, west: Open} => {
                "\u{252c}" // ESW
            },
            TileWalls {north: Open, east: Closed, south: Open, west: Open} => {
                "\u{2524}" // NSW
            },
            TileWalls {north: Open, east: Open, south: Closed, west: Open} => {
                "\u{2534}" // NEW
            },
            TileWalls {north: Open, east: Open, south: Open, west: Open} => {
                "\u{253c}" // NESW
            },
            _ => " ",
        })
    }
}

impl TileWalls {
    /// Returns true if only a single wall is open
    pub fn is_dead_end(&self) -> bool {
        use self::Wall::*;
        match *self {
            TileWalls {north: Open, east: Closed, south: Closed, west: Closed} |
            TileWalls {north: Closed, east: Open, south: Closed, west: Closed} |
            TileWalls {north: Closed, east: Closed, south: Open, west: Closed} |
            TileWalls {north: Closed, east: Closed, south: Closed, west: Open} => {
                true
            },
            _ => false,
        }
    }
}
