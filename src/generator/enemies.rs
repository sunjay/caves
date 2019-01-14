use std::collections::HashSet;

use rand::{rngs::StdRng};
use specs::{World, Builder};

use super::{GameGenerator, RanOutOfAttempts, EnemyValues};
use crate::components::{Position, Sprite, Enemy, HealthPoints, Attack, HitWait, Movement};
use crate::map::*;

impl<'a> GameGenerator<'a> {
    pub(in super) fn add_enemies(&self,
        rng: &mut StdRng,
        map: &FloorMap,
        world: &mut World,
        level: usize,
    ) -> Result<(), RanOutOfAttempts> {
        let grid = map.grid();
        for (room_id, room) in map.rooms() {
            if !room.can_generate_enemies() {
                continue;
            }

            let room_area = map.room_exact_area(room_id);
            let max_enemies = (room_area as f64 * self.max_room_enemy_area) as usize;
            let nenemies = self.room_enemies.gen(rng).min(max_enemies);

            let room_bounds = room.boundary();
            let mut placed = HashSet::new();

            let mut attempts = 0;
            while placed.len() < nenemies {
                if attempts > self.attempts {
                    return Err(RanOutOfAttempts);
                }
                attempts += 1;

                // Goal: Don't generate enemies near the walls (so those spaces are free for other things)
                let pos = room_bounds.random_inner_tile(rng);
                // Tile where an enemy has already been generated
                if placed.contains(&pos) {
                    continue;
                }
                // Not a tile in the right room
                if !grid.get(pos).is_room_floor(room_id) {
                    continue;
                }
                // Though we got an "inner" tile, we may still be near a wall or entrance
                if grid.adjacent_positions(pos).any(|pt| grid.get(pt).is_wall() || grid.is_room_entrance(pt)) {
                    continue;
                }

                let enemy_pos = pos.center(self.tile_size as i32);

                let EnemyValues {
                    behaviour,
                    animations,
                    attack,
                    speed,
                    health_points,
                    hit_wait,
                    bounding_box,
                } = self.enemy_config.random_enemy(rng, level);

                world.create_entity()
                    .with(Enemy {behaviour, speed})
                    .with(HealthPoints(health_points))
                    .with(Attack(attack))
                    .with(HitWait(hit_wait))
                    .with(Position(enemy_pos))
                    .with(bounding_box)
                    .with(Movement::default())
                    .with(Sprite(animations.default_sprite()))
                    .with(animations.default_animation())
                    .with(animations)
                    .build();

                placed.insert(pos);
            }
        }

        Ok(())
    }
}
