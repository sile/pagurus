use pagurus::audio::{AudioSpec, SampleFormat};
use pagurus::event::{Event, StateEvent, TimeoutEvent, WindowEvent};
use pagurus::failure::{Failure, OrFail};
use pagurus::spatial::Size;
use pagurus::timeout::{TimeoutId, TimeoutTag};
use pagurus::video::{PixelFormat, VideoFrameSpec};
use pagurus::{audio::AudioData, video::VideoFrame, ActionId, Result, System};
use sdl2::audio::{AudioQueue, AudioSpecDesired};
use sdl2::event::EventSender;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::{EventPump, Sdl, VideoSubsystem};
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant, UNIX_EPOCH};

type CustomWindowFn = Box<dyn 'static + Fn(VideoSubsystem) -> Result<Window>>;
type CustomCanvasFn = Box<dyn 'static + Fn(Window) -> Result<Canvas<Window>>>;

pub struct SdlSystemBuilder {
    data_dir: PathBuf,
    title: String,
    window_size: Option<Size>,
    custom_window: Option<CustomWindowFn>,
    custom_canvas: Option<CustomCanvasFn>,
}

impl SdlSystemBuilder {
    pub fn new() -> Self {
        Self {
            data_dir: PathBuf::from(SdlSystem::DEFAULT_DATA_DIR),
            title: SdlSystem::DEFAULT_TITLE.to_owned(),
            window_size: None,
            custom_window: None,
            custom_canvas: None,
        }
    }

    pub fn data_dir<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.data_dir = path.as_ref().to_path_buf();
        self
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_owned();
        self
    }

    pub fn window_size(mut self, size: Option<Size>) -> Self {
        self.window_size = size;
        self
    }

    pub fn build(self) -> Result<SdlSystem> {
        let sdl_context = sdl2::init().map_err(|e| Failure::new().message(e))?;

        // Video
        let sdl_video = sdl_context.video().map_err(|e| Failure::new().message(e))?;
        let sdl_window = if let Some(f) = self.custom_window {
            f(sdl_video).or_fail()?
        } else {
            let window_size = self.window_size.unwrap_or(SdlSystem::DEFAULT_WINDOW_SIZE);
            sdl_video
                .window(&self.title, window_size.width, window_size.height)
                .position_centered()
                .build()
                .or_fail()?
        };
        let sdl_canvas = if let Some(f) = self.custom_canvas {
            f(sdl_window).or_fail()?
        } else {
            sdl_window.into_canvas().build().or_fail()?
        };

        // Event
        let sdl_event = sdl_context.event().map_err(|e| Failure::new().message(e))?;
        sdl_event
            .register_custom_event::<Event>()
            .map_err(|e| Failure::new().message(e))?;
        let sdl_event_pump = sdl_context
            .event_pump()
            .map_err(|e| Failure::new().message(e))?;

        let (width, height) = sdl_canvas.window().size();
        sdl_event
            .event_sender()
            .push_custom_event(Event::Window(WindowEvent::RedrawNeeded {
                size: Size::from_wh(width, height),
            }))
            .map_err(|e| Failure::new().message(e))?;

        // I/O Thread
        let io_request_tx = IoThread::spawn(sdl_event.event_sender());

        Ok(SdlSystem {
            sdl_context,
            sdl_canvas,
            sdl_event_pump,
            sdl_audio_queue: None,
            io_request_tx,
            start: Instant::now(),
            timeout_queue: BinaryHeap::new(),
            next_action_id: ActionId::default(),
            next_timeout_id: TimeoutId::new(),
            data_dir: self.data_dir,
        })
    }
}

impl Default for SdlSystemBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SdlSystem {
    sdl_context: Sdl,
    sdl_canvas: Canvas<Window>,
    sdl_event_pump: EventPump,
    sdl_audio_queue: Option<AudioQueue<i16>>,
    io_request_tx: mpsc::Sender<IoRequest>,
    start: Instant,
    timeout_queue: BinaryHeap<(Reverse<Duration>, TimeoutEvent)>,
    next_action_id: ActionId,
    next_timeout_id: TimeoutId,
    data_dir: PathBuf,
}

impl SdlSystem {
    pub const DEFAULT_TITLE: &'static str = "Pagurus";
    pub const DEFAULT_WINDOW_SIZE: Size = Size::from_wh(800, 600);
    pub const DEFAULT_DATA_DIR: &'static str = "data/";

    pub fn new() -> Result<Self> {
        SdlSystemBuilder::default().build().or_fail()
    }

    pub fn window_size(&self) -> Size {
        let (width, height) = self.sdl_canvas.window().size();
        Size { width, height }
    }

    pub fn wait_event(&mut self) -> Event {
        loop {
            let timeout =
                if let Some((Reverse(expiry_time), event)) = self.timeout_queue.peek().copied() {
                    if let Some(timeout) = expiry_time.checked_sub(self.start.elapsed()) {
                        timeout
                    } else {
                        self.timeout_queue.pop();
                        return Event::Timeout(event);
                    }
                } else {
                    Duration::from_secs(1) // Arbitrary large timeout value
                };

            let event = self
                .sdl_event_pump
                .wait_event_timeout(timeout.as_millis() as u32)
                .and_then(crate::event::to_pagurus_event);
            if let Some(event) = event {
                return event;
            }
        }
    }

    fn state_file_path(&self, name: &str) -> PathBuf {
        self.data_dir.join(urlencoding::encode(name).as_ref())
    }
}

