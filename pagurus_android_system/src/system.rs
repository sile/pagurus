use pagurus::{ActionId, AudioData, System, VideoFrame};
use std::time::{Duration, Instant, UNIX_EPOCH};

#[derive(Debug)]
pub struct AndroidSystem {
    start: Instant,
    next_action_id: ActionId,
}

impl AndroidSystem {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            next_action_id: ActionId::default(),
        }
    }
}

impl System for AndroidSystem {
    fn video_render(&mut self, frame: VideoFrame<&[u8]>) {
        todo!()
    }

    fn audio_enqueue(&mut self, data: AudioData) -> usize {
        todo!()
    }

    fn console_log(&mut self, message: &str) {
        println!("{message}");
    }

    fn clock_game_time(&mut self) -> Duration {
        self.start.elapsed()
    }

    fn clock_unix_time(&mut self) -> Duration {
        UNIX_EPOCH.elapsed().unwrap_or_else(|e| panic!("{e}"))
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
