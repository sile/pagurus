use log::{Level, Log, Metadata, Record, SetLoggerError};
use pagurus::System;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Logger {
    rx: Receiver<String>,
}

impl Logger {
    pub fn init(level: Level) -> Result<Logger, SetLoggerError> {
        let (tx, rx) = mpsc::channel();
        let tx = LogSender {
            level,
            tx: Arc::new(Mutex::new(tx)),
        };
        log::set_boxed_logger(Box::new(tx))
            .map(|()| log::set_max_level(level.to_level_filter()))?;
        Ok(Self { rx })
    }

    pub fn null() -> Self {
        let (_tx, rx) = mpsc::channel();
        Self { rx }
    }

    pub fn flush<S: System>(&self, system: &mut S) {
        for msg in self.rx.try_iter() {
            system.console_log(&msg);
        }
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::null()
    }
}

#[derive(Debug)]
struct LogSender {
    level: Level,
    tx: Arc<Mutex<Sender<String>>>,
}

impl Log for LogSender {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        if let Ok(tx) = self.tx.lock() {
            let msg = format!(
                "[{}] [{}:{}] {}",
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            );
            let _ = tx.send(msg);
        }
    }

    fn flush(&self) {}
}
