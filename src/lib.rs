use crate::event::Event;
use crate::failure::{Failure, OrFail};
use crate::spatial::Size;
use std::time::Duration;

pub mod event;
pub mod failure;
pub mod input;
pub mod spatial;

pub type Result<T, E = Failure> = std::result::Result<T, E>;

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

pub trait Game<S: System> {
    fn requirements(&self) -> Result<Requirements>;
    fn initialize(&mut self, system: &mut S, config: Configuration) -> Result<()>;
    fn handle_event(&mut self, system: &mut S, event: Event) -> Result<bool>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    pub initial_window_size: Size,
    pub game_argument: String,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            initial_window_size: Size::from_wh(0, 0),
            game_argument: String::new(),
        }
    }
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Requirements {
    #[serde(default)]
    pub logical_window_size: Option<Size>,
}

#[derive(Debug)]
pub struct VideoFrame<'a> {
    bytes: &'a [u8],
    frame_size: Size,
}

impl<'a> VideoFrame<'a> {
    pub const PIXEL_FORMAT: &'static str = "RGB24";

    pub fn new(bytes: &'a [u8], width: u32) -> Result<Self> {
        (bytes.len() % 3 == 0).or_fail()?;
        ((bytes.len() / 3) as u32 % width == 0).or_fail()?;

        let frame_size = Size::from_wh(width, (bytes.len() / 3) as u32 / width);
        Ok(Self { bytes, frame_size })
    }

    pub fn bytes(&self) -> &[u8] {
        self.bytes
    }

    pub fn size(&self) -> Size {
        self.frame_size
    }
}

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
