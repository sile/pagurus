use crate::{
    failure::Failure,
    input::{Key, MouseButton},
    spatial::{Position, Size},
    timeout::{TimeoutId, TimeoutTag},
    ActionId,
};

#[derive(Debug, Clone)]
#[cfg_attr(
    target_arch = "wasm32",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub enum Event {
    Terminating,
    Timeout(TimeoutEvent),
    Key(KeyEvent),
    Mouse(MouseEvent),
    Window(WindowEvent),
    State(StateEvent),
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
    target_arch = "wasm32",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct TimeoutEvent {
    pub tag: TimeoutTag,
    pub id: TimeoutId,
}

#[derive(Debug, Clone)]
#[cfg_attr(
    target_arch = "wasm32",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub enum WindowEvent {
    RedrawNeeded { size: Size },
}

#[derive(Debug, Clone)]
#[cfg_attr(
    target_arch = "wasm32",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub enum StateEvent {
    Saved {
        id: ActionId,
        #[cfg_attr(target_arch = "wasm32", serde(default))]
        failed: Option<Failure>,
    },
    Loaded {
        id: ActionId,
        #[cfg_attr(target_arch = "wasm32", serde(default))]
        data: Option<Vec<u8>>,
        #[cfg_attr(target_arch = "wasm32", serde(default))]
        failed: Option<Failure>,
    },
    Deleted {
        id: ActionId,
        #[cfg_attr(target_arch = "wasm32", serde(default))]
        failed: Option<Failure>,
    },
}

#[derive(Debug, Clone)]
#[cfg_attr(
    target_arch = "wasm32",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub enum KeyEvent {
    Down { key: Key },
    Up { key: Key },
}

#[derive(Debug, Clone)]
#[cfg_attr(
    target_arch = "wasm32",
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
