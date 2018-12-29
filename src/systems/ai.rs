use specs::{System, Join, ReadExpect, ReadStorage, WriteStorage, Entities};

use crate::components::{Movement, BoundingBox, Position, Player, Enemy, Wait};
use crate::map::FloorMap;

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

        for (entity, movement, _, ()) in (&entities, &mut movements, &enemies, !&waits).join() {
            //TODO: Do not move if Wait is applied
        }
    }
}
