use crate::window::Window;
use pagurus::{
    audio::AudioData,
    event::{Event, WindowEvent},
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
    title: String,
    window_size: Option<Size>,
}

impl WindowsSystemBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_owned();
        self
    }

    pub fn window_size(mut self, size: Option<Size>) -> Self {
        self.window_size = size;
        self
    }

    pub fn build(self) -> Result<WindowsSystem> {
        let window = Window::new(&self.title, self.window_size).or_fail()?;
        Ok(WindowsSystem {
            window,
            start: Instant::now(),
        })
    }
}

impl Default for WindowsSystemBuilder {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from(WindowsSystem::DEFAULT_DATA_DIR),
            title: WindowsSystem::DEFAULT_TITLE.to_owned(),
            window_size: Some(WindowsSystem::DEFAULT_WINDOW_SIZE),
        }
    }
}

#[derive(Debug)]
pub struct WindowsSystem {
    window: Window,
    start: Instant,
}

impl WindowsSystem {
    pub const DEFAULT_TITLE: &'static str = "Pagurus";
    pub const DEFAULT_WINDOW_SIZE: Size = Size::from_wh(800, 600);
    pub const DEFAULT_DATA_DIR: &'static str = "data/";
}

impl WindowsSystem {
    pub fn next_event(&mut self) -> Event {
        self.window.dispatch();
        std::thread::sleep(Duration::from_secs(1));

        Event::Window(WindowEvent::RedrawNeeded {
            size: Size::from_wh(800, 600),
        })
    }
}

impl System for WindowsSystem {
    fn video_draw(&mut self, frame: VideoFrame<&[u8]>) {
        let dc = self.window.get_dc().unwrap_or_else(|e| panic!("{e}"));
        dc.draw_bitmap(frame).unwrap_or_else(|e| panic!("{e}"));
    }

    fn video_frame_spec(&mut self, resolution: Size) -> VideoFrameSpec {
        VideoFrameSpec {
            pixel_format: PixelFormat::Rgb24,
            resolution,
            stride: resolution.width,
        }
    }

    fn audio_enqueue(&mut self, data: AudioData) -> usize {
        todo!()
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
        todo!()
    }

    fn state_save(&mut self, name: &str, data: &[u8]) -> ActionId {
        todo!()
    }

    fn state_load(&mut self, name: &str) -> ActionId {
        todo!()
    }

    fn state_delete(&mut self, name: &str) -> ActionId {
        todo!()
    }
}
