use crate::event::EventPoller;
use crate::io_thread::{IoRequest, IoThread};
use crate::window::Window;
use ndk::audio::{AudioFormat, AudioStream, AudioStreamState};
use pagurus::event::{Event, TimeoutEvent, WindowEvent};
use pagurus::failure::OrFail;
use pagurus::spatial::Size;
use pagurus::video::{PixelFormat, VideoFrameSpec};
use pagurus::{audio::AudioData, video::VideoFrame, ActionId, Result, System};
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::sync::mpsc;
use std::{
    path::PathBuf,
    time::{Duration, Instant, UNIX_EPOCH},
};

#[derive(Debug, Default)]
pub struct AndroidSystemBuilder {}

impl AndroidSystemBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(self) -> Result<AndroidSystem> {
        #[allow(deprecated)]
        let data_dir = PathBuf::from(
            ndk_glue::native_activity()
                .internal_data_path()
                .to_str()
                .or_fail()?,
        );

        let event_poller = EventPoller::new().or_fail()?;

        let (event_tx, event_rx) = mpsc::channel();

        if let Some(window) = &ndk_glue::native_window() {
            let size = Window::new(window).get_window_size();
            let _ = event_tx.send(Event::Window(WindowEvent::RedrawNeeded { size }));
        }

        let io_request_tx = IoThread::spawn(event_tx.clone(), event_poller.notifier());

        let audio = ndk::audio::AudioStreamBuilder::new()
            .or_fail()?
            .channel_count(AudioData::CHANNELS as i32)
            .format(AudioFormat::PCM_I16)
            .sample_rate(AudioData::SAMPLE_RATE as i32)
            .open_stream()
            .or_fail()?;

        Ok(AndroidSystem {
            start: Instant::now(),
            audio,
            event_poller,
            event_rx,
            io_request_tx,
            timeout_queue: BinaryHeap::new(),
            next_action_id: ActionId::default(),
            data_dir,
        })
    }
}

#[derive(Debug)]
pub struct AndroidSystem {
    start: Instant,
    audio: AudioStream,
    next_action_id: ActionId,
    data_dir: PathBuf,
    event_poller: EventPoller,
    event_rx: mpsc::Receiver<Event>,
    io_request_tx: mpsc::Sender<IoRequest>,
    timeout_queue: BinaryHeap<(Reverse<Duration>, ActionId)>,
}

impl AndroidSystem {
    pub fn new() -> Result<Self> {
        AndroidSystemBuilder::default().build().or_fail()
    }

    pub fn wait_event(&mut self) -> Result<Event> {
        loop {
            let timeout =
                if let Some((Reverse(expiry_time), id)) = self.timeout_queue.peek().copied() {
                    if let Some(timeout) = expiry_time.checked_sub(self.start.elapsed()) {
                        timeout
                    } else {
                        self.timeout_queue.pop();
                        return Ok(Event::Timeout(TimeoutEvent { id }));
                    }
                } else {
                    Duration::from_secs(1) // Arbitrary large timeout value
                };

            match self.event_rx.try_recv() {
                Ok(event) => return Ok(event),
                Err(e) => match e {
                    mpsc::TryRecvError::Empty => {}
                    mpsc::TryRecvError::Disconnected => unreachable!(),
                },
            }

            if let Some(event) = self.event_poller.poll_once_timeout(timeout).or_fail()? {
                return Ok(event);
            }
        }
    }

    fn state_file_path(&self, name: &str) -> PathBuf {
        self.data_dir.join(urlencoding::encode(name).as_ref())
    }
}

impl System for AndroidSystem {
    fn video_draw(&mut self, frame: VideoFrame<&[u8]>) {
        let spec = frame.spec();
        if let Some(window) = &ndk_glue::native_window() {
            let window = Window::new(window);
            window.set_buffer_size(spec.resolution);

            if let Some(mut buffer) = window.acquire_buffer() {
                let stride = buffer.stride() as u32;
                let dst = buffer.as_slice_mut();
                let src = frame.data();
                if stride == spec.stride {
                    dst.copy_from_slice(src);
                } else {
                    let w = spec.resolution.width as usize;
                    for y in 0..spec.resolution.height {
                        let i = y as usize * stride as usize;
                        let j = y as usize * spec.stride as usize;
                        dst[i..i + w].copy_from_slice(&src[j..j + w]);
                    }
                }
            }
        }
    }

    fn video_frame_spec(&mut self, resolution: Size) -> VideoFrameSpec {
        let pixel_format = PixelFormat::Rgb24;

        let mut stride = resolution.width;
        if let Some(window) = &ndk_glue::native_window() {
            let window = Window::new(window);
            window.set_buffer_size(resolution);
            if let Some(buffer) = window.acquire_buffer() {
                stride = buffer.stride() as u32;
            }
        }

        VideoFrameSpec {
            pixel_format,
            resolution,
            stride,
        }
    }

    fn audio_enqueue(&mut self, data: AudioData) -> usize {
        let samples = data.samples().collect::<Vec<_>>();
        unsafe {
            let written = self
                .audio
                .write(samples.as_ptr() as *const _, samples.len() as i32, 0)
                .unwrap_or_else(|e| panic!("{e}"));

            let state = self.audio.get_state().unwrap_or_else(|e| panic!("{e}"));
            if !matches!(
                state,
                AudioStreamState::Started | AudioStreamState::Starting
            ) {
                self.audio
                    .request_start()
                    .unwrap_or_else(|e| panic!("{e} (current_state={state:?})"));
            }

            written as usize
        }
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
        let id = self.next_action_id.get_and_increment();
        let time = self.start.elapsed() + timeout;
        self.timeout_queue.push((Reverse(time), id));
        id
    }

    fn state_save(&mut self, name: &str, data: &[u8]) -> ActionId {
        let id = self.next_action_id.get_and_increment();
        let path = self.state_file_path(name);
        let data = data.to_owned();
        self.io_request_tx
            .send(IoRequest::Write { id, path, data })
            .unwrap_or_else(|_| panic!("I/O thread has terminated"));
        id
    }

    fn state_load(&mut self, name: &str) -> ActionId {
        let id = self.next_action_id.get_and_increment();
        let path = self.state_file_path(name);
        self.io_request_tx
            .send(IoRequest::Read { id, path })
            .unwrap_or_else(|_| panic!("I/O thread has terminated"));
        id
    }

    fn state_delete(&mut self, name: &str) -> ActionId {
        let id = self.next_action_id.get_and_increment();
        let path = self.state_file_path(name);
        self.io_request_tx
            .send(IoRequest::Delete { id, path })
            .unwrap_or_else(|_| panic!("I/O thread has terminated"));
        id
    }
}
