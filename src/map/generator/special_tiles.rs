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
        self.place_object_in_rooms(rng, map, valid_rooms, self.next_prev_tiles, TileObject::ToNextLevel)
    }

    pub(in super) fn place_to_prev_level_tiles(
        &self,
        rng: &mut StdRng,
        map: &mut FloorMap,
        rooms: &[(RoomId, Room)],
    ) -> Result<(), RanOutOfAttempts> {
        let valid_rooms = rooms.iter().filter(|(_, r)| r.can_contain_to_prev_level()).cloned();
        self.place_object_in_rooms(rng, map, valid_rooms, self.next_prev_tiles, TileObject::ToPrevLevel)
    }

    /// Places `nrooms` copies of a TileObject into `nrooms` randomly choosen rooms from rooms
    fn place_object_in_rooms<OB>(
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

                assert!(grid.is_room(pos, room_id),
                    "bug: picked a tile that was not in the room it was supposed to be");

                // Don't put anything beside a doorway
                //TODO: This is not correct. Needs to check for passage surrounded by walls or something.
                if grid.adjacent_positions(pos).find(|&p| grid.is_room(p, room_id)).is_some() {
                    continue;
                }

                let tile = grid.get_mut(pos).expect("bug: did not choose a valid room tile");
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
