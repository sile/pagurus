use crate::widgets::ButtonWidget;
use crate::{state::GameState, Env};
use pagurus::event::Event;
use pagurus::failure::{Failure, OrFail};
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
            Stage::Title(stage) => stage.handle_event(env, event).or_fail(),
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
        todo!()
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
        Self {
            play_button: ButtonWidget::new(env.assets.sprites.buttons.play.clone()),
            exit_button: ButtonWidget::new(env.assets.sprites.buttons.exit.clone()),
        }
    }

    fn handle_event<S: System>(
        &mut self,
        env: &mut Env<S>,
        event: Event,
    ) -> Result<HandleEventResult> {
        todo!()
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