impl System for SdlSystem {
    fn video_init(&mut self, resolution: Size) -> VideoFrameSpec {
        VideoFrameSpec {
            pixel_format: PixelFormat::Rgb24,
            resolution,
            stride: resolution.width,
        }
    }

    fn video_draw(&mut self, frame: VideoFrame<&[u8]>) {
        self.sdl_canvas.clear();

        let resolution = frame.spec().resolution;
        let texture_creator = self.sdl_canvas.texture_creator();
        let mut texture = texture_creator
            .create_texture_static(
                Some(PixelFormatEnum::RGB24),
                resolution.width,
                resolution.height,
            )
            .unwrap_or_else(|e| panic!("failed to create a texture: {e}"));
        texture
            .update(None, frame.data(), resolution.width as usize * 3)
            .unwrap_or_else(|e| panic!("failed to update texture: {e}"));

        self.sdl_canvas
            .copy(&texture, None, None)
            .unwrap_or_else(|e| panic!("failed to copy texture to canvas: {e}"));

        self.sdl_canvas.present();
    }

    fn audio_init(&mut self, sample_rate: u16, data_samples: usize) -> AudioSpec {
        let data_samples = std::cmp::min(data_samples, u16::MAX as usize);
        let sdl_audio = self
            .sdl_context
            .audio()
            .unwrap_or_else(|e| panic!("failed to initialize audio: {e}"));
        let audio_spec = AudioSpecDesired {
            freq: Some(i32::from(sample_rate)),
            channels: Some(AudioSpec::CHANNELS),
            samples: Some(data_samples as u16),
        };
        let sdl_audio_queue = sdl_audio
            .open_queue(None, &audio_spec)
            .unwrap_or_else(|e| panic!("failed to initialize audio: {e}"));
        sdl_audio_queue.resume();
        self.sdl_audio_queue = Some(sdl_audio_queue);
        AudioSpec {
            sample_format: SampleFormat::F32Le,
            sample_rate,
            data_samples,
        }
    }

    fn audio_enqueue(&mut self, data: AudioData<&[u8]>) {
        if let Some(audio) = &mut self.sdl_audio_queue {
            let samples = data.samples().map(|s| s.to_i16()).collect::<Vec<_>>();
            audio
                .queue_audio(&samples)
                .unwrap_or_else(|e| panic!("failed to queue audio data: {e}"));
        }
    }

    fn console_log(message: &str) {
        eprintln!("{message}");
    }

    fn clock_game_time(&self) -> Duration {
        self.start.elapsed()
    }

    fn clock_unix_time(&self) -> Duration {
        UNIX_EPOCH
            .elapsed()
            .unwrap_or_else(|e| panic!("failed to get UNIX timestamp: {e}"))
    }

    fn clock_set_timeout(&mut self, tag: TimeoutTag, timeout: Duration) -> TimeoutId {
        let id = self.next_timeout_id.increment();
        let time = self.start.elapsed() + timeout;
        self.timeout_queue
            .push((Reverse(time), TimeoutEvent { tag, id }));
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

impl std::fmt::Debug for SdlSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "SdlSystem {{ .. }}")
    }
}

struct IoThread {
    request_rx: mpsc::Receiver<IoRequest>,
    event_tx: EventSender,
}

impl IoThread {
    fn spawn(event_tx: EventSender) -> mpsc::Sender<IoRequest> {
        let (request_tx, request_rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut this = Self {
                request_rx,
                event_tx,
            };
            while this.run_once() {}
        });
        request_tx
    }

    fn run_once(&mut self) -> bool {
        match self.request_rx.recv() {
            Ok(IoRequest::Write { id, path, data }) => {
                self.handle_write(id, path, data);
            }
            Ok(IoRequest::Read { id, path }) => {
                self.handle_read(id, path);
            }
            Ok(IoRequest::Delete { id, path }) => {
                self.handle_delete(id, path);
            }
            Err(_) => return false,
        }
        true
    }

    fn handle_write(&mut self, id: ActionId, path: PathBuf, data: Vec<u8>) {
        let failed = (|| {
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir).or_fail()?;
            }
            std::fs::write(path, &data).or_fail()?;
            Ok(())
        })()
        .err();
        let event = Event::State(StateEvent::Saved { id, failed });
        self.event_tx
            .push_custom_event(event)
            .unwrap_or_else(|e| panic!("failed to send custom SDL event: {e}"));
    }

    fn handle_read(&mut self, id: ActionId, path: PathBuf) {
        let (data, failed) = match std::fs::read(path) {
            Ok(data) => (Some(data), None),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (None, None),
            Err(e) => (None, Some(Failure::new().message(e.to_string()))),
        };
        let event = Event::State(StateEvent::Loaded { id, data, failed });
        self.event_tx
            .push_custom_event(event)
            .unwrap_or_else(|e| panic!("failed to send custom SDL event: {e}"));
    }

    fn handle_delete(&mut self, id: ActionId, path: PathBuf) {
        let failed = std::fs::remove_file(path).err().and_then(|e| {
            (e.kind() != std::io::ErrorKind::NotFound)
                .then(|| Failure::new().message(e.to_string()))
        });
        let event = Event::State(StateEvent::Deleted { id, failed });
        self.event_tx
            .push_custom_event(event)
            .unwrap_or_else(|e| panic!("failed to send custom SDL event: {e}"));
    }
}

#[derive(Debug)]
enum IoRequest {
    Write {
        id: ActionId,
        path: PathBuf,
        data: Vec<u8>,
    },
    Read {
        id: ActionId,
        path: PathBuf,
    },
    Delete {
        id: ActionId,
        path: PathBuf,
    },
}
