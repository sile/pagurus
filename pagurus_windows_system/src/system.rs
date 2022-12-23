use crate::{
    audio::AudioPlayer,
    window::{Window, WindowBuilder},
};
use pagurus::{
    audio::{AudioData, AudioSpec, SampleFormat},
    event::{Event, StateEvent, TimeoutEvent},
    failure::{Failure, OrFail},
    spatial::Size,
    timeout::{TimeoutId, TimeoutTag},
    video::{PixelFormat, VideoFrame, VideoFrameSpec},
    ActionId, Result, System,
};
use std::{
    cmp::Reverse,
    collections::BinaryHeap,
    path::{Path, PathBuf},
    sync::mpsc,
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

    pub fn data_dir<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.data_dir = path.as_ref().to_path_buf();
        self
    }

    pub fn build(self) -> Result<WindowsSystem> {
        let window = self.window.build().or_fail()?;
        let io_request_tx = IoThread::spawn(window.event_tx());
        Ok(WindowsSystem {
            window,
            audio_player: None,
            start: Instant::now(),
            timeout_queue: BinaryHeap::new(),
            io_request_tx,
            next_action_id: ActionId::new(0),
            next_timeout_id: TimeoutId::new(),
            data_dir: self.data_dir,
        })
    }
}

#[derive(Debug)]
pub struct WindowsSystem {
    window: Window,
    audio_player: Option<AudioPlayer>,
    start: Instant,
    timeout_queue: BinaryHeap<Reverse<(Instant, TimeoutEvent)>>,
    io_request_tx: mpsc::Sender<IoRequest>,
    next_action_id: ActionId,
    next_timeout_id: TimeoutId,
    data_dir: PathBuf,
}

impl WindowsSystem {
    pub const DEFAULT_DATA_DIR: &'static str = "data/";

    pub fn next_event(&mut self) -> Event {
        loop {
            if let Some(&Reverse((timeout, event))) = self.timeout_queue.peek() {
                if timeout <= Instant::now() {
                    self.timeout_queue.pop();
                    return Event::Timeout(event);
                }
            }

            let timeout = self.timeout_queue.peek().map(|x| x.0 .0);
            if let Some(event) = self.window.next_event(timeout) {
                return event;
            }
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    fn state_file_path(&self, name: &str) -> PathBuf {
        self.data_dir.join(urlencoding::encode(name).as_ref())
    }
}

impl System for WindowsSystem {
    fn video_init(&mut self, resolution: Size) -> VideoFrameSpec {
        let w = resolution.width;
        let stride = w + (4 - (w % 4)) % 4;
        VideoFrameSpec {
            pixel_format: PixelFormat::Bgr24,
            resolution,
            stride,
        }
    }

    fn video_draw(&mut self, frame: VideoFrame<&[u8]>) {
        self.window
            .draw_video_frame(frame)
            .unwrap_or_else(|e| panic!("{e}"));
    }

    fn audio_init(&mut self, sample_rate: u16, data_samples: usize) -> AudioSpec {
        let player = AudioPlayer::new(sample_rate, data_samples)
            .unwrap_or_else(|e| panic!("failed to initialize audio: {e}"));
        self.audio_player = Some(player);
        AudioSpec {
            sample_format: SampleFormat::I16Be,
            sample_rate,
            data_samples,
        }
    }

    fn audio_enqueue(&mut self, data: AudioData<&[u8]>) {
        if let Some(player) = &mut self.audio_player {
            player.play(data).unwrap_or_else(|e| panic!("{e}"));
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
        self.timeout_queue.push(Reverse((
            Instant::now() + timeout,
            TimeoutEvent { tag, id },
        )));
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

struct IoThread {
    request_rx: mpsc::Receiver<IoRequest>,
    event_tx: mpsc::Sender<Event>,
}

impl IoThread {
    fn spawn(event_tx: mpsc::Sender<Event>) -> mpsc::Sender<IoRequest> {
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
        let _ = self.event_tx.send(event);
    }

    fn handle_read(&mut self, id: ActionId, path: PathBuf) {
        let (data, failed) = match std::fs::read(path) {
            Ok(data) => (Some(data), None),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (None, None),
            Err(e) => (None, Some(Failure::new().message(e.to_string()))),
        };
        let event = Event::State(StateEvent::Loaded { id, data, failed });
        let _ = self.event_tx.send(event);
    }

    fn handle_delete(&mut self, id: ActionId, path: PathBuf) {
        let failed = std::fs::remove_file(path).err().and_then(|e| {
            (e.kind() != std::io::ErrorKind::NotFound)
                .then(|| Failure::new().message(e.to_string()))
        });
        let event = Event::State(StateEvent::Deleted { id, failed });
        let _ = self.event_tx.send(event);
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
