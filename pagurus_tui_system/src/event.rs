use pagurus::{
    event::{Event, KeyEvent},
    failure::OrFail,
    input::Key,
    Result,
};
use std::{io::Read, sync::mpsc};
use termion::input::TermRead;
use termion::raw::IntoRawMode;

pub struct TerminalEventPoller {
    event_tx: mpsc::Sender<Event>,
    _raw_mode: termion::raw::RawTerminal<std::io::Stdout>,
}

impl TerminalEventPoller {
    pub fn spawn(event_tx: mpsc::Sender<Event>) -> Result<()> {
        let raw_mode = std::io::stdout().into_raw_mode().or_fail()?;
        std::thread::spawn(move || {
            Self {
                event_tx,
                _raw_mode: raw_mode,
            }
            .run()
        });
        Ok(())
    }

    fn run(mut self) {
        let stdin = std::io::stdin();
        let mut stdin = stdin.lock();
        loop {
            self.run_once(&mut stdin)
        }
    }

    fn run_once(&mut self, stdin: &mut impl Read) {
        for key in stdin.keys() {
            let key = key.unwrap_or_else(|e| panic!("{e}"));
            if let termion::event::Key::Esc = key {
                let _ = self.event_tx.send(Event::Terminating);
            } else if let Some(key) = to_pagurus_key(key) {
                let _ = self.event_tx.send(Event::Key(KeyEvent::Down { key }));
                let _ = self.event_tx.send(Event::Key(KeyEvent::Up { key }));
            }
        }
    }
}

fn to_pagurus_key(key: termion::event::Key) -> Option<Key> {
    match key {
        termion::event::Key::Backspace => Some(Key::Backspace),
        termion::event::Key::Left => Some(Key::Left),
        termion::event::Key::Right => Some(Key::Right),
        termion::event::Key::Up => Some(Key::Up),
        termion::event::Key::Down => Some(Key::Down),
        termion::event::Key::Delete => Some(Key::Delete),
        termion::event::Key::Alt(_) => Some(Key::Alt),
        termion::event::Key::Ctrl(_) => Some(Key::Ctrl),
        termion::event::Key::Char('\n') => Some(Key::Return),
        _ => None,
    }
}
