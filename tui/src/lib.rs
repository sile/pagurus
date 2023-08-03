use pagurus::{
    audio::{AudioData, AudioSpec, SampleFormat},
    event::{Event, TimeoutTag},
    spatial::Size,
    video::{VideoFrame, VideoFrameSpec},
    System,
};
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, VecDeque},
    time::{Duration, Instant, UNIX_EPOCH},
};

#[derive(Debug)]
pub struct TuiSystem {
    start_time: Instant,
    event_queue: VecDeque<Event>,
    timeout_queue: BinaryHeap<Reverse<(Duration, TimeoutTag)>>,
}

impl TuiSystem {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            event_queue: VecDeque::new(),
            timeout_queue: BinaryHeap::new(),
        }
    }

    pub fn next_event(&mut self) -> Option<Event> {
        while let Some(Reverse((timeout, tag))) = self.timeout_queue.peek().copied() {
            let now = self.clock_game_time();
            if timeout <= now {
                self.timeout_queue.pop();
                return Some(Event::Timeout(tag));
            } else {
                break;
            }
        }
        self.event_queue.pop_front()
    }
}

impl System for TuiSystem {
    fn video_init(&mut self, resolution: Size) -> VideoFrameSpec {
        todo!()
    }

    fn video_draw(&mut self, frame: VideoFrame<&[u8]>) {
        todo!()
    }

    fn audio_init(&mut self, sample_rate: u16, data_samples: usize) -> AudioSpec {
        // Returns dummy spec.
        AudioSpec {
            sample_format: SampleFormat::I16Be,
            sample_rate,
            data_samples,
        }
    }

    fn audio_enqueue(&mut self, _data: AudioData<&[u8]>) {
        // Discards audio data as TUI does not support audio.
    }

    fn clock_game_time(&self) -> Duration {
        self.start_time.elapsed()
    }

    fn clock_unix_time(&self) -> Duration {
        UNIX_EPOCH.elapsed().expect("cannot get UNIX time")
    }

    fn clock_set_timeout(&mut self, tag: TimeoutTag, timeout: Duration) {
        let now = self.clock_game_time();
        self.timeout_queue.push(Reverse((now + timeout, tag)));
    }
}
