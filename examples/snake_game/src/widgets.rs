use crate::assets::Button;
use crate::state::Direction;
use crate::{assets, Env};
use pagurus::event::{Event, Key, KeyEvent, MouseEvent};
use pagurus::failure::OrFail;
use pagurus::image::Canvas;
use pagurus::spatial::{Contains, Position, Region};
use pagurus::{Result, System};

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
            if matches!(event, KeyEvent { .. }) {
                env.change_state(&mut self.buttons[0].state, ButtonState::Focused);
            }
            return Ok(());
        };

        match event {
            KeyEvent { key: Key::Up, .. } => {
                env.change_state(&mut self.buttons[focus].state, ButtonState::Normal);
                env.change_state(
                    &mut self.buttons[focus.saturating_sub(1)].state,
                    ButtonState::Focused,
                );
            }
            KeyEvent { key: Key::Down, .. } => {
                env.change_state(&mut self.buttons[focus].state, ButtonState::Normal);
                env.change_state(
                    &mut self.buttons[std::cmp::min(focus + 1, self.buttons.len() - 1)].state,
                    ButtonState::Focused,
                );
            }
            KeyEvent {
                key: Key::Return, ..
            } => {
                env.change_state(&mut self.buttons[focus].state, ButtonState::Clicked);
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
            MouseEvent::Down { .. } => {
                env.change_state(&mut self.state, ButtonState::Pressed);
            }
            MouseEvent::Up { .. } => {
                if self.state == ButtonState::Pressed {
                    env.change_state(&mut self.state, ButtonState::Clicked);
                }
            }
        }

        Ok(true)
    }

    fn region(&self) -> Region {
        Region::new(self.position, Button::SIZE)
    }

    pub fn render<S: System>(&mut self, _env: &mut Env<S>, canvas: &mut Canvas) -> Result<()> {
        let button = match self.state {
            ButtonState::Normal => &self.sprite.normal,
            ButtonState::Focused => &self.sprite.focused,
            ButtonState::Pressed | ButtonState::Clicked => &self.sprite.pressed,
        };
        canvas.offset(self.position).draw_sprite(button);
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
    state: CursorState,
    position: Position,
    pub direction: Option<Direction>,
    pub enabled: bool,
}

impl CursorWidget {
    pub fn new(sprite: assets::Cursor) -> Self {
        Self {
            sprite,
            state: CursorState::Normal,
            position: Position::ORIGIN,
            direction: None,
            enabled: false,
        }
    }

    pub fn handle_event<S: System>(&mut self, env: &mut Env<S>, event: MouseEvent) -> Result<()> {
        match event {
            MouseEvent::Move { .. } => {
                if !matches!(self.state, CursorState::Normal) {
                    let delta = event.position() - self.position;
                    if delta.x.abs() < 16 && delta.y.abs() < 16 {
                        env.change_state(&mut self.state, CursorState::Pressing);
                    } else if delta.x.abs() > delta.y.abs() {
                        if delta.x < 0 {
                            env.change_state(&mut self.state, CursorState::Left);
                        } else {
                            env.change_state(&mut self.state, CursorState::Right);
                        }
                    } else if delta.y < 0 {
                        env.change_state(&mut self.state, CursorState::Up);
                    } else {
                        env.change_state(&mut self.state, CursorState::Down);
                    }
                    return Ok(());
                }
            }
            MouseEvent::Down { .. } if matches!(self.state, CursorState::Normal) => {
                env.change_state(&mut self.state, CursorState::Pressing);
                self.direction = None;
            }
            MouseEvent::Up { .. } => {
                self.direction = match self.state {
                    CursorState::Up => Some(Direction::Up),
                    CursorState::Down => Some(Direction::Down),
                    CursorState::Left => Some(Direction::Left),
                    CursorState::Right => Some(Direction::Right),
                    _ => None,
                };
                env.change_state(&mut self.state, CursorState::Normal);
            }
            _ => {}
        }
        self.position = event.position();

        Ok(())
    }

    pub fn render(&self, canvas: &mut Canvas) {
        if !self.enabled {
            return;
        }

        let cursor = match self.state {
            CursorState::Normal => {
                return;
            }
            CursorState::Pressing => &self.sprite.pressing,
            CursorState::Up => &self.sprite.select_up,
            CursorState::Down => &self.sprite.select_down,
            CursorState::Left => &self.sprite.select_left,
            CursorState::Right => &self.sprite.select_right,
        };
        canvas.offset(self.position - 16).draw_sprite(cursor);
    }
}

#[derive(Debug, PartialEq, Eq, Default)]
enum CursorState {
    #[default]
    Normal,
    Pressing,
    Up,
    Down,
    Left,
    Right,
}
