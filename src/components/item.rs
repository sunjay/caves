use specs::{Component, HashMapStorage};

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    TreasureKey,
    RoomKey,
    Potion {stength: u32},
}

#[derive(Debug, Clone, PartialEq, Component)]
#[storage(HashMapStorage)]
pub enum Chest {
    Item(Item),
    Opened,
}
