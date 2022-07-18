use spatial::Position;

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
    fn video_draw(&mut self, frame: VideoFrame<&[u8]>);
    fn audio_enqueue(&mut self, data: AudioData) -> usize;
    fn console_log(&mut self, message: &str);
    fn clock_game_time(&mut self) -> Duration;
    fn clock_unix_time(&mut self) -> Duration;
    fn clock_set_timeout(&mut self, timeout: Duration) -> ActionId;
    fn state_save(&mut self, name: &str, data: &[u8]) -> ActionId;
    fn state_load(&mut self, name: &str) -> ActionId;
    fn state_delete(&mut self, name: &str) -> ActionId;
}

pub trait Game<S: System> {
    fn initialize(&mut self, system: &mut S) -> Result<()>;
    fn handle_event(&mut self, system: &mut S, event: Event) -> Result<bool>;
}

#[derive(Debug)]
pub struct VideoFrame<B> {
    bytes: B,
    frame_size: Size,
}

impl<B: AsRef<[u8]>> VideoFrame<B> {
    pub const PIXEL_FORMAT: &'static str = "RGB24";

    pub fn new(bytes: B, width: u32) -> Result<Self> {
        let n = bytes.as_ref().len();
        (n % 3 == 0).or_fail()?;
        ((n / 3) as u32 % width == 0).or_fail()?;

        let frame_size = Size::from_wh(width, (n / 3) as u32 / width);
        Ok(Self { bytes, frame_size })
    }

    pub fn bytes(&self) -> &[u8] {
        self.bytes.as_ref()
    }

    pub fn r8g8b8_pixels(&self) -> impl '_ + Iterator<Item = (Position, (u8, u8, u8))> {
        (0..self.size().height).flat_map(move |y| {
            (0..self.size().width).map(move |x| {
                let pos = Position::from_xy(x as i32, y as i32);
                let i = y as usize * (self.size().width as usize) + x as usize;
                let r = self.bytes()[i * 3];
                let g = self.bytes()[i * 3 + 1];
                let b = self.bytes()[i * 3 + 2];
                (pos, (r, g, b))
            })
        })
    }

    pub fn r5g6b5_pixels(&self) -> impl '_ + Iterator<Item = (Position, u16)> {
        self.r8g8b8_pixels().map(|(pos, (r, g, b))| {
            let r = u16::from(r);
            let g = u16::from(g);
            let b = u16::from(b);
            let pixel = ((r << 8) & 0xf800) | ((g << 3) & 0x07e0) | (b >> 3);
            (pos, pixel)
        })
    }

    pub fn size(&self) -> Size {
        self.frame_size
    }

    pub fn as_ref(&self) -> VideoFrame<&[u8]> {
        VideoFrame {
            bytes: self.bytes.as_ref(),
            frame_size: self.frame_size,
        }
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
