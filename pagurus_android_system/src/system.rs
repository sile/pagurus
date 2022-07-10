use crate::io_thread::{IoRequest, IoThread};
use pagurus::event::{Event, TimeoutEvent};
use pagurus::failure::OrFail;
use pagurus::{ActionId, AudioData, Result, System, VideoFrame};
use std::sync::mpsc;
use std::{
    path::PathBuf,
    time::{Duration, Instant, UNIX_EPOCH},
};

#[derive(Debug)]
pub struct AndroidSystem {
    start: Instant,
    next_action_id: ActionId,
    data_dir: PathBuf,
    event_tx: mpsc::Sender<Event>,
    event_rx: mpsc::Receiver<Event>,
    io_request_tx: mpsc::Sender<IoRequest>,
}

impl AndroidSystem {
    pub fn new() -> Result<Self> {
        #[allow(deprecated)]
        let data_dir = PathBuf::from(
            ndk_glue::native_activity()
                .internal_data_path()
                .to_str()
                .or_fail()?,
        );

        let (event_tx, event_rx) = mpsc::channel();
        let io_request_tx = IoThread::spawn(event_tx.clone());
        Ok(Self {
            start: Instant::now(),
            event_tx,
            event_rx,
            io_request_tx,
            next_action_id: ActionId::default(),
            data_dir,
        })
    }

    pub fn next_event(&mut self) -> Option<Event> {
        match self.event_rx.try_recv() {
            Ok(event) => Some(event),
            Err(e) => match e {
                mpsc::TryRecvError::Empty => None,
                mpsc::TryRecvError::Disconnected => unreachable!(),
            },
        }
    }

    fn state_file_path(&self, name: &str) -> PathBuf {
        self.data_dir.join(urlencoding::encode(name).as_ref())
    }
}

impl System for AndroidSystem {
    fn video_render(&mut self, frame: VideoFrame<&[u8]>) {
        todo!()
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
        // TODO
        let id = self.next_action_id.get_and_increment();
        let _ = self.event_tx.send(Event::Timeout(TimeoutEvent { id }));
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
