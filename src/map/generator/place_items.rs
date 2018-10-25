use rand::{StdRng, Rng};

use super::{MapGenerator, RanOutOfAttempts};
use map::*;

impl MapGenerator {
    pub(in super) fn place_to_next_level_tiles(
        &self,
        rng: &mut StdRng,
        map: &mut FloorMap,
    ) -> Result<(), RanOutOfAttempts> {
        let valid_rooms = |(_, r): &(RoomId, &Room)| r.can_contain_to_next_level();
        self.place_object_in_rooms_edges(rng, map, valid_rooms, self.next_prev_tiles, TileObject::ToNextLevel)
    }

    pub(in super) fn place_to_prev_level_tiles(
        &self,
        rng: &mut StdRng,
        map: &mut FloorMap,
    ) -> Result<(), RanOutOfAttempts> {
        let valid_rooms = |(_, r): &(RoomId, &Room)| r.can_contain_to_prev_level();
        self.place_object_in_rooms_edges(rng, map, valid_rooms, self.next_prev_tiles, TileObject::ToPrevLevel)
    }

    /// Places `nrooms` copies of a TileObject into `nrooms` randomly choosen rooms from rooms
    fn place_object_in_rooms_edges<'a>(
        &self,
        rng: &mut StdRng,
        map: &mut FloorMap,
        room_filter: impl FnMut(&(RoomId, &Room)) -> bool,
        nrooms: usize,
        mut object: impl FnMut(usize) -> TileObject
    ) -> Result<(), RanOutOfAttempts> {
        // To do this using choose we would need to allocate anyway, so we might as well just use
        // shuffle to do all the random choosing at once
        let mut rooms: Vec<_> = map.rooms()
            .filter(room_filter)
            .map(|(id, r)| (id, *r.boundary()))
            .collect();
        assert!(rooms.len() >= nrooms,
            "Not enough rooms to place next/prev level tiles");
        rng.shuffle(&mut rooms);

        let grid = map.grid_mut();

        'place_loop: for (i, (room_id, rect)) in rooms.into_iter().take(nrooms).enumerate() {
            for _ in 0..self.attempts {
                // Pick a random point on one of the edges of the room
                let pos = if rng.gen() {
                    rect.random_horizontal_edge_tile(rng)
                } else {
                    rect.random_vertical_edge_tile(rng)
                };

                // Cannot place adjacent to corner
                if rect.is_corner(pos) {
                    continue;
                }

                if let Some(inner_room_tile) = self.find_place(grid, pos, room_id) {
                    let tile = grid.get_mut(inner_room_tile);
                    tile.place_object(object(i));
                    // Can not simply break because then we would return RanOutOfAttempts
                    continue 'place_loop;
                }
            }

            return Err(RanOutOfAttempts);
        }

        Ok(())
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
            // A corner without an entrance next to it
            [] => unreachable!("bug: earlier check should have detected corners"),
            [adj] => *adj,
            // A wall with an entrance adjacent to it. Technically this is fine for item
            // placement. A future enhancement would be to filter out the entrance and just
            // use the other adjacent that isn't an entrance (TODO).
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
