use pagurus::input::{Key, MouseButton};
use sdl2::{keyboard::Keycode, mouse::MouseButton as SdlMouseButton};

pub fn to_pagurus_button(button: SdlMouseButton) -> Option<MouseButton> {
    match button {
        SdlMouseButton::Left => Some(MouseButton::Left),
        SdlMouseButton::Middle => Some(MouseButton::Middle),
        SdlMouseButton::Right => Some(MouseButton::Right),
        _ => None,
    }
}

pub fn to_pagurus_key(code: Keycode) -> Option<Key> {
    use sdl2::keyboard::Keycode::*;

    match code {
        A => Some(Key::A),
        B => Some(Key::B),
        C => Some(Key::C),
        D => Some(Key::D),
        E => Some(Key::E),
        F => Some(Key::F),
        G => Some(Key::G),
        H => Some(Key::H),
        I => Some(Key::I),
        J => Some(Key::J),
        K => Some(Key::K),
        L => Some(Key::L),
        M => Some(Key::M),
        N => Some(Key::N),
        O => Some(Key::O),
        P => Some(Key::P),
        Q => Some(Key::Q),
        R => Some(Key::R),
        S => Some(Key::S),
        T => Some(Key::T),
        U => Some(Key::U),
        V => Some(Key::V),
        W => Some(Key::W),
        X => Some(Key::X),
        Y => Some(Key::Y),
        Z => Some(Key::Z),
        Num0 => Some(Key::Num0),
        Num1 => Some(Key::Num1),
        Num2 => Some(Key::Num2),
        Num3 => Some(Key::Num3),
        Num4 => Some(Key::Num4),
        Num5 => Some(Key::Num5),
        Num6 => Some(Key::Num6),
        Num7 => Some(Key::Num7),
        Num8 => Some(Key::Num8),
        Num9 => Some(Key::Num9),
        Right => Some(Key::Right),
        Left => Some(Key::Left),
        Down => Some(Key::Down),
        Up => Some(Key::Up),
        Space => Some(Key::Space),
        Return => Some(Key::Return),
        Backspace => Some(Key::Backspace),
        Delete => Some(Key::Delete),
        LShift => Some(Key::ShiftLeft),
        RShift => Some(Key::ShiftRight),
        LCtrl => Some(Key::CtrlLeft),
        RCtrl => Some(Key::CtrlRight),
        LAlt => Some(Key::AltLeft),
        RAlt => Some(Key::AltRight),
        _ => None,
    }
}
