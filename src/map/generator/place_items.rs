use rand::{StdRng, Rng};

use super::{MapGenerator, RanOutOfAttempts};
use map::*;

impl MapGenerator {
    pub(in super) fn place_to_next_level_tiles(
        &self,
        rng: &mut StdRng,
        map: &mut FloorMap,
        rooms: &[(RoomId, Room)],
    ) -> Result<(), RanOutOfAttempts> {
        let valid_rooms = rooms.iter().filter(|(_, r)| r.can_contain_to_next_level()).cloned();
        self.place_object_in_rooms_edges(rng, map, valid_rooms, self.next_prev_tiles, TileObject::ToNextLevel)
    }

    pub(in super) fn place_to_prev_level_tiles(
        &self,
        rng: &mut StdRng,
        map: &mut FloorMap,
        rooms: &[(RoomId, Room)],
    ) -> Result<(), RanOutOfAttempts> {
        let valid_rooms = rooms.iter().filter(|(_, r)| r.can_contain_to_prev_level()).cloned();
        self.place_object_in_rooms_edges(rng, map, valid_rooms, self.next_prev_tiles, TileObject::ToPrevLevel)
    }

    /// Places `nrooms` copies of a TileObject into `nrooms` randomly choosen rooms from rooms
    fn place_object_in_rooms_edges<OB>(
        &self,
        rng: &mut StdRng,
        map: &mut FloorMap,
        rooms: impl Iterator<Item=(RoomId, Room)>,
        nrooms: usize,
        mut object: OB
    ) -> Result<(), RanOutOfAttempts>
        where OB: FnMut(usize) -> TileObject {

        // To do this using choose we would need to allocate anyway, so we might as well just use
        // shuffle to do all the random choosing at once
        let mut rooms: Vec<_> = rooms.collect();
        assert!(rooms.len() >= nrooms,
            "Not enough rooms to place next/prev level tiles");
        rng.shuffle(&mut rooms);

        let grid = map.grid_mut();

        'place_loop: for (i, (room_id, room)) in rooms.into_iter().take(nrooms).enumerate() {
            for _ in 0..self.attempts {
                // Pick a random point on one of the edges of the room
                let pos = if rng.gen() {
                    room.random_horizontal_edge_tile(rng)
                } else {
                    room.random_vertical_edge_tile(rng)
                };

                // Cannot place adjacent to corner
                if room.is_corner(pos) {
                    continue;
                }

                // Must be a room wall with a single room tile adjacent to it. The adjacent room
                // tile will be where the object will go. This check also ensures that no item
                // gets placed beside a doorway.
                if !grid.is_room_wall(pos, room_id) {
                    continue;
                }
                let adj_room_tiles: Vec<_> = grid.adjacent_positions(pos)
                    .filter(|&pt| grid.is_room(pt, room_id))
                    .collect();
                let inner_room_tile = match &adj_room_tiles[..] {
                    // A corner without an entrance next to it
                    [] => unreachable!("bug: earlier check should have detected corners"),
                    [adj] => *adj,
                    // A wall with an entrance adjacent to it. Technically this is fine for item
                    // placement. A future enhancement would be to filter out the entrance and just
                    // use the other adjacent that isn't an entrance (TODO).
                    _ => continue,
                };

                assert!(grid.is_room(inner_room_tile, room_id),
                    "bug: can only place items within rooms on room tiles");

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
                    continue;
                }

                let tile = grid.get_mut(inner_room_tile).expect("bug: expected a tile");
                if tile.has_object() {
                    continue;
                }

                tile.place_object(object(i));
                // Can not simply break because then we would return RanOutOfAttempts
                continue 'place_loop;
            }

            return Err(RanOutOfAttempts);
        }

        Ok(())
    }
}
