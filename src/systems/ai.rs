use rand::{Rng, thread_rng};
use specs::{System, Join, ReadExpect, ReadStorage, WriteStorage, Entities};

use crate::components::{Movement, BoundingBox, Position, Player, Enemy, EnemyBehaviour, Wait};
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

        let mut rng = thread_rng();

        for (entity, enemy, movement, ()) in (&entities, &enemies, &mut movements, !&waits).join() {
            match enemy.behaviour {
                EnemyBehaviour::Random => {
                    // favor keeping the movement direction the same
                    if rng.gen_range(0, 10) == 0 {
                        movement.direction = rng.gen();
                    }
                    movement.speed = enemy.speed;
                }
            }
        }
    }
}
