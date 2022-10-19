use crate::window::{Window, WindowBuilder};
use pagurus::{
    audio::AudioData,
    event::Event,
    failure::OrFail,
    spatial::Size,
    video::{PixelFormat, VideoFrame, VideoFrameSpec},
    ActionId, Result, System,
};
use std::{
    path::PathBuf,
    time::{Duration, Instant, UNIX_EPOCH},
};

#[derive(Debug)]
pub struct WindowsSystemBuilder {
    data_dir: PathBuf,
    window: WindowBuilder,
}

impl WindowsSystemBuilder {
    pub fn new(title: &str) -> Self {
        Self {
            data_dir: PathBuf::from(WindowsSystem::DEFAULT_DATA_DIR),
            window: WindowBuilder::new(title),
        }
    }

    pub fn window_size(mut self, size: Option<Size>) -> Self {
        self.window = self.window.window_size(size);
        self
    }

    pub fn build(self) -> Result<WindowsSystem> {
        let window = self.window.build().or_fail()?;
        Ok(WindowsSystem {
            window,
            start: Instant::now(),
        })
    }
}

#[derive(Debug)]
pub struct WindowsSystem {
    window: Window,
    start: Instant,
}

impl WindowsSystem {
    pub const DEFAULT_DATA_DIR: &'static str = "data/";
}

impl WindowsSystem {
    pub fn next_event(&mut self) -> Event {
        self.window.next_event()
    }
}

impl System for WindowsSystem {
    fn video_draw(&mut self, frame: VideoFrame<&[u8]>) {
        self.window
            .draw_video_frame(frame)
            .unwrap_or_else(|e| panic!("{e}"));
    }

    fn video_frame_spec(&mut self, resolution: Size) -> VideoFrameSpec {
        let w = resolution.width;
        let stride = w + (4 - (w % 4)) % 4;
        VideoFrameSpec {
            pixel_format: PixelFormat::Bgr24,
            resolution,
            stride,
        }
    }

    fn audio_enqueue(&mut self, data: AudioData) -> usize {
        // todo!()
        data.samples().count()
    }

    fn console_log(&mut self, message: &str) {
        eprintln!("{message}");
    }

    fn clock_game_time(&mut self) -> Duration {
        self.start.elapsed()
    }

    fn clock_unix_time(&mut self) -> Duration {
        UNIX_EPOCH
            .elapsed()
            .unwrap_or_else(|e| panic!("failed to get UNIX timestamp: {e}"))
    }

    fn clock_set_timeout(&mut self, timeout: Duration) -> ActionId {
        //todo!()
        ActionId::new(0)
    }

    fn state_save(&mut self, name: &str, data: &[u8]) -> ActionId {
        //todo!()
        ActionId::new(0)
    }

    fn state_load(&mut self, name: &str) -> ActionId {
        //todo!()
        ActionId::new(0)
    }

    fn state_delete(&mut self, name: &str) -> ActionId {
        //todo!()
        ActionId::new(0)
    }
}
