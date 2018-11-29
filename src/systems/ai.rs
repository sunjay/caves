use specs::{System, Join, ReadExpect, ReadStorage, WriteStorage, Entities};

use components::{Movement, BoundingBox, Position, Player, Enemy, Wait};
use map::FloorMap;

#[derive(SystemData)]
pub struct AIData<'a> {
    entities: Entities<'a>,
    map: ReadExpect<'a, FloorMap>,
    movements: WriteStorage<'a, Movement>,
    bounding_boxes: ReadStorage<'a, BoundingBox>,
    positions: ReadStorage<'a, Position>,
    players: ReadStorage<'a, Player>,
    enemies: ReadStorage<'a, Enemy>,
    waits: ReadStorage<'a, Wait>,
}

pub struct AI;

impl<'a> System<'a> for AI {
    type SystemData = AIData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let AIData {
            entities,
            map,
            mut movements,
            bounding_boxes,
            positions,
            players,
            enemies,
            waits,
        } = data;
        let level = map.current_level_map();

        for (entity, movement, _, ()) in (&entities, &mut movements, &enemies, !&waits).join() {
        }
    }
}
