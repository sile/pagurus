use crate::spatial::{Position, Size};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub enum Event {
    Key(KeyEvent),
    // TOOD: s/Mouse/Pointer/
    Mouse(MouseEvent),
    Timeout(TimeoutTag),
    WindowResized(Size),
}

impl Event {
    pub fn position(&self) -> Option<Position> {
        match self {
            Event::Mouse(event) => Some(event.position()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct KeyEvent {
    pub ctrl: bool,
    pub alt: bool,
    pub key: Key,
}

impl From<Key> for KeyEvent {
    fn from(key: Key) -> Self {
        Self {
            ctrl: false,
            alt: false,
            key,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub enum MouseEvent {
    Move { position: Position },
    Down { position: Position },
    Up { position: Position },
}

impl MouseEvent {
    pub fn is_up(self) -> bool {
        matches!(self, Self::Up { .. })
    }

    pub fn is_down(&self) -> bool {
        matches!(self, Self::Down { .. })
    }

    pub fn position(&self) -> Position {
        match self {
            Self::Up { position, .. } | Self::Down { position, .. } | Self::Move { position } => {
                *position
            }
        }
    }

    pub fn set_position(&mut self, pos: Position) {
        match self {
            Self::Move { position } | Self::Down { position, .. } | Self::Up { position, .. } => {
                *position = pos;
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub enum Key {
    Left,
    Right,
    Down,
    Up,
    Return,
    Backspace,
    Delete,
    Tab,
    BackTab,
    Esc,
    Char(char),
    #[cfg_attr(feature = "serde", serde(other))]
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimeoutTag(u32);

impl TimeoutTag {
    pub const fn new(tag: u32) -> Self {
        Self(tag)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}
