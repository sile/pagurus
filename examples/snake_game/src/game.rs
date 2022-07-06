use crate::assets::Assets;
use pagurus::failure::OrFail;
use pagurus::{event::Event, Game, GameRequirements, Result, System, SystemConfig};
use pagurus_game_std::logger::Logger;

pagurus_game_std::export_wasm_functions!(SnakeGame);

const LOG_LEVEL: log::Level = log::Level::Debug;

pub struct SnakeGame {
    assets: Option<Assets>,
    logger: Logger,
}

impl Default for SnakeGame {
    fn default() -> Self {
        Self {
            assets: None,
            logger: Logger::null(),
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

        Ok(())
    }

    fn handle_event(&mut self, system: &mut S, event: Event) -> Result<bool> {
        self.logger.flush(system);

        Ok(!matches!(event, Event::Terminating))
    }
}
