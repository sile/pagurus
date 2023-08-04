use pagurus::{
    audio::{AudioData, AudioSpec, SampleFormat},
    event::{Event, Key, KeyEvent, MouseButton, MouseEvent, TimeoutTag},
    failure::{Failure, OrFail},
    spatial::{Position, Size},
    video::{PixelFormat, VideoFrame, VideoFrameSpec},
    System,
};
use std::{
    cmp::Reverse,
    collections::{BTreeMap, BinaryHeap},
    io::{Stdout, Write},
    sync::mpsc,
    time::{Duration, Instant, UNIX_EPOCH},
};
use termion::{
    color::{Bg, Fg, Rgb},
    cursor::HideCursor,
    input::{MouseTerminal, TermRead},
    raw::{IntoRawMode, RawTerminal},
};

pub struct TuiSystem {
    start_time: Instant,
    event_queue: mpsc::Receiver<Event>,
    event_sender: mpsc::Sender<Event>,
    timeout_queue: BinaryHeap<Reverse<(Duration, TimeoutTag)>>,
    stdout: HideCursor<MouseTerminal<RawTerminal<Stdout>>>,
    dirty_pixels: BTreeMap<DirtyPixelsKey, UpperLowerPixels>,
    frame_buffer: FrameBuffer,
    failed: Option<Failure>,
}

impl TuiSystem {
    pub fn new() -> pagurus::Result<Self> {
        if !termion::is_tty(&std::io::stdout()) {
            return Err(Failure::new().message("Not a TTY"));
        }

        let mut stdout = HideCursor::from(MouseTerminal::from(
            std::io::stdout().into_raw_mode().or_fail()?,
        ));
        write!(stdout, "{}", termion::clear::All).or_fail()?;
        stdout.flush().or_fail()?;

        let terminal_size = Self::terminal_size().or_fail()?;

        let (tx, rx) = mpsc::channel();
        let _ = tx.send(Event::WindowResized(terminal_size));
        let event_sender = tx.clone();
        std::thread::spawn(move || listen_input_events(tx));

        Ok(Self {
            start_time: Instant::now(),
            event_queue: rx,
            event_sender,
            timeout_queue: BinaryHeap::new(),
            stdout,
            dirty_pixels: BTreeMap::new(),
            frame_buffer: FrameBuffer::default(),
            failed: None,
        })
    }

    fn terminal_size() -> pagurus::Result<Size> {
        termion::terminal_size()
            .map(|(w, h)| Size::from_wh(w as u32, h as u32 * 2))
            .or_fail()
    }

    pub fn next_event(&mut self) -> pagurus::Result<Event> {
        if let Some(e) = self.failed.take() {
            return Err(e);
        }

        if let Some(Reverse((expire_time, tag))) = self.timeout_queue.peek().copied() {
            let now = self.clock_game_time();
            if let Some(timeout) = expire_time.checked_sub(now) {
                if let Ok(event) = self.event_queue.recv_timeout(timeout) {
                    return Ok(event);
                }
            }
            self.timeout_queue.pop();
            Ok(Event::Timeout(tag))
        } else {
            self.event_queue.recv().or_fail()
        }
    }

    fn resize_frame_buffer(&mut self, size: Size) -> pagurus::Result<()> {
        self.frame_buffer.resize(size);
        self.event_sender
            .send(Event::WindowResized(size))
            .or_fail()?;
        write!(self.stdout, "{}", termion::clear::All).or_fail()?;
        Ok(())
    }

    fn draw(&mut self, frame: VideoFrame<&[u8]>) -> pagurus::Result<()> {
        let terminal_size = Self::terminal_size().or_fail()?;

        if self.frame_buffer.size != terminal_size {
            self.resize_frame_buffer(terminal_size).or_fail()?;
            return Ok(());
        }

        self.draw_to_buffer(frame);
        self.draw_to_terminal().or_fail()?;

        Ok(())
    }

