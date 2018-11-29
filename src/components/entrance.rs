use specs::{Component, NullStorage, HashMapStorage};

/// A door between two rooms
#[derive(Debug, Default, Component)]
#[storage(HashMapStorage)]
pub struct Door {
    orientation: HoriVert
}

/// A gate between two rooms
#[derive(Debug, Default, Component)]
#[storage(HashMapStorage)]
pub struct Gate {
    orientation: HoriVert
}

/// A locked door/gate
#[derive(Debug, Default, Component)]
#[storage(NullStorage)]
pub struct Locked;

/// Represents the orientation of something that can be either horizontal or vertical
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HoriVert {
    Horizontal,
    Vertical,
}
