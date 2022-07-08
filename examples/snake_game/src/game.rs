use crate::assets::Assets;
use crate::stages::Stage;
use crate::state::GameState;
use pagurus::failure::OrFail;
use pagurus::spatial::Size;
use pagurus::{event::Event, Configuration, Game, Requirements, Result, System};
use pagurus_game_std::audio::AudioPlayer;
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
    game_state: GameState,
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

        // Game state.
        self.game_state = GameState::new(&mut self.rng);

        // Stage.
        self.stage
            .initialize(system, self.assets.as_ref().or_fail()?)
            .or_fail()?;

        // FIXME:
        let audio = self
            .assets
            .as_ref()
            .unwrap()
            .audios
            .load_click_audio()
            .or_fail()?;
        self.audio_player.play(system, audio).or_fail()?;

        Ok(())
    }

    fn handle_event(&mut self, system: &mut S, event: Event) -> Result<bool> {
        let result = self.handle_event_without_log_flush(system, event);
        self.logger.flush(system);
        result
    }
}

impl SnakeGame {
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

        Ok(!matches!(event, Event::Terminating))
    }
}
