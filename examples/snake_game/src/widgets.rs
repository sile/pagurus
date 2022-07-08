use crate::assets;
use pagurus::event::Event;
use pagurus::{Result, System};
use pagurus_game_std::image::Canvas;

#[derive(Debug)]
pub struct ButtonWidget {
    sprite: assets::Button,
}

impl ButtonWidget {
    pub fn new(sprite: assets::Button) -> Self {
        Self { sprite }
    }

    pub fn handle_event<S: System>(
        &mut self,
        _system: &mut S,
        _event: Event,
    ) -> Result<Option<Event>> {
        todo!()
    }

    pub fn render(&self, _canvas: &mut Canvas) {}
}

#[derive(Debug)]
pub struct CursorWidget {
    sprite: assets::Cursor,
}

impl CursorWidget {
    pub fn new(sprite: assets::Cursor) -> Self {
        Self { sprite }
    }

    pub fn handle_event<S: System>(
        &mut self,
        _system: &mut S,
        _event: Event,
    ) -> Result<Option<Event>> {
        todo!()
    }

    pub fn render(&self, _canvas: &mut Canvas) {}
}
