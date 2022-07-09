use crate::assets::Button;
use crate::widgets::ButtonWidget;
use crate::WINDOW_SIZE;
use crate::{state::GameState, Env};
use pagurus::event::Event;
use pagurus::failure::{Failure, OrFail};
use pagurus::spatial::Position;
use pagurus::{Result, System};
use pagurus_game_std::image::Canvas;

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
            Stage::Play(_) => todo!(),
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
            Stage::Play(_) => todo!(),
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
        for button in [&mut self.play_button, &mut self.exit_button] {
            if button.handle_event(env, &event).or_fail()? {
                break;
            }
        }
        if self.play_button.is_clicked() {
            Err(Failure::todo())
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
}

#[derive(Debug)]
pub struct GameOverStage {}

#[derive(Debug)]
enum HandleEventResult {
    Ok,
    Exit,
    NextStage(Stage),
}
