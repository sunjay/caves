/// Used to decouple SpriteImage from a specific SpriteTable
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloorSprite {
    Floor1,
    Floor2,
    Floor3,
    Floor4,
    Floor5,
    Floor6,
    Floor7,
    Floor8,
    Floor9,
    Floor10,
    Floor11,
    Floor12,
}

impl Default for FloorSprite {
    fn default() -> Self {
        // Strategy: Fill with a default floor tile and then come back and place patterns after
        FloorSprite::Floor1
    }
}

/// These patterns will be placed in a non-overlapping way throughout the tiles on the map
use self::FloorSprite::*;
pub static FLOOR_PATTERNS: &[&[&[FloorSprite]]] = &[
    &[
        &[Floor1, Floor2, Floor3, Floor1],
        &[Floor5, Floor6, Floor7, Floor8],
        &[Floor9, Floor10, Floor11, Floor12],
        &[Floor1, Floor5, Floor8, Floor1],
    ],
    &[
        &[Floor5, Floor8],
        &[Floor9, Floor12],
        &[Floor5, Floor8],
    ],
    &[
        &[Floor1, Floor5, Floor8, Floor1],
        &[Floor1, Floor2, Floor3, Floor1],
        &[Floor5, Floor6, Floor6, Floor8],
        &[Floor1, Floor9, Floor12, Floor1],
        &[Floor1, Floor5, Floor8, Floor1],
    ],
    &[
        &[Floor5, Floor8],
        &[Floor9, Floor12],
    ],
    &[
        &[Floor5, Floor8],
        &[Floor2, Floor3],
        &[Floor5, Floor8],
    ],
    &[
        &[Floor1, Floor2, Floor3, Floor1, Floor1],
        &[Floor5, Floor6, Floor6, Floor8, Floor1],
        &[Floor9, Floor10, Floor11, Floor12, Floor1],
        &[Floor5, Floor7, Floor6, Floor6, Floor8],
        &[Floor9, Floor12, Floor9, Floor12, Floor1],
        &[Floor1, Floor5, Floor8, Floor1, Floor1],
    ],
    &[
        &[Floor1, Floor9, Floor1, Floor1],
        &[Floor5, Floor6, Floor8, Floor1],
        &[Floor1, Floor2, Floor3, Floor1],
        &[Floor5, Floor7, Floor6, Floor8],
        &[Floor9, Floor10, Floor11, Floor12],
        &[Floor1, Floor5, Floor8, Floor1],
    ],
    &[
        &[Floor1, Floor9, Floor1],
        &[Floor5, Floor7, Floor8],
        &[Floor1, Floor9, Floor1],
    ],
    &[
        &[Floor1, Floor9, Floor1],
        &[Floor5, Floor6, Floor8],
        &[Floor1, Floor9, Floor12],
        &[Floor1, Floor5, Floor8],
    ],
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consistent_floor_patterns() {
        for pat in FLOOR_PATTERNS {
            let rows = pat.len();
            assert!(rows > 0, "floor pattern must be non-empty");

            let row_len = pat[0].len();
            for row in pat.iter() {
                assert_eq!(row.len(), row_len, "rows must all be same size");
            }
        }
    }
}
