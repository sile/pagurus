use crate::event::EventPoller;
use crate::io_thread::{IoRequest, IoThread};
use crate::window::Window;
use pagurus::event::{Event, MouseEvent, TimeoutEvent, WindowEvent};
use pagurus::failure::OrFail;
use pagurus::spatial::Size;
use pagurus::{ActionId, AudioData, Result, System, VideoFrame};
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::sync::mpsc;
use std::{
    path::PathBuf,
    time::{Duration, Instant, UNIX_EPOCH},
};

#[derive(Debug)]
pub struct AndroidSystemBuilder {
    logical_window_size: Option<Size>,
}

impl AndroidSystemBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn logical_window_size(mut self, size: Option<Size>) -> Self {
        self.logical_window_size = size;
        self
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
        let io_request_tx = IoThread::spawn(event_tx.clone(), event_poller.notifier());

        let window_size = if let Some(window) = &*ndk_glue::native_window() {
            Window::new(window).get_window_size()
        } else {
            Size::default()
        };

        Ok(AndroidSystem {
            start: Instant::now(),
            event_poller,
            event_tx,
            event_rx,
            io_request_tx,
            timeout_queue: BinaryHeap::new(),
            next_action_id: ActionId::default(),
            data_dir,
            window_size,
            logical_window_size: self.logical_window_size,
        })
    }
}

impl Default for AndroidSystemBuilder {
    fn default() -> Self {
        Self {
            logical_window_size: None,
        }
    }
}

#[derive(Debug)]
pub struct AndroidSystem {
    start: Instant,
    next_action_id: ActionId,
    data_dir: PathBuf,
    event_poller: EventPoller,
    event_tx: mpsc::Sender<Event>,
    event_rx: mpsc::Receiver<Event>,
    io_request_tx: mpsc::Sender<IoRequest>,
    timeout_queue: BinaryHeap<(Reverse<Duration>, ActionId)>,
    window_size: Size,
    logical_window_size: Option<Size>,
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
                if let Some(event) = self.handle_event(event) {
                    return Ok(event);
                }
            }
        }
    }

    pub fn window_size(&self) -> Size {
        self.window_size
    }

    fn handle_event(&mut self, event: Event) -> Option<Event> {
        match event {
            Event::Window(WindowEvent::Resized { size }) => {
                self.window_size = size;
                Some(event)
            }
            Event::Mouse(event) => Some(Event::Mouse(self.adjust_mouse_position(event))),
            _ => Some(event),
        }
    }

    fn state_file_path(&self, name: &str) -> PathBuf {
        self.data_dir.join(urlencoding::encode(name).as_ref())
    }

    fn adjust_mouse_position(&self, event: MouseEvent) -> MouseEvent {
        let logical_window_size = if let Some(logical_window_size) = self.logical_window_size {
            logical_window_size
        } else {
            return event;
        };

        todo!()
    }

    // fn calc_window_buffer_region(&self, window: &Window) -> Region {

    // }
}

impl System for AndroidSystem {
    fn video_render(&mut self, frame: VideoFrame<&[u8]>) {
        // if let Some(window) = &*ndk_glue::native_window() {
        //     let window = Window::new(window);
        // }
    }

    fn audio_enqueue(&mut self, data: AudioData) -> usize {
        // TODO
        data.samples().count()
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
