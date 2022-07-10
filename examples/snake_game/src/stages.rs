use crate::assets::Button;
use crate::state::{Direction, MoveResult};
use crate::widgets::{ButtonGroup, ButtonWidget};
use crate::{state::GameState, Env};
use crate::{CELL_SIZE, WINDOW_SIZE};
use pagurus::event::{Event, KeyEvent, TimeoutEvent};
use pagurus::failure::{Failure, OrFail};
use pagurus::input::Key;
use pagurus::spatial::Position;
use pagurus::{ActionId, Result, System};
use pagurus_game_std::image::Canvas;
use std::time::Duration;

#[derive(Debug, Default)]
pub enum Stage {
    #[default]
    Uninitialized,
    Title(TitleStage),
    Play(PlayStage),
    GameOver(GameOverStage),
}

impl Stage {
    pub fn initialize<S: System>(&mut self, env: &mut Env<S>) -> Result<()> {
        matches!(self, Self::Uninitialized).or_fail()?;
        *self = Self::Title(TitleStage::new(env));
        Ok(())
    }

    pub fn handle_event<S: System>(&mut self, env: &mut Env<S>, event: Event) -> Result<bool> {
        let result = match self {
            Stage::Uninitialized => Err(Failure::unreachable()),
            Stage::Title(x) => x.handle_event(env, event).or_fail(),
            Stage::Play(x) => x.handle_event(env, event).or_fail(),
            Stage::GameOver(_) => todo!(),
        }?;
        match result {
            HandleEventResult::Ok => Ok(true),
            HandleEventResult::Exit => Ok(false),
            HandleEventResult::NextStage(stage) => {
                *self = stage;
                Ok(true)
            }
        }
    }

    pub fn render<S: System>(&mut self, env: &mut Env<S>, canvas: &mut Canvas) -> Result<()> {
        match self {
            Stage::Uninitialized => Err(Failure::unreachable()),
            Stage::Title(x) => x.render(env, canvas).or_fail(),
            Stage::Play(x) => x.render(env, canvas).or_fail(),
            Stage::GameOver(_) => todo!(),
        }
    }
}

#[derive(Debug)]
pub struct TitleStage {
    play_button: ButtonWidget,
    exit_button: ButtonWidget,
    // TODO: key, sound
}

impl TitleStage {
    fn new<S: System>(env: &mut Env<S>) -> Self {
        let x = (WINDOW_SIZE.width / 2 - Button::SIZE.width / 2) as i32;
        let y = (WINDOW_SIZE.height / 2 + 14) as i32;
        Self {
            play_button: ButtonWidget::new(
                env.assets.sprites.buttons.play.clone(),
                Position::from_xy(x, y),
            ),
            exit_button: ButtonWidget::new(
                env.assets.sprites.buttons.exit.clone(),
                Position::from_xy(x, y + 44),
            ),
        }
    }

    fn handle_event<S: System>(
        &mut self,
        env: &mut Env<S>,
        event: Event,
    ) -> Result<HandleEventResult> {
        ButtonGroup::new([&mut self.play_button, &mut self.exit_button])
            .handle_event(env, &event)
            .or_fail()?;

        if self.play_button.is_clicked() {
            Ok(HandleEventResult::NextStage(Stage::Play(PlayStage::new(
                env,
            ))))
        } else if self.exit_button.is_clicked() {
            Ok(HandleEventResult::Exit)
        } else {
            Ok(HandleEventResult::Ok)
        }
    }

    fn render<S: System>(&mut self, env: &mut Env<S>, canvas: &mut Canvas) -> Result<()> {
        self.play_button.render(env, canvas).or_fail()?;
        self.exit_button.render(env, canvas).or_fail()?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct PlayStage {
    game_state: GameState,
    prev_direction: Direction,
    curr_direction: Direction,
    move_timeout: ActionId,
}

impl PlayStage {
    fn new<S: System>(env: &mut Env<S>) -> Self {
        let move_timeout = env.system.clock_set_timeout(Duration::from_secs(0));
        Self {
            game_state: GameState::new(env.rng),
            prev_direction: Direction::Up,
            curr_direction: Direction::Up,
            move_timeout,
        }
    }

    fn handle_event<S: System>(
        &mut self,
        env: &mut Env<S>,
        event: Event,
    ) -> Result<HandleEventResult> {
        match event {
            Event::Key(event) => self.handle_key_event(env, event).or_fail()?,
            Event::Mouse(_) => todo!(),
            Event::Timeout(TimeoutEvent { id }) if id == self.move_timeout => {
                match self.game_state.move_snake(env.rng, self.curr_direction) {
                    MoveResult::Moved => {}
                    MoveResult::Ate => {
                        // TODO: sound
                    }
                    MoveResult::Crashed => {
                        return Err(Failure::todo());
                    }
                }

                self.prev_direction = self.curr_direction;
                self.move_timeout = env.system.clock_set_timeout(Duration::from_millis(250));
                env.is_render_needed = true;
            }
            _ => {}
        }
        Ok(HandleEventResult::Ok)
    }

    fn handle_key_event<S: System>(&mut self, _env: &mut Env<S>, event: KeyEvent) -> Result<()> {
        let prev = self.prev_direction;
        self.curr_direction = match event {
            KeyEvent::Up { key: Key::Up } if prev != Direction::Down => Direction::Up,
            KeyEvent::Up { key: Key::Down } if prev != Direction::Up => Direction::Down,
            KeyEvent::Up { key: Key::Left } if prev != Direction::Right => Direction::Left,
            KeyEvent::Up { key: Key::Right } if prev != Direction::Left => Direction::Right,
            _ => self.curr_direction,
        };
        Ok(())
    }

    fn render<S: System>(&mut self, env: &mut Env<S>, canvas: &mut Canvas) -> Result<()> {
        let offset = Position::from_xy(1, 1);
        let scale = CELL_SIZE;

        canvas.render_sprite(
            (offset + self.game_state.apple) * scale,
            &env.assets.sprites.items.apple,
        );
        canvas.render_sprite(
            (offset + self.game_state.snake.head) * scale,
            &env.assets.sprites.items.snake_head,
        );
        for &tail in &self.game_state.snake.tail {
            canvas.render_sprite(
                (offset + tail) * scale,
                &env.assets.sprites.items.snake_tail,
            );
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct GameOverStage {}

#[derive(Debug)]
enum HandleEventResult {
    Ok,
    Exit,
    NextStage(Stage),
}
