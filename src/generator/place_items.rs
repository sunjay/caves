use rand::{rngs::StdRng, seq::SliceRandom};
use specs::{World, Builder, ReadStorage, Join};

use super::{GameGenerator, RanOutOfAttempts};
use super::world_helpers::world_contains_any_entity;
use crate::map::TilePos;
use crate::map_sprites::WallSprite;
use crate::components::{Position, Ghost, BoundingBox, Sprite, Stairs};
use crate::map::*;

fn validate_chosen_staircase(grid: &TileGrid, world: &World, pos: TilePos, tile_size: u32) -> bool {
    // The staircase cannot be directly beside another staircase. It also cannot be beside
    // a tile that is beside an entrance or else that entrance will get blocked by a wall
    // in surround_stairways

    let has_staircase = |pos: TilePos| {
        let bounds = pos.tile_rect(tile_size);
        let (positions, stairs) = world.system_data::<(ReadStorage<'_, Position>, ReadStorage<'_, Stairs>)>();
        (&positions, &stairs).join()
            .any(|(&Position(pos), _)| bounds.contains_point(pos))
    };

    let mut open_sides = 0;
    for adj in grid.adjacent_positions(pos) {
        // It must be possible to enter into the stairs from one side or the other.
        // Taking advantage of the fact that all stairways are on vertical edges of rooms
        if adj.row == pos.row && grid.get(adj).is_floor() {
            open_sides += 1;
        }

        if has_staircase(adj) {
            return false;
        }

        if grid.adjacent_positions(adj).any(|adj2| grid.is_room_entrance(adj2)) {
            return false;
        }
    }

    open_sides == 1
}

impl<'a> GameGenerator<'a> {
    pub(in super) fn place_to_next_level_tiles(
        &self,
        rng: &mut StdRng,
        map: &mut FloorMap,
        world: &mut World,
    ) -> Result<(), RanOutOfAttempts> {
        let valid_rooms = |(_, r): &(RoomId, &Room)| r.can_contain_to_next_level();
        // Can only place on vertical edge since we only have sprites for tiles adjacent to those
        let next_pos = |rng: &mut StdRng, rect: TileRect| rect.random_right_vertical_edge_tile(rng);

        let place_object = |world: &mut World, map: &mut FloorMap, obj_pos, wall_pos, id| {
            self.place_stairs(world, map, obj_pos, wall_pos, Stairs::ToNextLevel {id});
            self.surround_stairways(obj_pos, map);
        };
        self.place_object_in_rooms(rng, map, world, valid_rooms, self.next_prev_tiles,
            next_pos, validate_chosen_staircase, place_object)?;
        Ok(())
    }

    pub(in super) fn place_to_prev_level_tiles(
        &self,
        rng: &mut StdRng,
        map: &mut FloorMap,
        world: &mut World,
    ) -> Result<(), RanOutOfAttempts> {
        let valid_rooms = |(_, r): &(RoomId, &Room)| r.can_contain_to_prev_level();
        // Can only place on vertical edge since we only have sprites for tiles adjacent to those
        let next_pos = |rng: &mut StdRng, rect: TileRect| rect.random_left_vertical_edge_tile(rng);

        let place_object = |world: &mut World, map: &mut FloorMap, obj_pos, wall_pos, id| {
            self.place_stairs(world, map, obj_pos, wall_pos, Stairs::ToPrevLevel {id});
            self.surround_stairways(obj_pos, map);
        };
        self.place_object_in_rooms(rng, map, world, valid_rooms, self.next_prev_tiles,
            next_pos, validate_chosen_staircase, place_object)?;
        Ok(())
    }

    fn place_stairs(
        &self,
        world: &mut World,
        map: &FloorMap,
        obj_pos: TilePos,
        wall_pos: TilePos,
        stairs: Stairs,
    ) {
        let pos = obj_pos.center(map.tile_size() as i32);
        // Want to face away from the wall
        let stairs_entrance_to_right = match wall_pos.difference(obj_pos) {
            (0, 0) => unreachable!("bug: a position cannot face itself"),
            (0, a) => a < 0,
            _ => unreachable!("bug: stairs only support facing left or right"),
        };
        let sprite = match stairs {
            Stairs::ToNextLevel {..} if stairs_entrance_to_right => self.sprites.staircase_down_right(),
            Stairs::ToNextLevel {..} => self.sprites.staircase_down_left(),
            Stairs::ToPrevLevel {..} if stairs_entrance_to_right => self.sprites.staircase_up_right(),
            Stairs::ToPrevLevel {..} => self.sprites.staircase_up_left(),
        };
        // Make the stairs a little bit smaller so the player really needs to walk on top to enter
        let stair_size = self.tile_size / 2;
        world.create_entity()
            .with(Ghost) // Allow the player to walk on top of stairs
            .with(Position(pos))
            .with(BoundingBox::Full {width: stair_size, height: stair_size})
            .with(stairs)
            .with(Sprite(sprite))
            .build();
    }

