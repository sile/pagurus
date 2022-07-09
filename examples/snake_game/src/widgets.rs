use crate::assets::Button;
use crate::{assets, Env};
use pagurus::event::{Event, KeyEvent, MouseEvent};
use pagurus::failure::OrFail;
use pagurus::input::{Key, MouseButton};
use pagurus::spatial::{Contains, Position, Region};
use pagurus::{Result, System};
use pagurus_game_std::image::Canvas;

#[derive(Debug)]
pub struct ButtonGroup<'a, const N: usize> {
    buttons: [&'a mut ButtonWidget; N],
}

impl<'a, const N: usize> ButtonGroup<'a, N> {
    pub fn new(buttons: [&'a mut ButtonWidget; N]) -> Self {
        Self { buttons }
    }

    pub fn handle_event<S: System>(&mut self, env: &mut Env<S>, event: &Event) -> Result<()> {
        if let Event::Key(event) = event {
            self.handle_key_event(env, event).or_fail()?;
        } else {
            for button in &mut self.buttons {
                if button.handle_event(env, event).or_fail()? {
                    break;
                }
            }
        }
        Ok(())
    }

    fn handle_key_event<S: System>(&mut self, env: &mut Env<S>, event: &KeyEvent) -> Result<()> {
        let focus = if let Some(i) = self
            .buttons
            .iter()
            .position(|b| matches!(b.state, ButtonState::Focused | ButtonState::Pressed))
        {
            i
        } else {
            if matches!(event, KeyEvent::Up { .. }) {
                env.change_state(&mut self.buttons[0].state, ButtonState::Focused);
            }
            return Ok(());
        };

        match event {
            KeyEvent::Up { key: Key::Up } => {
                env.change_state(&mut self.buttons[focus].state, ButtonState::Normal);
                env.change_state(
                    &mut self.buttons[focus.saturating_sub(1)].state,
                    ButtonState::Focused,
                );
            }
            KeyEvent::Up { key: Key::Down } => {
                env.change_state(&mut self.buttons[focus].state, ButtonState::Normal);
                env.change_state(
                    &mut self.buttons[std::cmp::min(focus + 1, self.buttons.len() - 1)].state,
                    ButtonState::Focused,
                );
            }
            KeyEvent::Down { key: Key::Return } => {
                env.change_state(&mut self.buttons[focus].state, ButtonState::Pressed);
            }
            KeyEvent::Up { key: Key::Return } => {
                if self.buttons[focus].state == ButtonState::Pressed {
                    env.change_state(&mut self.buttons[focus].state, ButtonState::Clicked);
                }
            }
            _ => {}
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct ButtonWidget {
    sprite: assets::Button,
    position: Position,
    state: ButtonState,
}

impl ButtonWidget {
    pub fn new(sprite: assets::Button, position: Position) -> Self {
        Self {
            sprite,
            position,
            state: Default::default(),
        }
    }

    pub fn is_clicked(&self) -> bool {
        self.state == ButtonState::Clicked
    }

    pub fn handle_event<S: System>(&mut self, env: &mut Env<S>, event: &Event) -> Result<bool> {
        match event {
            // Event::Key(_) => todo!(),
            Event::Mouse(event) => self.handle_mouse_event(env, event).or_fail(),
            //Event::Touch(_) => todo!(),
            _ => Ok(false),
        }
    }

    fn handle_mouse_event<S: System>(
        &mut self,
        env: &mut Env<S>,
        event: &MouseEvent,
    ) -> Result<bool> {
        let pos = event.position();
        if !(self.region().contains(&pos)
            && self
                .sprite
                .normal
                .get_pixel(pos - self.position)
                .map_or(false, |p| p.a != 0))
        {
            env.change_state(&mut self.state, ButtonState::Normal);
            return Ok(false);
        }

        match event {
            MouseEvent::Move { .. } => {
                if self.state != ButtonState::Pressed {
                    env.change_state(&mut self.state, ButtonState::Focused);
                }
            }
            MouseEvent::Down {
                button: MouseButton::Left,
                ..
            } => {
                env.change_state(&mut self.state, ButtonState::Pressed);
            }
            MouseEvent::Up {
                button: MouseButton::Left,
                ..
            } => {
                if self.state == ButtonState::Pressed {
                    env.change_state(&mut self.state, ButtonState::Clicked);
                }
            }
            _ => {}
        }

        Ok(true)
    }

    fn region(&self) -> Region {
        Region::new(self.position, Button::SIZE)
    }

    pub fn render<S: System>(&mut self, env: &mut Env<S>, canvas: &mut Canvas) -> Result<()> {
        let button = match self.state {
            ButtonState::Normal => &self.sprite.normal,
            ButtonState::Focused => &self.sprite.focused,
            ButtonState::Pressed | ButtonState::Clicked => &self.sprite.pressed,
        };
        canvas.render_sprite(self.position, button);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Default)]
enum ButtonState {
    #[default]
    Normal,
    Focused,
    Pressed,
    Clicked,
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
