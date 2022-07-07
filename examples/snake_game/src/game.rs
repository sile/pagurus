use crate::assets::Assets;
use pagurus::failure::OrFail;
use pagurus::{event::Event, Game, GameRequirements, Result, System, SystemConfig};
use pagurus_game_std::audio::AudioPlayer;
use pagurus_game_std::logger::Logger;

pagurus_game_std::export_wasm_functions!(SnakeGame);

const LOG_LEVEL: log::Level = log::Level::Debug;

pub struct SnakeGame {
    logger: Logger,
    assets: Option<Assets>,
    audio_player: AudioPlayer,
}

impl Default for SnakeGame {
    fn default() -> Self {
        Self {
            logger: Logger::null(),
            assets: None,
            audio_player: AudioPlayer::new(),
        }
    }
}

impl<S: System> Game<S> for SnakeGame {
    fn requirements(&self) -> Result<GameRequirements> {
        // TODO
        Ok(Default::default())
    }

    fn initialize(&mut self, system: &mut S, config: SystemConfig) -> Result<()> {
        self.logger = Logger::init(LOG_LEVEL).or_fail()?;

        let start = system.clock_game_time();
        self.assets = Some(Assets::load().or_fail()?);
        log::debug!(
            "assets were loaded (took {} seconds)",
            (system.clock_game_time() - start).as_secs_f64()
        );

        // TODO:
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
        let result = (|| {
            let event =
                if let Some(event) = self.audio_player.handle_event(system, event).or_fail()? {
                    event
                } else {
                    return Ok(true);
                };
            Ok(!matches!(event, Event::Terminating))
        })();
        self.logger.flush(system);
        result
    }
}
