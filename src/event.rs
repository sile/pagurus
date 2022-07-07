use crate::{
    failure::Failure,
    input::{Button, Key, TouchId},
    spatial::{Position, Size},
    ActionId,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum Event {
    Terminating,
    Timeout(TimeoutEvent),
    Key(KeyEvent),
    Mouse(MouseEvent),
    Touch(TouchEvent),
    Window(WindowEvent),
    State(StateEvent),
}

impl Event {
    pub fn position(&self) -> Option<Position> {
        match self {
            Event::Mouse(event) => Some(event.position()),
            Event::Touch(event) => Some(event.position()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeoutEvent {
    pub id: ActionId,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum WindowEvent {
    Resized { size: Size },
    RedrawNeeded,
    FocusGained,
    FocusLost,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum StateEvent {
    Saved {
        id: ActionId,
        #[serde(default)]
        failed: Option<Failure>,
    },
    Loaded {
        id: ActionId,
        #[serde(default)]
        data: Option<Vec<u8>>,
        #[serde(default)]
        failed: Option<Failure>,
    },
    Deleted {
        id: ActionId,
        #[serde(default)]
        failed: Option<Failure>,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum KeyEvent {
    Up { key: Key },
    Down { key: Key },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum MouseEvent {
    Up { position: Position, button: Button },
    Down { position: Position, button: Button },
    Move { position: Position },
}

impl MouseEvent {
    pub fn position(&self) -> Position {
        match self {
            Self::Up { position, .. } | Self::Down { position, .. } | Self::Move { position } => {
                *position
            }
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum TouchEvent {
    Up { id: TouchId, position: Position },
    Down { id: TouchId, position: Position },
    Move { id: TouchId, position: Position },
}

impl TouchEvent {
    pub fn position(&self) -> Position {
        match self {
            Self::Up { position, .. }
            | Self::Down { position, .. }
            | Self::Move { position, .. } => *position,
        }
    }
}
