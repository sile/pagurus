use crate::assets::Assets;
use crate::stages::Stage;
use crate::Env;
use pagurus::failure::OrFail;
use pagurus::spatial::Size;
use pagurus::{event::Event, Configuration, Game, Requirements, Result, System};
use pagurus_game_std::audio::AudioPlayer;
use pagurus_game_std::image::Canvas;
use pagurus_game_std::logger::Logger;
use pagurus_game_std::random::StdRng;

pagurus_game_std::export_wasm_functions!(SnakeGame);

const LOG_LEVEL: log::Level = log::Level::Debug;

const CELL_SIZE: u32 = 32;
const CELL_COUNT: u32 = 12;
const WINDOW_SIZE: Size = Size::square(CELL_SIZE * CELL_COUNT);

#[derive(Debug, Default)]
pub struct SnakeGame {
    logger: Logger,
    rng: StdRng,
    assets: Option<Assets>,
    audio_player: AudioPlayer,
    canvas: Canvas,
    stage: Stage,
}

impl<S: System> Game<S> for SnakeGame {
    fn requirements(&self) -> Result<Requirements> {
        Ok(Requirements {
            logical_window_size: Some(WINDOW_SIZE),
            ..Default::default()
        })
    }

    fn initialize(&mut self, system: &mut S, _config: Configuration) -> Result<()> {
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
            self.assets.as_ref().or_fail()?,
        );
        self.stage.initialize(&mut env).or_fail()?;
        self.render(system).or_fail()?;

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
            .render_sprite(Default::default(), &assets.sprites.background);

        let mut env = Env::new(system, &mut self.rng, &mut self.audio_player, assets);
        self.stage.render(&mut env, &mut self.canvas).or_fail()?;

        let frame = self.canvas.to_video_frame().or_fail()?;
        system.video_render(frame.as_ref());

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

        if matches!(event, Event::Terminating) {
            return Ok(false);
        }

        let mut env = Env::new(
            system,
            &mut self.rng,
            &mut self.audio_player,
            self.assets.as_ref().or_fail()?,
        );
        let do_continue = self.stage.handle_event(&mut env, event).or_fail()?;
        if env.is_render_needed {
            self.render(system).or_fail()?;
        }
        Ok(do_continue)
    }
}
