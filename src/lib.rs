use crate::color::Rgb;
use crate::spatial::Size;
use std::num::NonZeroU32;
use std::time::Duration;

pub mod color;
pub mod spatial;

pub const AUDIO_CHANNELS: u8 = 1;
pub const AUDIO_SAMPLE_RATE: u32 = 48_000;

pub trait System {
    fn image_render(&mut self, data: &[Rgb], image_size: Size);
    fn audio_enqueue(&mut self, data: &[i16]);
    fn audio_cancel(&mut self);
    fn clock_now(&mut self) -> Duration;
    fn console_log(&mut self, message: &str);
    fn resource_put(&mut self, uri: &str, data: &[u8]) -> ActionId;
    fn resource_get(&mut self, uri: &str) -> ActionId;
    fn resource_delete(&mut self, uri: &str) -> ActionId;
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameRequirements {
    #[serde(default)]
    pub window_size: Option<Size>,

    #[serde(default)]
    pub fps: Option<NonZeroU32>,

    #[serde(default)]
    pub memory_bytes: Option<NonZeroU32>,
}

pub trait Game<S: System> {
    fn requirements(&self) -> GameRequirements;
    fn initialize(&mut self, system: &mut S);
    fn handle_event(&mut self, system: &mut S, event: Event);
    fn is_finished(&self) -> bool;
}

// TODO:
pub enum Event {}

#[derive(
    Debug,
    Clone,
    Copy,
    Default,
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

    // fetch_and_increment
}
