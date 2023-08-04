use pagurus::{
    audio::{AudioData, AudioSpec, SampleFormat},
    event::{Event, Key, KeyEvent, MouseButton, MouseEvent, TimeoutTag},
    spatial::{Position, Size},
    video::{VideoFrame, VideoFrameSpec},
    System,
};
use std::{
    cmp::Reverse,
    collections::BinaryHeap,
    sync::mpsc,
    time::{Duration, Instant, UNIX_EPOCH},
};
use termion::input::TermRead;

#[derive(Debug)]
pub struct TuiSystem {
    start_time: Instant,
    event_queue: mpsc::Receiver<Event>,
    timeout_queue: BinaryHeap<Reverse<(Duration, TimeoutTag)>>,
}

impl TuiSystem {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || listen_input_events(tx));

        Self {
            start_time: Instant::now(),
            event_queue: rx,
            timeout_queue: BinaryHeap::new(),
        }
    }

    pub fn next_event(&mut self) -> Option<Event> {
        if let Some(Reverse((expire_time, tag))) = self.timeout_queue.peek().copied() {
            let now = self.clock_game_time();
            if let Some(timeout) = expire_time.checked_sub(now) {
                if let Some(event) = self.event_queue.recv_timeout(timeout).ok() {
                    return Some(event);
                }
            }
            self.timeout_queue.pop();
            return Some(Event::Timeout(tag));
        } else {
            self.event_queue.recv().ok()
        }
    }
}

impl System for TuiSystem {
    fn video_init(&mut self, resolution: Size) -> VideoFrameSpec {
        todo!()
    }

    fn video_draw(&mut self, frame: VideoFrame<&[u8]>) {
        todo!()
    }

    fn audio_init(&mut self, sample_rate: u16, data_samples: usize) -> AudioSpec {
        // Returns dummy spec.
        AudioSpec {
            sample_format: SampleFormat::I16Be,
            sample_rate,
            data_samples,
        }
    }

    fn audio_enqueue(&mut self, _data: AudioData<&[u8]>) {
        // Discards audio data as TUI does not support audio.
    }

    fn clock_game_time(&self) -> Duration {
        self.start_time.elapsed()
    }

    fn clock_unix_time(&self) -> Duration {
        UNIX_EPOCH.elapsed().expect("cannot get UNIX time")
    }

    fn clock_set_timeout(&mut self, tag: TimeoutTag, timeout: Duration) {
        let now = self.clock_game_time();
        self.timeout_queue.push(Reverse((now + timeout, tag)));
    }
}

fn listen_input_events(tx: mpsc::Sender<Event>) {
    let mut mouse_state = MouseState::default();
    for event in std::io::stdin().events() {
        let Ok(event) = event else {
            break;
        };

        for event in to_pagurus_events(&mut mouse_state, event) {
            if tx.send(event).is_err() {
                break;
            }
        }
    }
}

fn to_pagurus_events(
    mouse_state: &mut MouseState,
    v: termion::event::Event,
) -> Box<dyn Iterator<Item = Event>> {
    match v {
        termion::event::Event::Key(v) => Box::new(to_pagurus_key_events(v).map(Event::Key)),
        termion::event::Event::Mouse(v) => Box::new(
            mouse_state
                .to_pagurus_mouse_event(v)
                .map(Event::Mouse)
                .into_iter(),
        ),
        termion::event::Event::Unsupported(_) => Box::new(std::iter::empty()),
    }
}

fn to_pagurus_key_events(v: termion::event::Key) -> Box<dyn Iterator<Item = KeyEvent>> {
    fn key(key: Key) -> impl Iterator<Item = KeyEvent> {
        std::iter::once(KeyEvent::Down { key }).chain(std::iter::once(KeyEvent::Up { key }))
    }

    fn with(modifier: Key, c: char) -> impl Iterator<Item = KeyEvent> {
        std::iter::once(KeyEvent::Down { key: modifier })
            .chain(std::iter::once(KeyEvent::Down { key: Key::Char(c) }))
            .chain(std::iter::once(KeyEvent::Up { key: Key::Char(c) }))
            .chain(std::iter::once(KeyEvent::Up { key: modifier }))
    }

    match v {
        termion::event::Key::Backspace => Box::new(key(Key::Backspace)),
        termion::event::Key::Left => Box::new(key(Key::Left)),
        termion::event::Key::Right => Box::new(key(Key::Right)),
        termion::event::Key::Up => Box::new(key(Key::Up)),
        termion::event::Key::Down => Box::new(key(Key::Down)),
        termion::event::Key::Delete => Box::new(key(Key::Delete)),
        termion::event::Key::Esc => Box::new(key(Key::Esc)),
        termion::event::Key::Char(c) => Box::new(key(Key::Char(c))),
        termion::event::Key::Ctrl(c) => Box::new(with(Key::Ctrl, c)),
        termion::event::Key::Alt(c) => Box::new(with(Key::Alt, c)),
        _ => Box::new(std::iter::empty()),
    }
}

#[derive(Debug, Default)]
struct MouseState {
    pressed_button: Option<MouseButton>,
}

impl MouseState {
    fn to_pagurus_mouse_event(&mut self, v: termion::event::MouseEvent) -> Option<MouseEvent> {
        fn position(x: u16, y: u16) -> Position {
            Position::from_xy(x as i32 - 1, y as i32 - 1)
        }

        match v {
            termion::event::MouseEvent::Press(button, x, y) => {
                let button = match button {
                    termion::event::MouseButton::Left => Some(MouseButton::Left),
                    termion::event::MouseButton::Right => Some(MouseButton::Right),
                    termion::event::MouseButton::Middle => Some(MouseButton::Middle),
                    _ => None,
                }?;
                self.pressed_button = Some(button);
                Some(MouseEvent::Down {
                    button,
                    position: position(x, y),
                })
            }
            termion::event::MouseEvent::Release(x, y) => {
                self.pressed_button.take().map(|button| MouseEvent::Up {
                    button,
                    position: position(x, y),
                })
            }
            termion::event::MouseEvent::Hold(x, y) => Some(MouseEvent::Move {
                position: position(x, y),
            }),
        }
    }
}
