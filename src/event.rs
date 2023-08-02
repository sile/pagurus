use crate::{
    input::{Key, MouseButton},
    spatial::{Position, Size},
    timeout::{TimeoutId, TimeoutTag},
};

#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub enum Event {
    Timeout(TimeoutEvent),
    Key(KeyEvent),
    Mouse(MouseEvent),
    Window(WindowEvent),
}

impl Event {
    pub fn position(&self) -> Option<Position> {
        match self {
            Event::Mouse(event) => Some(event.position()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct TimeoutEvent {
    pub tag: TimeoutTag,
    pub id: TimeoutId,
}

#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub enum WindowEvent {
    RedrawNeeded { size: Size },
}

#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub enum KeyEvent {
    Down { key: Key },
    Up { key: Key },
}

#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub enum MouseEvent {
    Move {
        position: Position,
    },
    Down {
        position: Position,
        button: MouseButton,
    },
    Up {
        position: Position,
        button: MouseButton,
    },
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
