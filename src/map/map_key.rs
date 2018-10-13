use std::str::FromStr;
use std::fmt;

use rand::{
    StdRng,
    Rng,
    SeedableRng,
    distributions::{
        Distribution,
        Standard,
    },
};
use base64::{self, DecodeError};

lazy_static! {
    /// The configuration of the encoder/decoder for the seed
    static ref SEED_ENCODER_CONFIG: base64::Config = base64::Config::new(
        base64::CharacterSet::UrlSafe,
        false,
        false,
        base64::LineWrap::NoWrap,
    );
}

#[derive(Debug)]
pub enum InvalidMapKey {
    InvalidLength,
    DecodeError(DecodeError),
}

/// The seed of the random number generator
type Seed = <StdRng as SeedableRng>::Seed;

/// Uniquely identifies a map
///
/// Can be passed to the generator to recreate a specific map.
///
/// To create a random MapKey, use the `rand::random` function:
///
/// ```rust
/// # use rand::random;
/// # use map_generator::MapKey;
/// let map_key: MapKey = random();
/// ```
///
/// MapKeys can be parsed from strings using `.parse()`:
///
/// ```rust,no_run
/// # use map_generator::MapKey;
/// let map_key: MapKey = "yourvalidmapkey".parse();
/// ```
///
/// You can get the string representation of a MapKey either with `.to_string()` or
/// by directly using Display `{}` formatting:
///
/// ```rust,no_run
/// # use rand::random;
/// # use map_generator::MapKey;
/// let map_key: MapKey = random();
/// assert_eq!(format!("{}", map_key), map_key.to_string());
/// ```
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MapKey(Seed);

impl MapKey {
    pub(in super) fn to_rng(self) -> StdRng {
        StdRng::from_seed(self.0)
    }
}

impl Distribution<MapKey> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> MapKey {
        MapKey(rng.gen())
    }
}

impl fmt::Debug for MapKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MapKey(\"{}\")", self)
    }
}

impl fmt::Display for MapKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", base64::encode_config(&self.0, *SEED_ENCODER_CONFIG))
    }
}

impl FromStr for MapKey {
    type Err = InvalidMapKey;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut key: Seed = Default::default();
        let decoded = base64::decode_config(s, *SEED_ENCODER_CONFIG)
            .map_err(|err| InvalidMapKey::DecodeError(err))?;
        if decoded.len() != key.len() {
            return Err(InvalidMapKey::InvalidLength);
        }
        key.copy_from_slice(&decoded);
        Ok(MapKey(key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand::random;

    #[test]
    fn unique_map_key_can_decode_itself() {
        // Generates random MapKeys and checks if they are at least different from their previous
        // keys. Then ensures that the MapKey can decode its encoded form.
        let runs = 10000;

        let mut prev_key: MapKey = random();
        let mut prev_key_encoded = prev_key.to_string();
        for _ in 0..runs {
            let key: MapKey = random();

            let encoded = key.to_string();
            assert_ne!(key, prev_key);
            assert_ne!(encoded, prev_key_encoded);

            // Encoding and decoding should result in the same key
            assert_eq!(key, encoded.parse().unwrap());
            // Should not be the same as the previous key (redundant but important check)
            assert_ne!(prev_key, encoded.parse().unwrap());

            prev_key = key;
            prev_key_encoded = encoded;
        }
    }
}
