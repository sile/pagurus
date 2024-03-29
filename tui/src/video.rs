use crate::TuiSystemOptions;
use orfail::{Failure, OrFail};
use pagurus::{
    event::{Event, Key, KeyEvent, MouseEvent},
    spatial::{Position, Size},
    video::VideoFrame,
};
use std::{collections::BTreeMap, io::Write, sync::mpsc};
use termion::{
    color::{Bg, Fg, Rgb},
    cursor::HideCursor,
    input::{MouseTerminal, TermRead},
    raw::IntoRawMode,
    screen::IntoAlternateScreen,
};

pub struct VideoSystem {
    event_queue: mpsc::Receiver<Event>,
    event_sender: mpsc::Sender<Event>,
    dirty_pixels: BTreeMap<DirtyPixelsKey, UpperLowerPixels>,
    frame_buffer: FrameBuffer,
    stdout: Box<dyn 'static + Write>,
}

impl VideoSystem {
    pub fn new(options: TuiSystemOptions) -> orfail::Result<Self> {
        if !termion::is_tty(&std::io::stdout()) {
            return Err(Failure::new("Not a TTY"));
        }

        let mut stdout: Box<dyn 'static + Write> =
            Box::new(std::io::stdout().into_raw_mode().or_fail()?);
        if !options.disable_mouse {
            stdout = Box::new(MouseTerminal::from(stdout));
        }
        stdout = Box::new(HideCursor::from(stdout));
        if !options.disable_alternate_screen {
            stdout = Box::new(stdout.into_alternate_screen().or_fail()?);
        }
        write!(stdout, "{}", termion::clear::All).or_fail()?;
        stdout.flush().or_fail()?;

        let terminal_size = Self::terminal_size().or_fail()?;

        let (tx, rx) = mpsc::channel();
        let _ = tx.send(Event::WindowResized(terminal_size));
        let event_sender = tx.clone();
        std::thread::spawn(move || listen_input_events(tx));

        let mut frame_buffer = FrameBuffer::default();
        frame_buffer.resize(terminal_size);

        Ok(Self {
            event_queue: rx,
            event_sender,
            dirty_pixels: BTreeMap::new(),
            frame_buffer,
            stdout,
        })
    }

    pub fn draw(&mut self, frame: VideoFrame<&[u8]>) -> pagurus::Result<()> {
        let terminal_size = Self::terminal_size().or_fail()?;

        if self.frame_buffer.size != terminal_size {
            self.resize_frame_buffer(terminal_size).or_fail()?;
            return Ok(());
        }

        self.draw_to_buffer(frame);
        self.draw_to_terminal().or_fail()?;

        Ok(())
    }

    pub fn request_redraw(&mut self) -> pagurus::Result<()> {
        let size = Self::terminal_size().or_fail()?;
        self.event_sender
            .send(Event::WindowResized(size))
            .or_fail()?;
        Ok(())
    }

    pub fn event_queue_mut(&mut self) -> &mut mpsc::Receiver<Event> {
        &mut self.event_queue
    }

    fn resize_frame_buffer(&mut self, size: Size) -> pagurus::Result<()> {
        self.frame_buffer.resize(size);
        self.event_sender
            .send(Event::WindowResized(size))
            .or_fail()?;
        write!(self.stdout, "{}", termion::clear::All).or_fail()?;
        Ok(())
    }

    fn terminal_size() -> pagurus::Result<Size> {
        termion::terminal_size()
            .map(|(w, h)| Size::from_wh(w as u32, h as u32 * 2))
            .or_fail()
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
}

impl std::fmt::Debug for VideoSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VideoSystem").finish()
    }
}

fn listen_input_events(tx: mpsc::Sender<Event>) {
    let mut mouse_state = MouseState::default();
    for event in std::io::stdin().events() {
        let Ok(event) = event else {
            break;
        };

        if let Some(event) = to_pagurus_event(&mut mouse_state, event) {
            if tx.send(event).is_err() {
                break;
            }
        }
    }
}

fn to_pagurus_event(mouse_state: &mut MouseState, v: termion::event::Event) -> Option<Event> {
    match v {
        termion::event::Event::Key(v) => to_pagurus_key_event(v).map(Event::Key),
        termion::event::Event::Mouse(v) => mouse_state
            .convert_to_pagurus_mouse_event(v)
            .map(Event::Mouse),
        termion::event::Event::Unsupported(_) => None,
    }
}

fn to_pagurus_key_event(v: termion::event::Key) -> Option<KeyEvent> {
    fn char_to_key(c: char) -> Key {
        match c {
            '\n' => Key::Return,
            '\t' => Key::Tab,
            c => Key::Char(c),
        }
    }

    match v {
        termion::event::Key::Backspace => Some(Key::Backspace.into()),
        termion::event::Key::Left => Some(Key::Left.into()),
        termion::event::Key::Right => Some(Key::Right.into()),
        termion::event::Key::Up => Some(Key::Up.into()),
        termion::event::Key::Down => Some(Key::Down.into()),
        termion::event::Key::Delete => Some(Key::Delete.into()),
        termion::event::Key::Esc => Some(Key::Esc.into()),
        termion::event::Key::BackTab => Some(Key::BackTab.into()),
        termion::event::Key::Char(c) => Some(char_to_key(c).into()),
        termion::event::Key::Ctrl(c) => Some(KeyEvent {
            key: char_to_key(c),
            ctrl: true,
            alt: false,
        }),
        termion::event::Key::Alt(c) => Some(KeyEvent {
            key: char_to_key(c),
            ctrl: false,
            alt: true,
        }),
        _ => None,
    }
}

#[derive(Debug, Default)]
struct MouseState {
    pressed: bool,
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
                self.pressed = false;
                if button != termion::event::MouseButton::Left {
                    return None;
                }
                self.pressed = true;
                Some(MouseEvent::Down {
                    position: position(x, y),
                })
            }
            termion::event::MouseEvent::Release(x, y) => self.pressed.then(|| {
                self.pressed = false;
                MouseEvent::Up {
                    position: position(x, y),
                }
            }),
            termion::event::MouseEvent::Hold(x, y) => self.pressed.then(|| MouseEvent::Move {
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
