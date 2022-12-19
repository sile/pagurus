use crate::event::EventNotifier;
use pagurus::event::{Event, StateEvent};
use pagurus::failure::{Failure, OrFail};
use pagurus::ActionId;
use std::path::PathBuf;
use std::sync::mpsc;

#[derive(Debug)]
pub struct IoThread {
    request_rx: mpsc::Receiver<IoRequest>,
    event_tx: mpsc::Sender<Event>,
    notifier: EventNotifier,
}

impl IoThread {
    pub fn spawn(
        event_tx: mpsc::Sender<Event>,
        notifier: EventNotifier,
    ) -> mpsc::Sender<IoRequest> {
        let (request_tx, request_rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut this = Self {
                request_rx,
                event_tx,
                notifier,
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
        self.notifier.notify();
    }

    fn handle_read(&mut self, id: ActionId, path: PathBuf) {
        let (data, failed) = match std::fs::read(path) {
            Ok(data) => (Some(data), None),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (None, None),
            Err(e) => (None, Some(Failure::new().message(e.to_string()))),
        };
        let event = Event::State(StateEvent::Loaded { id, data, failed });
        let _ = self.event_tx.send(event);
        self.notifier.notify();
    }

    fn handle_delete(&mut self, id: ActionId, path: PathBuf) {
        let failed = std::fs::remove_file(path).err().and_then(|e| {
            (e.kind() != std::io::ErrorKind::NotFound)
                .then(|| Failure::new().message(e.to_string()))
        });
        let event = Event::State(StateEvent::Deleted { id, failed });
        let _ = self.event_tx.send(event);
        self.notifier.notify();
    }
}

#[derive(Debug)]
pub enum IoRequest {
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
