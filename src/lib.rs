use crate::event::Event;
use crate::failure::{Failure, OrFail};
use crate::spatial::Size;
use std::time::Duration;

pub mod event;
pub mod failure;
pub mod input;
pub mod spatial;

pub trait System {
    fn video_render(&mut self, frame: VideoFrame);
    fn audio_enqueue(&mut self, data: AudioData) -> usize;
    fn audio_cancel(&mut self);
    fn console_log(&mut self, message: &str);
    fn clock_game_time(&mut self) -> Duration;
    fn clock_unix_time(&mut self) -> Duration;
    fn clock_set_timeout(&mut self, timeout: Duration) -> ActionId;
    fn state_save(&mut self, name: &str, data: &[u8]) -> ActionId;
    fn state_load(&mut self, name: &str) -> ActionId;
    fn state_delete(&mut self, name: &str) -> ActionId;
}

// TODO: rename
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemConfig {
    pub window_size: Size,
}

#[derive(Debug)]
pub struct VideoFrame<'a> {
    // TODO: bytes
    data: &'a [u8],
    size: Size,
}

impl<'a> VideoFrame<'a> {
    pub const PIXEL_FORMAT: &'static str = "RGB24";

    pub fn new(data: &'a [u8], width: u32) -> Option<Self> {
        if data.len() % 3 != 0 {
            None
        } else if (data.len() / 3) as u32 % width != 0 {
            None
        } else {
            let size = Size::from_wh(width, (data.len() / 3) as u32 / width);
            Some(Self { data, size })
        }
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn size(&self) -> Size {
        self.size
    }
}

#[derive(Debug)]
pub struct AudioData<'a> {
    // TODO: bytes
    data: &'a [u8],
}

impl<'a> AudioData<'a> {
    pub const CHANNELS: u8 = 1;
    pub const SAMPLE_RATE: u32 = 48_000;
    pub const BIT_DEPTH: u8 = 16;

    pub fn new(data: &'a [u8]) -> Result<Self> {
        (data.len() % 2 == 0).or_fail()?;
        Ok(Self { data })
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn samples(&self) -> impl 'a + Iterator<Item = i16> {
        self.data
            .chunks_exact(2)
            .map(|v| (i16::from(v[0]) << 8) | i16::from(v[1]))
    }
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameRequirements {
    #[serde(default)]
    pub logical_window_size: Option<Size>,
}

pub trait Game<S: System> {
    fn requirements(&self) -> Result<GameRequirements>;
    fn initialize(&mut self, system: &mut S, config: SystemConfig) -> Result<()>;
    fn handle_event(&mut self, system: &mut S, event: Event) -> Result<bool>;
}

pub type Result<T, E = Failure> = std::result::Result<T, E>;

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct ActionId(u64);

impl ActionId {
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    pub fn increment(&mut self) {
        self.0 += 1;
    }

    pub fn get_and_increment(&mut self) -> Self {
        let id = *self;
        self.increment();
        id
    }
}
