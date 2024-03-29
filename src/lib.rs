use crate::audio::AudioData;
use crate::audio::AudioSpec;
use crate::event::Event;
use crate::event::TimeoutTag;
use crate::spatial::Size;
use crate::video::{VideoFrame, VideoFrameSpec};
use std::time::Duration;

pub mod audio;
pub mod event;
#[cfg(feature = "fixed_window")]
pub mod fixed_window;
#[cfg(feature = "image")]
pub mod image;
pub mod io;
#[cfg(feature = "random")]
pub mod random;
pub mod spatial;
pub mod video;
#[cfg(feature = "wasm")]
pub mod wasm;

pub type Result<T, E = orfail::Failure> = std::result::Result<T, E>;

pub trait System {
    fn video_init(&mut self, resolution: Size) -> VideoFrameSpec;
    fn video_draw(&mut self, frame: VideoFrame<&[u8]>);
    fn audio_init(&mut self, sample_rate: u16, data_samples: usize) -> AudioSpec;
    fn audio_enqueue(&mut self, data: AudioData<&[u8]>);
    fn clock_game_time(&self) -> Duration;
    fn clock_unix_time(&self) -> Duration;
    fn clock_set_timeout(&mut self, tag: TimeoutTag, timeout: Duration);
}

pub trait Game<S: System> {
    fn initialize(&mut self, system: &mut S) -> Result<()>;
    fn handle_event(&mut self, system: &mut S, event: Event) -> Result<bool>;

    #[allow(unused_variables)]
    fn query(&mut self, system: &mut S, name: &str) -> Result<Vec<u8>> {
        Err(orfail::Failure::new(format!("unknown query: {name:?}")))
    }

    #[allow(unused_variables)]
    fn command(&mut self, system: &mut S, name: &str, data: &[u8]) -> Result<()> {
        Err(orfail::Failure::new(format!("unknown command: {name:?}")))
    }
}
