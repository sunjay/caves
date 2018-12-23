use rand::{rngs::StdRng, Rng, seq::SliceRandom};
use specs::{World, Builder};

use super::{GameGenerator, RanOutOfAttempts};
use sprites::WallSprite;
use components::{Position, Stairs, StairsDirection};
use map::*;

fn validate_chosen_staircase(grid: &TileGrid, pos: TilePos) -> bool {
    // The staircase cannot be directly beside another staircase. It also cannot be beside
    // a tile that is beside an entrance or else that entrance will get blocked by a wall
    // in surround_stairways

    let mut open_sides = 0;
    for adj in grid.adjacent_positions(pos) {
        // It must be possible to enter into the stairs from one side or the other.
        // Taking advantage of the fact that all stairways are on vertical edges of rooms
        if adj.row == pos.row && grid.get(adj).is_traversable() {
            open_sides += 1;
        }

        if grid.get(adj).has_staircase() {
            return false;
        }
        if grid.adjacent_positions(adj).any(|adj2| grid.is_room_entrance(adj2)) {
            return false;
        }
    }

    open_sides == 1
}

impl GameGenerator {
    pub(in super) fn place_to_next_level_tiles(
        &self,
        rng: &mut StdRng,
        map: &FloorMap,
        world: &mut World,
    ) -> Result<(), RanOutOfAttempts> {
        let valid_rooms = |(_, r): &(RoomId, &Room)| r.can_contain_to_next_level();
        // Can only place on vertical edge since we only have sprites for tiles adjacent to those
        let next_pos = |rng: &mut StdRng, rect: TileRect| rect.random_right_vertical_edge_tile(rng);

        let object = |map: &FloorMap, id, obj_pos, wall_pos| {
            let pos = map.tile_center(obj_pos);
            let direction = StairsDirection::towards_target(wall_pos, obj_pos);
            world.create_entity()
                .with(Position(pos))
                .with(Stairs::ToNextLevel {id, direction})
                .build();
        };
        let placed = self.place_object_in_rooms(rng, map, valid_rooms, self.next_prev_tiles,
            next_pos, validate_chosen_staircase, object)?;
        self.surround_stairways(&placed, map);
        Ok(())
    }

    pub(in super) fn place_to_prev_level_tiles(
        &self,
        rng: &mut StdRng,
        map: &FloorMap,
        world: &mut World,
    ) -> Result<(), RanOutOfAttempts> {
        let valid_rooms = |(_, r): &(RoomId, &Room)| r.can_contain_to_prev_level();
        // Can only place on vertical edge since we only have sprites for tiles adjacent to those
        let next_pos = |rng: &mut StdRng, rect: TileRect| rect.random_left_vertical_edge_tile(rng);

        let object = |map: &FloorMap, id, obj_pos, wall_pos| {
            let pos = map.tile_center(obj_pos);
            let direction = StairsDirection::towards_target(wall_pos, obj_pos);
            world.create_entity()
                .with(Position(pos))
                .with(Stairs::ToPrevLevel {id, direction})
                .build();
        };
        let placed = self.place_object_in_rooms(rng, map, valid_rooms, self.next_prev_tiles,
            next_pos, validate_chosen_staircase, object)?;
        self.surround_stairways(&placed, map);
        Ok(())
    }

    /// Ensures that there is a wall on each side of a staircase
    fn surround_stairways(&self, staircases: &[TilePos], map: &FloorMap) {
        let grid = map.grid_mut();
        for &stairs in staircases {
            for adj in grid.adjacent_positions(stairs) {
                // Taking advantage of the fact that all stairways are on vertical edges of rooms
                if adj.col == stairs.col && !grid.get(adj).is_wall() {
                    grid.get_mut(adj).become_wall(WallSprite::default());
                }
            }
        }
    }

    /// Places `nrooms` copies of a TileObject into `nrooms` randomly choosen rooms from rooms
    fn place_object_in_rooms(
        &self,
        rng: &mut StdRng,
        map: &FloorMap,
        room_filter: impl FnMut(&(RoomId, &Room)) -> bool,
        nrooms: usize,
        mut next_pos: impl FnMut(&mut StdRng, TileRect) -> TilePos,
        mut extra_validation: impl FnMut(&TileGrid, TilePos) -> bool,
        mut place_object: impl FnMut(&FloorMap, usize, TilePos, TilePos),
    ) -> Result<Vec<TilePos>, RanOutOfAttempts> {
        // To do this using choose we would need to allocate anyway, so we might as well just use
        // shuffle to do all the random choosing at once
        let mut rooms: Vec<_> = map.rooms()
            .filter(room_filter)
            .map(|(id, r)| (id, *r.boundary()))
            .collect();
        assert!(rooms.len() >= nrooms, "Not enough rooms to place items");
        rooms.shuffle(&mut rng);

        let grid = map.grid_mut();

        // This cycles through all the rooms up until we have gone through `self.attempts` rooms.
        // We do one attempt per room. Trying to do all of the attempts on every room doesn't make
        // sense since the number of allowed attempts is way more tiles than that room has anyway.
        // Some rooms simply do not have a place to put the object, so there is no point wasting
        // cycles trying to find a place. This method efficiently moves on to the next room as soon
        // as a single attempt fails, ensuring that the search will either make progress or fail
        // as soon as we reach the attempt limit.
        let mut attempts = 0;
        let mut placed = Vec::new();
        for (room_id, rect) in rooms.into_iter().cycle() {
            // Found enough places
            if placed.len() >= nrooms {
                break;
            }

            if attempts >= self.attempts {
                return Err(RanOutOfAttempts);
            }
            attempts += 1;

            // Pick a random point on one of the edges of the room
            let pos = next_pos(rng, rect);

            if !grid.get(pos).is_wall() {
                // Can happen since rooms overlap
                continue;
            }

            // Cannot place adjacent to corner since corners are only adjacent to other wall
            // tiles and to other rooms
            if rect.is_corner(pos) {
                continue;
            }

            if let Some(inner_room_tile) = self.find_place(grid, pos, room_id) {
                if !extra_validation(grid, inner_room_tile) {
                    continue;
                }

                let tile = grid.get_mut(inner_room_tile);

                // Want to face away from the wall
                tile.place_object(place_object(map, placed.len(), inner_room_tile, pos));

                placed.push(inner_room_tile);
            }
        }

        debug_assert_eq!(placed.len(), nrooms);
        Ok(placed)
    }

    /// Attempts to find a room tile adjacent to the given tile that we can place the object in
    fn find_place(&self, grid: &TileGrid, pos: TilePos, room_id: RoomId) -> Option<TilePos> {
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
        if grid.get(inner_room_tile).has_object() {
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
        if grid.adjacent_positions(inner_room_tile).find(|&pt| grid.is_room_entrance(pt)).is_some() {
            return None;
        }

        Some(inner_room_tile)
    }
}
