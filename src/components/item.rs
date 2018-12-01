use specs::{Component, HashMapStorage};

use map_sprites::TorchAnimation;

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

#[derive(Debug, Clone, Component)]
#[storage(HashMapStorage)]
pub enum WallDecoration {
    Torch(TorchAnimation),
    //TODO: arrow shooter, portal, spikes, etc.
}

impl WallDecoration {
    pub fn torch() -> Self {
        WallDecoration::Torch(TorchAnimation::default())
    }
}
