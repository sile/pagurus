use crate::audio::AudioData;
use crate::event::Event;
use crate::spatial::Size;
use crate::video::{VideoFrame, VideoFrameSpec};
use std::time::Duration;

pub mod failure {
    pub use orfail::{Failure, OrFail};
}

pub mod audio;
pub mod event;
pub mod input;
pub mod spatial;
pub mod video;

pub type Result<T, E = crate::failure::Failure> = std::result::Result<T, E>;

pub trait System {
    fn video_draw(&mut self, frame: VideoFrame<&[u8]>);
    fn video_frame_spec(&mut self, resolution: Size) -> VideoFrameSpec;
    fn audio_enqueue(&mut self, data: AudioData) -> usize;
    fn console_log(message: &str);
    fn clock_game_time(&self) -> Duration;
    fn clock_unix_time(&self) -> Duration;
    fn clock_set_timeout(&mut self, timeout: Duration) -> ActionId;
    fn state_save(&mut self, name: &str, data: &[u8]) -> ActionId;
    fn state_load(&mut self, name: &str) -> ActionId;
    fn state_delete(&mut self, name: &str) -> ActionId;
}

pub trait Game<S: System> {
    fn initialize(&mut self, system: &mut S) -> Result<()>;
    fn handle_event(&mut self, system: &mut S, event: Event) -> Result<bool>;

    #[allow(unused_variables)]
    fn query(&mut self, system: &mut S, name: &str) -> Result<Vec<u8>> {
        Err(crate::failure::Failure::new().message(format!("unknown query: {name:?}")))
    }

    #[allow(unused_variables)]
    fn command(&mut self, system: &mut S, name: &str, data: &[u8]) -> Result<()> {
        Err(crate::failure::Failure::new().message(format!("unknown command: {name:?}")))
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
