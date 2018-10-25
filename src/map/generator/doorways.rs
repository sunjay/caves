use rand::{StdRng, Rng};

use super::{MapGenerator, RanOutOfAttempts};
use map::*;

impl MapGenerator {
    pub(in super) fn connect_rooms(&self, rng: &mut StdRng, map: &mut FloorMap) {

    }

    pub(in super) fn place_locks(&self, rng: &mut StdRng, map: &mut FloorMap) {

    }
}
