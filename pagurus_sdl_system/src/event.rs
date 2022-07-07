use pagurus::event::{Event, WindowEvent};
use pagurus::spatial::Size;
use sdl2::event::{Event as SdlEvent, WindowEvent as SdlWindowEvent};

pub fn to_pagurus_event(sdl_event: SdlEvent) -> Option<Event> {
    match sdl_event {
        SdlEvent::Quit { .. } => Some(Event::Terminating),
        SdlEvent::User { .. } => sdl_event.as_user_event_type::<Event>(),
        SdlEvent::Window { win_event, .. } => to_pagurus_window_event(win_event).map(Event::Window),
        // SdlEvent::KeyDown {
        //     timestamp,
        //     window_id,
        //     keycode,
        //     scancode,
        //     keymod,
        //     repeat,
        // } => todo!(),
        // SdlEvent::KeyUp {
        //     timestamp,
        //     window_id,
        //     keycode,
        //     scancode,
        //     keymod,
        //     repeat,
        // } => todo!(),
        // SdlEvent::MouseMotion {
        //     timestamp,
        //     window_id,
        //     which,
        //     mousestate,
        //     x,
        //     y,
        //     xrel,
        //     yrel,
        // } => todo!(),
        // SdlEvent::MouseButtonDown {
        //     timestamp,
        //     window_id,
        //     which,
        //     mouse_btn,
        //     clicks,
        //     x,
        //     y,
        // } => todo!(),
        // SdlEvent::MouseButtonUp {
        //     timestamp,
        //     window_id,
        //     which,
        //     mouse_btn,
        //     clicks,
        //     x,
        //     y,
        // } => todo!(),
        // SdlEvent::FingerDown {
        //     timestamp,
        //     touch_id,
        //     finger_id,
        //     x,
        //     y,
        //     dx,
        //     dy,
        //     pressure,
        // } => todo!(),
        // SdlEvent::FingerUp {
        //     timestamp,
        //     touch_id,
        //     finger_id,
        //     x,
        //     y,
        //     dx,
        //     dy,
        //     pressure,
        // } => todo!(),
        // SdlEvent::FingerMotion {
        //     timestamp,
        //     touch_id,
        //     finger_id,
        //     x,
        //     y,
        //     dx,
        //     dy,
        //     pressure,
        // } => todo!(),
        // SdlEvent::MultiGesture {
        //     timestamp,
        //     touch_id,
        //     d_theta,
        //     d_dist,
        //     x,
        //     y,
        //     num_fingers,
        // } => todo!(),
        // SdlEvent::RenderTargetsReset { timestamp } => todo!(),
        // SdlEvent::RenderDeviceReset { timestamp } => todo!(),
        _ => {
            // dbg!(sdl_event);
            None
        }
    }
}

fn to_pagurus_window_event(sdl_event: SdlWindowEvent) -> Option<WindowEvent> {
    // dbg!(&sdl_event);
    match sdl_event {
        SdlWindowEvent::SizeChanged(width, height) => Some(WindowEvent::Resized {
            size: Size::from_wh(width as u32, height as u32),
        }),
        SdlWindowEvent::FocusGained => Some(WindowEvent::FocusGained),
        SdlWindowEvent::FocusLost => Some(WindowEvent::FocusLost),
        _ => None,
    }
}

// pub fn sdl_event_to_gazami_event(from: &SdlEvent) -> Option<Event> {
//     match from {
//         SdlEvent::Quit { .. } => Some(Event::Terminate),
//         SdlEvent::KeyDown {
//             keycode: Some(code),
//             ..
//         } => to_key(code).map(|key| Event::Key(KeyEvent::Down { key })),
//         SdlEvent::KeyUp {
//             keycode: Some(code),
//             ..
//         } => to_key(code).map(|key| Event::Key(KeyEvent::Up { key })),
//         SdlEvent::MouseButtonDown {
//             mouse_btn, x, y, ..
//         } => to_button(mouse_btn).map(|button| {
//             Event::Mouse(MouseEvent::Down {
//                 position: Position::from_xy(*x as u32, *y as u32),
//                 button,
//             })
//         }),
//         SdlEvent::MouseButtonUp {
//             mouse_btn, x, y, ..
//         } => to_button(mouse_btn).map(|button| {
//             Event::Mouse(MouseEvent::Up {
//                 position: Position::from_xy(*x as u32, *y as u32),
//                 button,
//             })
//         }),
//         SdlEvent::MouseMotion { x, y, .. } => Some(Event::Mouse(MouseEvent::Move {
//             position: Position::from_xy(*x as u32, *y as u32),
//         })),
//         SdlEvent::Window { win_event, .. } => match *win_event {
//         },
//         _ => None,
//     }
// }

// fn to_key(code: &Keycode) -> Option<Key> {
//     use sdl2::keyboard::Keycode::*;

//     match code {
//         A => Some(Key::A),
//         B => Some(Key::B),
//         C => Some(Key::C),
//         D => Some(Key::D),
//         E => Some(Key::E),
//         F => Some(Key::F),
//         G => Some(Key::G),
//         H => Some(Key::H),
//         I => Some(Key::I),
//         J => Some(Key::J),
//         K => Some(Key::K),
//         L => Some(Key::L),
//         M => Some(Key::M),
//         N => Some(Key::N),
//         O => Some(Key::O),
//         P => Some(Key::P),
//         Q => Some(Key::Q),
//         R => Some(Key::R),
//         S => Some(Key::S),
//         T => Some(Key::T),
//         U => Some(Key::U),
//         V => Some(Key::V),
//         W => Some(Key::W),
//         X => Some(Key::X),
//         Y => Some(Key::Y),
//         Z => Some(Key::Z),
//         Num0 => Some(Key::Num0),
//         Num1 => Some(Key::Num1),
//         Num2 => Some(Key::Num2),
//         Num3 => Some(Key::Num3),
//         Num4 => Some(Key::Num4),
//         Num5 => Some(Key::Num5),
//         Num6 => Some(Key::Num6),
//         Num7 => Some(Key::Num7),
//         Num8 => Some(Key::Num8),
//         Num9 => Some(Key::Num9),
//         Right => Some(Key::Right),
//         Left => Some(Key::Left),
//         Down => Some(Key::Down),
//         Up => Some(Key::Up),
//         Space => Some(Key::Space),
//         Return => Some(Key::Return),
//         Backspace => Some(Key::Backspace),
//         Delete => Some(Key::Delete),
//         LShift => Some(Key::ShiftLeft),
//         RShift => Some(Key::ShiftRight),
//         LCtrl => Some(Key::CtrlLeft),
//         RCtrl => Some(Key::CtrlRight),
//         LAlt => Some(Key::AltLeft),
//         RAlt => Some(Key::AltRight),
//         _ => None,
//     }
// }

// fn to_button(btn: &MouseButton) -> Option<Button> {
//     match btn {
//         MouseButton::Left => Some(Button::Left),
//         MouseButton::Middle => Some(Button::Middle),
//         MouseButton::Right => Some(Button::Right),
//         _ => None,
//     }
// }
