use crate::{
    failure::Failure,
    input::{Button, Key, Touch},
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
    Resource(ResourceEvent),
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
pub enum ResourceEvent {
    Put {
        id: ActionId,
        #[serde(default)]
        failed: Option<Failure>,
    },
    Get {
        id: ActionId,
        #[serde(default)]
        data: Option<Vec<u8>>,
        #[serde(default)]
        failed: Option<Failure>,
    },
    Delete {
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
    Cancel,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum TouchEvent {
    Up { touches: Vec<Touch> },
    Down { touches: Vec<Touch> },
    Move { touches: Vec<Touch> },
    Cancel,
}
