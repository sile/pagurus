use crate::audio::AudioData;
use crate::audio::AudioSpec;
use crate::event::Event;
use crate::spatial::Size;
use crate::timeout::{TimeoutId, TimeoutTag};
use crate::video::{VideoFrame, VideoFrameSpec};
use std::time::Duration;

pub mod failure {
    pub use orfail::{Failure, OrFail};
}
pub use orfail::{todo, unreachable};

pub mod audio;
pub mod event;
#[cfg(feature = "fixed_window")]
pub mod fixed_window;
#[cfg(feature = "image")]
pub mod image;
pub mod input;
#[cfg(feature = "random")]
pub mod random;
pub mod spatial;
pub mod timeout;
pub mod video;
#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(feature = "wasm")]
mod logger;

pub type Result<T, E = crate::failure::Failure> = std::result::Result<T, E>;

pub trait System {
    // TODO: type Event, Command, Query;

    fn video_init(&mut self, resolution: Size) -> VideoFrameSpec;
    fn video_draw(&mut self, frame: VideoFrame<&[u8]>);
    fn audio_init(&mut self, sample_rate: u16, data_samples: usize) -> AudioSpec;
    fn audio_enqueue(&mut self, data: AudioData<&[u8]>);
    fn console_log(message: &str);
    fn clock_game_time(&self) -> Duration;
    fn clock_unix_time(&self) -> Duration;
    fn clock_set_timeout(&mut self, tag: TimeoutTag, timeout: Duration) -> TimeoutId;
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
