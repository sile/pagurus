use rand_chacha::ChaChaRng;
use rand_core::{RngCore, SeedableRng as _};
use std::{num::NonZeroU32, time::Duration};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct StdRngState {
    pub seed: [u8; 32],
    pub word_pos: u128,
}

#[derive(Debug)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct StdRng(ChaChaRng);

impl StdRng {
    pub fn from_clock_seed(now: Duration) -> Self {
        let inner = ChaChaRng::seed_from_u64(now.as_micros() as u64);
        Self(inner)
    }

    pub fn from_state(state: StdRngState) -> Self {
        let mut inner = ChaChaRng::from_seed(state.seed);
        inner.set_word_pos(state.word_pos);
        Self(inner)
    }

    pub fn state(&self) -> StdRngState {
        StdRngState {
            seed: self.0.get_seed(),
            word_pos: self.0.get_word_pos(),
        }
    }
}

impl Default for StdRng {
    fn default() -> Self {
        Self::from_clock_seed(Duration::from_secs(0))
    }
}

impl Clone for StdRng {
    fn clone(&self) -> Self {
        Self::from_state(self.state())
    }
}

impl RngCore for StdRng {
    fn next_u32(&mut self) -> u32 {
        self.0.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0.fill_bytes(dest);
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.0.try_fill_bytes(dest)
    }
}

fn always_fail(_buf: &mut [u8]) -> Result<(), getrandom::Error> {
    let custom_code = getrandom::Error::CUSTOM_START + 7397;
    Err(NonZeroU32::new(custom_code).expect("unreachable").into())
}

getrandom::register_custom_getrandom!(always_fail);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clone_std_rng_works() {
        let mut rng = StdRng::default();

        for _ in 0..100 {
            let mut cloned = rng.clone();
            assert_eq!(rng.next_u32(), cloned.next_u32());
        }
    }
}
