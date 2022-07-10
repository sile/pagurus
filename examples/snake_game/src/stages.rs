use crate::assets::Button;
use crate::state::{Direction, MoveResult};
use crate::widgets::{ButtonGroup, ButtonWidget, CursorWidget};
use crate::{state::GameState, Env};
use crate::{CELL_SIZE, WINDOW_SIZE};
use pagurus::event::{Event, KeyEvent, MouseEvent, TimeoutEvent};
use pagurus::failure::{Failure, OrFail};
use pagurus::input::Key;
use pagurus::spatial::Position;
use pagurus::{ActionId, Result, System};
use pagurus_game_std::color::Rgb;
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
            Stage::GameOver(x) => x.handle_event(env, event).or_fail(),
        }?;
        match result {
            HandleEventResult::Ok => Ok(true),
            HandleEventResult::Exit => Ok(false),
            HandleEventResult::NextStage(stage) => {
                env.is_render_needed = true;
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
            Stage::GameOver(x) => x.render(env, canvas).or_fail(),
        }
    }
}

#[derive(Debug)]
pub struct TitleStage {
    play_button: ButtonWidget,
    exit_button: ButtonWidget,
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
            let audio = env.assets.audios.load_click_audio().or_fail()?;
            env.audio_player.play(env.system, audio).or_fail()?;

            let stage = PlayStage::new(env);
            Ok(HandleEventResult::NextStage(Stage::Play(stage)))
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
    cursor: CursorWidget,
}

impl PlayStage {
    fn new<S: System>(env: &mut Env<S>) -> Self {
        let move_timeout = env.system.clock_set_timeout(Duration::from_secs(0));
        Self {
            game_state: GameState::new(env.rng),
            prev_direction: Direction::Up,
            curr_direction: Direction::Up,
            move_timeout,
            cursor: CursorWidget::new(env.assets.sprites.cursor.clone()),
        }
    }

    fn handle_event<S: System>(
        &mut self,
        env: &mut Env<S>,
        event: Event,
    ) -> Result<HandleEventResult> {
        match event {
            Event::Key(event) => self.handle_key_event(env, event).or_fail()?,
            Event::Mouse(event) => self.handle_mouse_event(env, event).or_fail()?,
            Event::Timeout(event) => {
                if !self.handle_timeout_event(env, event).or_fail()? {
                    let audio = env.assets.audios.load_crash_audio().or_fail()?;
                    env.audio_player.play(env.system, audio).or_fail()?;

                    let stage = GameOverStage::new(self.game_state.clone(), env);
                    return Ok(HandleEventResult::NextStage(Stage::GameOver(stage)));
                }
            }
            _ => {}
        }
        Ok(HandleEventResult::Ok)
    }

    fn handle_timeout_event<S: System>(
        &mut self,
        env: &mut Env<S>,
        event: TimeoutEvent,
    ) -> Result<bool> {
        if event.id == self.move_timeout {
            match self.game_state.move_snake(env.rng, self.curr_direction) {
                MoveResult::Moved => {}
                MoveResult::Ate => {
                    let audio = env.assets.audios.load_eat_audio().or_fail()?;
                    env.audio_player.play(env.system, audio).or_fail()?;
                }
                MoveResult::Crashed => {
                    return Ok(false);
                }
            }

            self.prev_direction = self.curr_direction;
            self.move_timeout = env.system.clock_set_timeout(Duration::from_millis(250));
            env.is_render_needed = true;
        }
        Ok(true)
    }

    fn handle_mouse_event<S: System>(&mut self, env: &mut Env<S>, event: MouseEvent) -> Result<()> {
        env.change_state(&mut self.cursor.enabled, true);
        self.cursor.handle_event(env, event).or_fail()?;
        if let Some(d) = self.cursor.direction {
            if d.reverse() != self.prev_direction {
                self.curr_direction = d;
            }
        }
        Ok(())
    }

    fn handle_key_event<S: System>(&mut self, env: &mut Env<S>, event: KeyEvent) -> Result<()> {
        env.change_state(&mut self.cursor.enabled, false);

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
        render_game_state(env, canvas, &self.game_state);
        self.cursor.render(canvas);

        Ok(())
    }
}

fn render_game_state<S: System>(env: &mut Env<S>, canvas: &mut Canvas, game_state: &GameState) {
    let offset = Position::from_xy(1, 1);
    let scale = CELL_SIZE;

    canvas.render_sprite(
        (offset + game_state.apple) * scale,
        &env.assets.sprites.items.apple,
    );
    canvas.render_sprite(
        (offset + game_state.snake.head) * scale,
        &env.assets.sprites.items.snake_head,
    );
    for &tail in &game_state.snake.tail {
        canvas.render_sprite(
            (offset + tail) * scale,
            &env.assets.sprites.items.snake_tail,
        );
    }
}

#[derive(Debug)]
pub struct GameOverStage {
    game_state: GameState,
    retry_button: ButtonWidget,
    title_button: ButtonWidget,
}

impl GameOverStage {
    fn new<S: System>(game_state: GameState, env: &mut Env<S>) -> Self {
        let x = (WINDOW_SIZE.width / 2 - Button::SIZE.width / 2) as i32;
        let y = (WINDOW_SIZE.height / 2 + 14) as i32;
        Self {
            game_state,
            retry_button: ButtonWidget::new(
                env.assets.sprites.buttons.retry.clone(),
                Position::from_xy(x, y),
            ),
            title_button: ButtonWidget::new(
                env.assets.sprites.buttons.title.clone(),
                Position::from_xy(x, y + 44),
            ),
        }
    }

    fn handle_event<S: System>(
        &mut self,
        env: &mut Env<S>,
        event: Event,
    ) -> Result<HandleEventResult> {
        ButtonGroup::new([&mut self.retry_button, &mut self.title_button])
            .handle_event(env, &event)
            .or_fail()?;

        if self.retry_button.is_clicked() {
            let audio = env.assets.audios.load_click_audio().or_fail()?;
            env.audio_player.play(env.system, audio).or_fail()?;

            let stage = PlayStage::new(env);
            Ok(HandleEventResult::NextStage(Stage::Play(stage)))
        } else if self.title_button.is_clicked() {
            let audio = env.assets.audios.load_click_audio().or_fail()?;
            env.audio_player.play(env.system, audio).or_fail()?;

            let stage = TitleStage::new(env);
            Ok(HandleEventResult::NextStage(Stage::Title(stage)))
        } else {
            Ok(HandleEventResult::Ok)
        }
    }

    fn render<S: System>(&mut self, env: &mut Env<S>, canvas: &mut Canvas) -> Result<()> {
        render_game_state(env, canvas, &self.game_state);
        canvas.fill_rgba(Rgb::BLACK.alpha(60));

        self.retry_button.render(env, canvas).or_fail()?;
        self.title_button.render(env, canvas).or_fail()?;

        Ok(())
    }
}

#[derive(Debug)]
enum HandleEventResult {
    Ok,
    Exit,
    NextStage(Stage),
}
