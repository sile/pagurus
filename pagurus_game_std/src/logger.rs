use log::{Level, Log, Metadata, Record, SetLoggerError};
use pagurus::System;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Logger<S> {
    level: Level,
    _system: PhantomData<S>,
}

unsafe impl<S> Send for Logger<S> {}

unsafe impl<S> Sync for Logger<S> {}

impl<S: System + 'static> Logger<S> {
    pub fn init(level: Level) -> Result<(), SetLoggerError> {
        let logger = Self {
            level,
            _system: PhantomData,
        };
        log::set_boxed_logger(Box::new(logger))
            .map(|()| log::set_max_level(level.to_level_filter()))?;
        Ok(())
    }
}

impl<S: System> Log for Logger<S> {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let msg = format!(
            "[{}] [{}:{}] {}",
            record.level(),
            record.file().unwrap_or("unknown"),
            record.line().unwrap_or(0),
            record.args()
        );
        S::console_log(&msg);
    }

    fn flush(&self) {}
}
