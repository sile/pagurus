use crate::{
    audio::AudioPlayer,
    window::{Window, WindowBuilder},
};
use pagurus::{
    audio::AudioData,
    event::{Event, TimeoutEvent},
    failure::OrFail,
    spatial::Size,
    video::{PixelFormat, VideoFrame, VideoFrameSpec},
    ActionId, Result, System,
};
use std::{
    cmp::Reverse,
    collections::BinaryHeap,
    path::PathBuf,
    time::{Duration, Instant, UNIX_EPOCH},
};

#[derive(Debug)]
pub struct WindowsSystemBuilder {
    data_dir: PathBuf,
    enable_audio: bool,
    window: WindowBuilder,
}

impl WindowsSystemBuilder {
    pub fn new(title: &str) -> Self {
        Self {
            data_dir: PathBuf::from(WindowsSystem::DEFAULT_DATA_DIR),
            enable_audio: true,
            window: WindowBuilder::new(title),
        }
    }

    pub fn window_size(mut self, size: Option<Size>) -> Self {
        self.window = self.window.window_size(size);
        self
    }

    pub fn enable_audio(mut self, enable: bool) -> Self {
        self.enable_audio = enable;
        self
    }

    pub fn build(self) -> Result<WindowsSystem> {
        let window = self.window.build().or_fail()?;
        Ok(WindowsSystem {
            window,
            audio_player: if self.enable_audio {
                Some(AudioPlayer::new().or_fail()?)
            } else {
                None
            },
            start: Instant::now(),
            timeout_queue: BinaryHeap::new(),
            next_action_id: ActionId::new(0),
        })
    }
}

#[derive(Debug)]
pub struct WindowsSystem {
    window: Window,
    audio_player: Option<AudioPlayer>,
    start: Instant,
    timeout_queue: BinaryHeap<Reverse<(Instant, ActionId)>>,
    next_action_id: ActionId,
}

impl WindowsSystem {
    pub const DEFAULT_DATA_DIR: &'static str = "data/";
}

impl WindowsSystem {
    pub fn next_event(&mut self) -> Event {
        loop {
            if let Some(&Reverse((timeout, id))) = self.timeout_queue.peek() {
                if timeout <= Instant::now() {
                    self.timeout_queue.pop();
                    return Event::Timeout(TimeoutEvent { id });
                }
            }

            let timeout = self.timeout_queue.peek().map(|x| x.0 .0);
            if let Some(event) = self.window.next_event(timeout) {
                return event;
            }
        }
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
        let samples = data.samples().count();
        if let Some(player) = &mut self.audio_player {
            player.play(data).unwrap_or_else(|e| panic!("{e}"));
        }
        samples
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
        let id = self.next_action_id.get_and_increment();
        self.timeout_queue
            .push(Reverse((Instant::now() + timeout, id)));
        id
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