    /// Ensures that there is a wall on each side of a staircase
    fn surround_stairways(&self, pos: TilePos, map: &mut FloorMap) {
        let grid = map.grid_mut();
        for adj in grid.adjacent_positions(pos) {
            // Taking advantage of the fact that all stairways are on vertical edges of rooms
            if adj.col == pos.col && !grid.get(adj).is_wall() {
                grid.get_mut(adj).become_wall(WallSprite::default());
            }
        }
    }

    /// Places `nrooms` copies of a TileObject into `nrooms` randomly choosen rooms from rooms
    fn place_object_in_rooms(
        &self,
        rng: &mut StdRng,
        map: &mut FloorMap,
        world: &mut World,
        room_filter: impl FnMut(&(RoomId, &Room)) -> bool,
        nrooms: usize,
        mut next_pos: impl FnMut(&mut StdRng, TileRect) -> TilePos,
        mut extra_validation: impl FnMut(&TileGrid, &World, TilePos, u32) -> bool,
        mut place_object: impl FnMut(&mut World, &mut FloorMap, TilePos, TilePos, usize),
    ) -> Result<(), RanOutOfAttempts> {
        // To do this using choose we would need to allocate anyway, so we might as well just use
        // shuffle to do all the random choosing at once
        let mut rooms: Vec<_> = map.rooms()
            .filter(room_filter)
            .map(|(id, r)| (id, *r.boundary()))
            .collect();
        assert!(rooms.len() >= nrooms, "Not enough rooms to place items");
        rooms.shuffle(rng);

        let tile_size = map.tile_size();

        // This cycles through all the rooms up until we have gone through `self.attempts` rooms.
        // We do one attempt per room. Trying to do all of the attempts on every room doesn't make
        // sense since the number of allowed attempts is way more tiles than that room has anyway.
        // Some rooms simply do not have a place to put the object, so there is no point wasting
        // cycles trying to find a place. This method efficiently moves on to the next room as soon
        // as a single attempt fails, ensuring that the search will either make progress or fail
        // as soon as we reach the attempt limit.
        let mut attempts = 0;

        let mut placed = 0;
        for (room_id, rect) in rooms.into_iter().cycle() {
            // Found enough places
            if placed >= nrooms {
                break;
            }

            if attempts >= self.attempts {
                return Err(RanOutOfAttempts);
            }
            attempts += 1;

            // Pick a random point on one of the edges of the room
            let pos = next_pos(rng, rect);

            if !map.grid().get(pos).is_wall() {
                // Can happen since rooms overlap
                continue;
            }

            // Cannot place adjacent to corner since corners are only adjacent to other wall
            // tiles and to other rooms
            if rect.is_corner(pos) {
                continue;
            }

            let inner_room_tile = self.find_place(map.grid(), world, pos, tile_size, room_id);
            if let Some(inner_room_tile) = inner_room_tile {
                if !extra_validation(map.grid(), world, inner_room_tile, tile_size) {
                    continue;
                }

                place_object(world, map, inner_room_tile, pos, placed);
                placed += 1;
            }
        }

        debug_assert_eq!(placed, nrooms);
        Ok(())
    }

    /// Attempts to find a room tile adjacent to the given tile that we can place the object in
    fn find_place(&self, grid: &TileGrid, world: &World, pos: TilePos, tile_size: u32, room_id: RoomId) -> Option<TilePos> {
        let tile = grid.get(pos);

        // Must be a wall with a single room tile of the given room_id adjacent to it. The
        // adjacent room tile will be where the object will go. This check also ensures
        // that no item gets placed beside a doorway.
        if !tile.is_wall() {
            return None;
        }
        let adj_room_tiles: Vec<_> = grid.adjacent_positions(pos)
            .filter(|&pt| grid.get(pt).is_room_floor(room_id))
            .collect();
        let inner_room_tile = match &adj_room_tiles[..] {
            [adj] => *adj,
            // Either a wall with an entrance next to it or a wall next to another room.
            _ => return None,
        };

        assert!(grid.get(inner_room_tile).is_room_floor(room_id),
            "bug: can only place items within rooms on room tiles");

        // Cannot place on a tile that already has an item
        let inner_room_tile_bounds = inner_room_tile.tile_rect(tile_size);
        if world_contains_any_entity(world, inner_room_tile_bounds) {
            return None;
        }

        // Make sure the tile we have chosen isn't surrounded by any room entrances
        // This can still happen even with all of the other checks if we choose x in the
        // picture below:
        //
        //     ooooooxo
        //     o
        //     o      o
        //     oooooooo
        //
        // x would pass all of the previous checks but get caught by this one
        if grid.adjacent_positions(inner_room_tile).any(|&pt| grid.is_room_entrance(pt)) {
            return None;
        }

        Some(inner_room_tile)
    }
}
