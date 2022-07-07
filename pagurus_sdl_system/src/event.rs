use crate::input;
use pagurus::event::{Event, KeyEvent, MouseEvent, WindowEvent};
use pagurus::spatial::{Position, Size};
use sdl2::event::{Event as SdlEvent, WindowEvent as SdlWindowEvent};

pub fn to_pagurus_event(sdl_event: SdlEvent) -> Option<Event> {
    match sdl_event {
        SdlEvent::Quit { .. } => Some(Event::Terminating),
        SdlEvent::User { .. } => sdl_event.as_user_event_type::<Event>(),
        SdlEvent::Window { win_event, .. } => to_pagurus_window_event(win_event).map(Event::Window),
        SdlEvent::KeyDown { .. } | SdlEvent::KeyUp { .. } => {
            to_pagurus_key_event(sdl_event).map(Event::Key)
        }
        SdlEvent::MouseMotion { .. }
        | SdlEvent::MouseButtonDown { .. }
        | SdlEvent::MouseButtonUp { .. } => to_pagurus_mouse_event(sdl_event).map(Event::Mouse),
        _ => None,
    }
}

fn to_pagurus_key_event(sdl_event: SdlEvent) -> Option<KeyEvent> {
    match sdl_event {
        SdlEvent::KeyDown { keycode, .. } => keycode
            .and_then(input::to_pagurus_key)
            .map(|key| KeyEvent::Down { key }),
        SdlEvent::KeyUp { keycode, .. } => keycode
            .and_then(input::to_pagurus_key)
            .map(|key| KeyEvent::Up { key }),
        _ => None,
    }
}

fn to_pagurus_mouse_event(sdl_event: SdlEvent) -> Option<MouseEvent> {
    match sdl_event {
        SdlEvent::MouseMotion { x, y, .. } => {
            let position = Position { x, y };
            Some(MouseEvent::Move { position })
        }
        SdlEvent::MouseButtonDown {
            mouse_btn, x, y, ..
        } => {
            let position = Position { x, y };
            input::to_pagurus_button(mouse_btn).map(|button| MouseEvent::Down { position, button })
        }
        SdlEvent::MouseButtonUp {
            mouse_btn, x, y, ..
        } => {
            let position = Position { x, y };
            input::to_pagurus_button(mouse_btn).map(|button| MouseEvent::Up { position, button })
        }
        _ => None,
    }
}

fn to_pagurus_window_event(sdl_event: SdlWindowEvent) -> Option<WindowEvent> {
    match sdl_event {
        SdlWindowEvent::SizeChanged(width, height) => Some(WindowEvent::Resized {
            size: Size::from_wh(width as u32, height as u32),
        }),
        SdlWindowEvent::FocusGained => Some(WindowEvent::FocusGained),
        SdlWindowEvent::FocusLost => Some(WindowEvent::FocusLost),
        _ => None,
    }
}
