use specs::{Component, NullStorage};

/// A door between two rooms
#[derive(Debug, Default, Component)]
#[storage(NullStorage)]
pub struct Door;

/// A gate between two rooms
#[derive(Debug, Default, Component)]
#[storage(NullStorage)]
pub struct Gate;

/// A locked door/gate
#[derive(Debug, Default, Component)]
#[storage(NullStorage)]
pub struct Locked;
