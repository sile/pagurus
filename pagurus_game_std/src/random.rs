use rand_chacha::ChaChaRng;
use rand_core::{RngCore, SeedableRng as _};
use std::{num::NonZeroU32, time::Duration};

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct StdRngState {
    pub seed: [u8; 32],
    pub word_pos: u128,
}

#[derive(Debug)]
pub struct StdRng {
    inner: ChaChaRng,
}

impl StdRng {
    pub fn from_clock_seed(now: Duration) -> Self {
        let inner = ChaChaRng::seed_from_u64(now.as_micros() as u64);
        Self { inner }
    }

    pub fn from_state(state: StdRngState) -> Self {
        let mut inner = ChaChaRng::from_seed(state.seed);
        inner.set_word_pos(state.word_pos);
        Self { inner }
    }

    pub fn state(&self) -> StdRngState {
        StdRngState {
            seed: self.inner.get_seed(),
            word_pos: self.inner.get_word_pos(),
        }
    }
}

impl RngCore for StdRng {
    fn next_u32(&mut self) -> u32 {
        self.inner.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.inner.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.inner.fill_bytes(dest);
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.inner.try_fill_bytes(dest)
    }
}

fn always_fail(_buf: &mut [u8]) -> Result<(), getrandom::Error> {
    let custom_code = getrandom::Error::CUSTOM_START + 7397;
    Err(NonZeroU32::new(custom_code).expect("unreachable").into())
}

getrandom::register_custom_getrandom!(always_fail);
