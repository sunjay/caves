use sdl2::rect::{Point, Rect};

#[derive(Debug, Clone)]
pub struct Map {
    level_start: Point,
    level_boundary: Rect,
}

impl Map {
    pub fn generate() -> Self {
        Self {
            level_start: Point::new(50, 50),
            level_boundary: Rect::new(0, 0, 100, 100),
        }
    }

    pub fn level_start(&self) -> Point {
        self.level_start
    }

    pub fn level_boundary(&self) -> Rect {
        self.level_boundary
    }
}
