#[cfg(feature = "audio")]
mod audio;

#[cfg(feature = "video")]
mod video;

use orfail::{Failure, OrFail};
use pagurus::{
    audio::{AudioData, AudioSpec, SampleFormat},
    event::{Event, TimeoutTag},
    spatial::Size,
    video::{PixelFormat, VideoFrame, VideoFrameSpec},
    System,
};
use std::{
    cmp::Reverse,
    collections::BinaryHeap,
    time::{Duration, Instant, UNIX_EPOCH},
};

#[derive(Debug, Default, Clone)]
pub struct TuiSystemOptions {
    #[cfg(feature = "video")]
    pub disable_mouse: bool,
    #[cfg(feature = "video")]
    pub disable_alternate_screen: bool,
}

#[derive(Debug)]
pub struct TuiSystem {
    start_time: Instant,
    timeout_queue: BinaryHeap<Reverse<(Duration, TimeoutTag)>>,
    failed: Option<Failure>,

    #[cfg(feature = "audio")]
    audio: Option<self::audio::AudioSystem>,

    #[cfg(feature = "video")]
    video: self::video::VideoSystem,
}

impl TuiSystem {
    #[cfg_attr(not(feature = "video"), allow(unused_variables))]
    pub fn with_options(options: TuiSystemOptions) -> pagurus::Result<Self> {
        Ok(Self {
            start_time: Instant::now(),
            timeout_queue: BinaryHeap::new(),
            failed: None,

            #[cfg(feature = "audio")]
            audio: None,

            #[cfg(feature = "video")]
            video: self::video::VideoSystem::new(options).or_fail()?,
        })
    }

    pub fn new() -> pagurus::Result<Self> {
        Self::with_options(TuiSystemOptions::default())
    }

    pub fn next_event(&mut self) -> pagurus::Result<Event> {
        if let Some(e) = self.failed.take() {
            return Err(e);
        }

        if let Some(Reverse((expire_time, tag))) = self.timeout_queue.peek().copied() {
            let now = self.clock_game_time();
            if let Some(timeout) = expire_time.checked_sub(now) {
                if let Ok(event) = self.next_video_event(Some(timeout)) {
                    return Ok(event);
                }
            }
            self.timeout_queue.pop();
            Ok(Event::Timeout(tag))
        } else {
            self.next_video_event(None).or_fail()
        }
    }

    #[cfg_attr(not(feature = "video"), allow(unused_variables))]
    fn next_video_event(&mut self, timeout: Option<Duration>) -> orfail::Result<Event> {
        #[cfg(not(feature = "video"))]
        {
            Err(Failure::new("No more TUI events"))
        }
        #[cfg(feature = "video")]
        if let Some(timeout) = timeout {
            self.video.event_queue_mut().recv_timeout(timeout).or_fail()
        } else {
            self.video.event_queue_mut().recv().or_fail()
        }
    }

    #[cfg(feature = "video")]
    pub fn request_redraw(&mut self) -> pagurus::Result<()> {
        self.video.request_redraw().or_fail()
    }
}

impl System for TuiSystem {
    fn video_init(&mut self, resolution: Size) -> VideoFrameSpec {
        VideoFrameSpec {
            pixel_format: PixelFormat::Rgb24,
            resolution,
            stride: resolution.width,
        }
    }

    #[cfg_attr(not(feature = "video"), allow(unused_variables))]
    fn video_draw(&mut self, frame: VideoFrame<&[u8]>) {
        if self.failed.is_some() {
            return;
        }

        #[cfg(feature = "video")]
        {
            self.failed = self.video.draw(frame).err();
        }
    }

    fn audio_init(&mut self, sample_rate: u16, data_samples: usize) -> AudioSpec {
        #[cfg(feature = "audio")]
        match self::audio::AudioSystem::new(sample_rate) {
            Err(e) => {
                self.failed = Some(e);
            }
            Ok(audio) => {
                self.audio = Some(audio);
            }
        }

        AudioSpec {
            sample_format: SampleFormat::I16Be,
            sample_rate,
            data_samples,
        }
    }

    #[cfg_attr(not(feature = "audio"), allow(unused_variables))]
    fn audio_enqueue(&mut self, data: AudioData<&[u8]>) {
        if self.failed.is_some() {
            return;
        }

        #[cfg(feature = "audio")]
        {
            self.failed = self
                .audio
                .as_mut()
                .and_then(|a| a.enqueue(data).or_fail().err());
        }
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
