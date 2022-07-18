use crate::io::{IoRequest, IoThread};
use image::{DynamicImage, Rgb, RgbImage};
use pagurus::event::{TimeoutEvent, WindowEvent};
use pagurus::failure::OrFail;
use pagurus::spatial::Size;
use pagurus::{event::Event, ActionId, AudioData, Result, System, VideoFrame};
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::{
    path::{Path, PathBuf},
    sync::mpsc,
    time::{Duration, Instant, UNIX_EPOCH},
};

pub struct TuiSystemBuilder {
    data_dir: PathBuf,
}

impl TuiSystemBuilder {
    pub fn new() -> Self {
        Self {
            data_dir: PathBuf::from(TuiSystem::DEFAULT_DATA_DIR),
        }
    }

    pub fn data_dir<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.data_dir = path.as_ref().to_path_buf();
        self
    }

    pub fn build(self) -> Result<TuiSystem> {
        // Event
        let (event_tx, event_rx) = mpsc::channel();
        let _ = event_tx.send(Event::Window(WindowEvent::RedrawNeeded {
            size: terminal_size(),
        }));

        // I/O Thread
        let io_request_tx = IoThread::spawn(event_tx);

        Ok(TuiSystem {
            start: Instant::now(),
            event_rx,
            io_request_tx,
            timeout_queue: BinaryHeap::new(),
            next_action_id: ActionId::default(),
            data_dir: self.data_dir,
        })
    }
}

impl Default for TuiSystemBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct TuiSystem {
    start: Instant,
    event_rx: mpsc::Receiver<Event>,
    io_request_tx: mpsc::Sender<IoRequest>,
    timeout_queue: BinaryHeap<(Reverse<Duration>, ActionId)>,
    next_action_id: ActionId,
    data_dir: PathBuf,
}

impl TuiSystem {
    pub const DEFAULT_DATA_DIR: &'static str = "data/";

    pub fn new() -> Result<Self> {
        TuiSystemBuilder::default().build().or_fail()
    }

    pub fn wait_event(&mut self) -> Event {
        loop {
            let timeout =
                if let Some((Reverse(expiry_time), id)) = self.timeout_queue.peek().copied() {
                    if let Some(timeout) = expiry_time.checked_sub(self.start.elapsed()) {
                        timeout
                    } else {
                        self.timeout_queue.pop();
                        return Event::Timeout(TimeoutEvent { id });
                    }
                } else {
                    Duration::from_secs(1) // Arbitrary large timeout value
                };

            match self.event_rx.recv_timeout(timeout) {
                Ok(event) => return event,
                Err(e) => match e {
                    mpsc::RecvTimeoutError::Timeout => {}
                    mpsc::RecvTimeoutError::Disconnected => {
                        unreachable!()
                    }
                },
            }
        }
    }

    fn state_file_path(&self, name: &str) -> PathBuf {
        self.data_dir.join(urlencoding::encode(name).as_ref())
    }
}

impl System for TuiSystem {
    fn video_draw(&mut self, frame: VideoFrame<&[u8]>) {
        let mut image = RgbImage::new(frame.size().width, frame.size().height);
        for (pos, (r, g, b)) in frame.r8g8b8_pixels() {
            image.put_pixel(pos.x as u32, pos.y as u32, Rgb([r, g, b]));
        }
        let image = DynamicImage::ImageRgb8(image);
        viuer::print(&image, &Default::default()).unwrap_or_else(|e| panic!("{e}"));
    }

    fn audio_enqueue(&mut self, data: AudioData) -> usize {
        // Just ignored as the audio feature is not supported by this system.
        data.samples().count()
    }

    fn console_log(&mut self, message: &str) {
        eprintln!("{message}");
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

fn terminal_size() -> Size {
    let (w, h) = viuer::terminal_size();
    Size::from_wh(u32::from(w), u32::from(h))
}
