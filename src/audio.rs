use crate::failure::OrFail;
use crate::Result;

#[derive(Debug)]
pub struct AudioData<'a> {
    bytes: &'a [u8],
}

impl<'a> AudioData<'a> {
    pub const CHANNELS: u8 = 1;
    pub const SAMPLE_RATE: u32 = 48_000;
    pub const BIT_DEPTH: u8 = 16;

    pub fn new(bytes: &'a [u8]) -> Result<Self> {
        (bytes.len() % 2 == 0).or_fail()?;
        Ok(Self { bytes })
    }

    pub fn bytes(&self) -> &[u8] {
        self.bytes
    }

    pub fn samples(&self) -> impl 'a + Iterator<Item = i16> {
        self.bytes
            .chunks_exact(2)
            .map(|v| (i16::from(v[0]) << 8) | i16::from(v[1]))
    }
}
