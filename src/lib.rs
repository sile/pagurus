use crate::event::Event;
use crate::i18n::{LanguageTag, TimeZone};
use crate::spatial::Size;
use std::num::NonZeroU32;
use std::time::Duration;

mod error;
pub mod event;
pub mod i18n;
pub mod input;
pub mod spatial;

pub use crate::error::{Failure, OrFail};

pub trait System {
    fn video_render(&mut self, frame: VideoFrame);
    fn audio_enqueue(&mut self, data: AudioData) -> usize;
    fn audio_cancel(&mut self);
    fn console_log(&mut self, message: &str);
    fn clock_game_time(&mut self) -> Duration;
    fn clock_unix_time(&mut self) -> Duration;
    fn clock_set_timeout(&mut self, timeout: Duration, tag: u64);
    fn resource_put(&mut self, name: &str, data: &[u8]);
    fn resource_get(&mut self, name: &str);
    fn resource_delete(&mut self, name: &str);
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
    pub data: &'a [u8],
    pub size: Size,
}

impl<'a> VideoFrame<'a> {
    pub const PIXEL_FORMAT: &'static str = "RGB24";
}

#[derive(Debug)]
pub struct AudioData<'a> {
    pub data: &'a [u8],
}

impl<'a> AudioData<'a> {
    pub const CHANNELS: u8 = 1;
    pub const SAMPLE_RATE: u32 = 48_000;
    pub const BIT_DEPTH: u8 = 16;
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameRequirements {
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
