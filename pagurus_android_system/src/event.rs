use crate::window::Window;
use ndk::event::InputEvent;
use ndk::looper::{ForeignLooper, ThreadLooper};
use pagurus::event::{Event, MouseEvent, WindowEvent};
use pagurus::failure::{Failure, OrFail};
use pagurus::input::MouseButton;
use pagurus::spatial::Position;
use pagurus::Result;
use std::collections::VecDeque;
use std::time::Duration;

#[derive(Debug)]
pub struct EventNotifier {
    foreign_looper: ForeignLooper,
}

impl EventNotifier {
    pub fn notify(&self) {
        self.foreign_looper.wake();
    }
}

#[derive(Debug)]
pub struct EventPoller {
    looper: ThreadLooper,
    queue: VecDeque<Event>,
}

impl EventPoller {
    pub fn new() -> Result<Self> {
        let looper = ThreadLooper::for_thread().or_fail()?;
        Ok(Self {
            looper,
            queue: VecDeque::new(),
        })
    }

    pub fn notifier(&self) -> EventNotifier {
        EventNotifier {
            foreign_looper: self.looper.as_foreign().clone(),
        }
    }

    pub fn poll_once_timeout(&mut self, timeout: Duration) -> Result<Option<Event>> {
        if let Some(event) = self.queue.pop_front() {
            return Ok(Some(event));
        }

        match self.looper.poll_once_timeout(timeout).or_fail()? {
            ndk::looper::Poll::Wake => {}
            ndk::looper::Poll::Callback => {}
            ndk::looper::Poll::Timeout => {}
            ndk::looper::Poll::Event { ident, .. } => match ident {
                ndk_glue::NDK_GLUE_LOOPER_EVENT_PIPE_IDENT => {
                    if let Some(event) = ndk_glue::poll_events() {
                        if let Some(event) = from_ndk_glue_event(event) {
                            self.queue.push_back(event);
                        }
                    }
                }
                ndk_glue::NDK_GLUE_LOOPER_INPUT_QUEUE_IDENT => {
                    let input_queue = ndk_glue::input_queue();
                    let input_queue = input_queue.as_ref().or_fail()?;
                    while let Some(input_event) = input_queue.get_event() {
                        if let Some(input_event) = input_queue.pre_dispatch(input_event) {
                            if let Some(event) = from_input_event(&input_event) {
                                input_queue.finish_event(input_event, true);
                                self.queue.push_back(event);
                            } else {
                                input_queue.finish_event(input_event, false);
                            }
                        }
                    }
                }
                _ => {
                    return Err(Failure::new(format!(
                        "unexpected event identifier: {ident}"
                    )))
                }
            },
        }

        Ok(self.queue.pop_front())
    }
}

fn from_ndk_glue_event(event: ndk_glue::Event) -> Option<Event> {
    println!(
        "[DEBUG] [{}:{}] ndk_glue::Event={event:?}",
        file!(),
        line!()
    );
    match event {
        ndk_glue::Event::WindowLostFocus => Some(WindowEvent::FocusLost),
        ndk_glue::Event::WindowHasFocus => Some(WindowEvent::FocusGained),
        ndk_glue::Event::WindowRedrawNeeded => Some(WindowEvent::RerenderNeeded),
        ndk_glue::Event::WindowResized => {
            if let Some(window) = &*ndk_glue::native_window() {
                let size = Window::new(window).get_window_size();
                Some(WindowEvent::Resized { size })
            } else {
                None
            }
        }
        _ => None,
    }
    .map(Event::Window)
}

fn from_input_event(event: &InputEvent) -> Option<Event> {
    match event {
        InputEvent::MotionEvent(event) => {
            let pointer = event.pointers().find(|p| p.pointer_id() == 0)?;
            let position = Position::from_xy(pointer.x() as i32, pointer.y() as i32);
            let button = MouseButton::Left;
            match event.action() {
                ndk::event::MotionAction::Down => {
                    Some(Event::Mouse(MouseEvent::Down { position, button }))
                }
                ndk::event::MotionAction::Up => {
                    Some(Event::Mouse(MouseEvent::Up { position, button }))
                }
                ndk::event::MotionAction::Move => Some(Event::Mouse(MouseEvent::Move { position })),
                _ => None,
            }
        }
        InputEvent::KeyEvent(event) => {
            println!(
                "[WARN] [{}:{}] not implemented: key_event={event:?}",
                file!(),
                line!()
            );
            None
        }
    }
}
