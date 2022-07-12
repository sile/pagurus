use crate::assets::Assets;
use crate::stages::Stage;
use crate::state::HighScore;
use crate::{Env, WINDOW_SIZE};
use pagurus::event::{StateEvent, WindowEvent};
use pagurus::failure::OrFail;
use pagurus::{event::Event, Game, Result, System};
use pagurus_game_std::audio::AudioPlayer;
use pagurus_game_std::image::Canvas;
use pagurus_game_std::logger::Logger;
use pagurus_game_std::random::StdRng;

pagurus_game_std::export_wasm_functions!(SnakeGame);

const LOG_LEVEL: log::Level = log::Level::Debug;
pub const STATE_HIGH_SCORE: &str = "high_score";

#[derive(Debug, Default)]
pub struct SnakeGame {
    logger: Logger,
    rng: StdRng,
    assets: Option<Assets>,
    audio_player: AudioPlayer,
    high_score: HighScore,
    canvas: Canvas,
    stage: Stage,
}

impl<S: System> Game<S> for SnakeGame {
    fn initialize(&mut self, system: &mut S) -> Result<()> {
        // Logger.
        self.logger = Logger::init(LOG_LEVEL).or_fail()?;

        // Rng.
        self.rng = StdRng::from_clock_seed(system.clock_unix_time());

        // Assets.
        let start = system.clock_game_time();
        self.assets = Some(Assets::load().or_fail()?);
        log::debug!(
            "assets were loaded (took {} seconds)",
            (system.clock_game_time() - start).as_secs_f64()
        );

        // Canvas.
        self.canvas = Canvas::new(WINDOW_SIZE);

        // Stage.
        let mut env = Env::new(
            system,
            &mut self.rng,
            &mut self.audio_player,
            &mut self.high_score,
            self.assets.as_ref().or_fail()?,
        );
        self.stage.initialize(&mut env).or_fail()?;
        self.render(system).or_fail()?;

        // High Score.
        system.state_load(STATE_HIGH_SCORE);

        Ok(())
    }

    fn handle_event(&mut self, system: &mut S, event: Event) -> Result<bool> {
        let result = self.handle_event_without_log_flush(system, event);
        self.logger.flush(system);
        result
    }
}

impl SnakeGame {
    fn render<S: System>(&mut self, system: &mut S) -> Result<()> {
        let assets = self.assets.as_ref().or_fail()?;
        self.canvas
            .draw_sprite(Default::default(), &assets.sprites.background);

        let mut env = Env::new(
            system,
            &mut self.rng,
            &mut self.audio_player,
            &mut self.high_score,
            assets,
        );
        self.stage.render(&mut env, &mut self.canvas).or_fail()?;

        let frame = self.canvas.to_video_frame().or_fail()?;
        system.video_draw(frame.as_ref());

        Ok(())
    }

    fn handle_event_without_log_flush<S: System>(
        &mut self,
        system: &mut S,
        event: Event,
    ) -> Result<bool> {
        let event = if let Some(event) = self.audio_player.handle_event(system, event).or_fail()? {
            event
        } else {
            return Ok(true);
        };

        match event {
            Event::Terminating => {
                return Ok(false);
            }
            Event::Window(WindowEvent::RedrawNeeded { .. }) => {
                self.render(system).or_fail()?;
                return Ok(true);
            }
            Event::State(StateEvent::Loaded {
                data: Some(data), ..
            }) => {
                (data.len() == 1).or_fail()?;
                self.high_score.0 = data[0];
                self.render(system).or_fail()?;
                return Ok(true);
            }
            _ => {}
        }

        let mut env = Env::new(
            system,
            &mut self.rng,
            &mut self.audio_player,
            &mut self.high_score,
            self.assets.as_ref().or_fail()?,
        );
        let do_continue = self.stage.handle_event(&mut env, event).or_fail()?;
        if env.is_render_needed {
            self.render(system).or_fail()?;
        }
        Ok(do_continue)
    }
}
