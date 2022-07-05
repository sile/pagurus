use crate::event::Event;
use crate::failure::Failure;
use crate::i18n::{LanguageTag, TimeZone};
use crate::resource::ResourceName;
use crate::spatial::Size;
use std::num::NonZeroU32;
use std::time::Duration;

pub mod event;
pub mod failure;
pub mod i18n;
pub mod input;
pub mod resource;
pub mod spatial;

pub trait System {
    fn video_render(&mut self, frame: VideoFrame);
    fn audio_enqueue(&mut self, data: AudioData) -> usize;
    fn audio_cancel(&mut self);
    fn console_log(&mut self, message: &str);
    fn clock_game_time(&mut self) -> Duration;
    fn clock_unix_time(&mut self) -> Duration;
    fn clock_set_timeout(&mut self, timeout: Duration, tag: u64);
    fn resource_put(&mut self, name: &ResourceName, data: &[u8]);
    fn resource_get(&mut self, name: &ResourceName);
    fn resource_delete(&mut self, name: &ResourceName);
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemConfig {
    pub window_size: Size,
    pub language: LanguageTag,
    pub time_zone: TimeZone,
}

#[derive(Debug)]
pub struct VideoFrame<'a> {
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
    data: &'a [u8],
}

impl<'a> AudioData<'a> {
    pub const CHANNELS: u8 = 1;
    pub const SAMPLE_RATE: u32 = 48_000;
    pub const BIT_DEPTH: u8 = 16;

    pub fn new(data: &'a [u8]) -> Option<Self> {
        (data.len() % 2 == 0).then(|| Self { data })
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameRequirements {
    // TODO: aspect_ratio
    #[serde(default)]
    pub window_size: Option<Size>,

    #[serde(default)]
    pub memory_bytes: Option<NonZeroU32>,
}

pub trait Game<S: System> {
    fn requirements(&self) -> Result<GameRequirements>;
    fn initialize(&mut self, system: &mut S, config: SystemConfig) -> Result<()>;
    fn handle_event(&mut self, system: &mut S, event: Event) -> Result<bool>;
}

pub type Result<T, E = Failure> = std::result::Result<T, E>;