    fn draw_to_buffer(&mut self, frame: VideoFrame<&[u8]>) {
        let width = self
            .frame_buffer
            .size
            .width
            .min(frame.spec().resolution.width);
        let height = self
            .frame_buffer
            .size
            .height
            .min(frame.spec().resolution.height);

        for y in 0..height {
            let event_y = y / 2 * 2;
            for x in 0..width {
                let position = Position::from_xy(x as i32, y as i32);
                let old_rgb = self.frame_buffer.get_rgb(position);
                let new_rgb = frame.read_rgb(position);
                let new_rgb = Rgb(new_rgb.0, new_rgb.1, new_rgb.2);
                self.frame_buffer.set_rgb(position, new_rgb);

                if self.frame_buffer.initialized && old_rgb == new_rgb {
                    continue;
                }

                let bg_position = Position::from_xy(x as i32, event_y as i32);
                let fg_position = bg_position.move_y(1);
                self.dirty_pixels.insert(
                    DirtyPixelsKey(bg_position),
                    UpperLowerPixels {
                        upper: self.frame_buffer.get_rgb(bg_position),
                        lower: self.frame_buffer.get_rgb(fg_position),
                    },
                );
            }
        }

        self.frame_buffer.initialized = true;
    }

    fn draw_to_terminal(&mut self) -> pagurus::Result<()> {
        fn goto(position: Position) -> termion::cursor::Goto {
            termion::cursor::Goto(position.x as u16 + 1, position.y as u16 / 2 + 1)
        }

        let mut last_position = Position::ORIGIN;
        write!(self.stdout, "{}", goto(last_position)).or_fail()?;
        for (DirtyPixelsKey(position), pixels) in std::mem::take(&mut self.dirty_pixels) {
            if last_position.y == position.y && last_position.x + 1 == position.x {
                write!(self.stdout, "{}", pixels).or_fail()?;
            } else {
                write!(self.stdout, "{}{}", goto(position), pixels).or_fail()?;
            }
            last_position = position;
        }
        self.stdout.flush().or_fail()?;

        Ok(())
    }
}

impl std::fmt::Debug for TuiSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("TuiSystem").finish()
    }
}

impl System for TuiSystem {
    fn video_init(&mut self, resolution: Size) -> VideoFrameSpec {
        VideoFrameSpec {
            pixel_format: PixelFormat::Rgb24,
            resolution,
            stride: resolution.width,
        }
    }

    fn video_draw(&mut self, frame: VideoFrame<&[u8]>) {
        if self.failed.is_some() {
            return;
        }

        self.failed = self.draw(frame).err();
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
                .convert_to_pagurus_mouse_event(v)
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
        termion::event::Key::Char('\n') => Box::new(key(Key::Return)),
        termion::event::Key::Char('\t') => Box::new(key(Key::Tab)),
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
    fn convert_to_pagurus_mouse_event(
        &mut self,
        v: termion::event::MouseEvent,
    ) -> Option<MouseEvent> {
        fn position(x: u16, y: u16) -> Position {
            Position::from_xy(x as i32 - 1, (y as i32 - 1) * 2)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DirtyPixelsKey(Position);

impl PartialOrd for DirtyPixelsKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DirtyPixelsKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.0.y, self.0.x).cmp(&(other.0.y, other.0.x))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct UpperLowerPixels {
    upper: Rgb,
    lower: Rgb,
}

impl std::fmt::Display for UpperLowerPixels {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}\u{2584}", Bg(self.upper), Fg(self.lower))
    }
}

#[derive(Debug, Default)]
struct FrameBuffer {
    pixels: Vec<Rgb>,
    size: Size,
    initialized: bool,
}

impl FrameBuffer {
    fn resize(&mut self, size: Size) {
        self.size = size;
        self.pixels = vec![Rgb(0, 0, 0); size.len()];
        self.initialized = false;
    }

    fn get_rgb(&self, position: Position) -> Rgb {
        let i = position.y as u32 * self.size.width + position.x as u32;
        self.pixels[i as usize]
    }

    fn set_rgb(&mut self, position: Position, rgb: Rgb) {
        let i = position.y as u32 * self.size.width + position.x as u32;
        self.pixels[i as usize] = rgb;
    }
}
