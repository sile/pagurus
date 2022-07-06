use pagurus::{
    event::Event, failure::Failure, Game, GameRequirements, Result, System, SystemConfig,
};

pagurus_game_std::export_wasm_functions!(SnakeGame);

pub struct SnakeGame {}

impl Default for SnakeGame {
    fn default() -> Self {
        Self {}
    }
}

impl<S: System> Game<S> for SnakeGame {
    fn requirements(&self) -> Result<GameRequirements> {
        Err(Failure::todo())
    }

    fn initialize(&mut self, system: &mut S, config: SystemConfig) -> Result<()> {
        Err(Failure::todo())
    }

    fn handle_event(&mut self, system: &mut S, event: Event) -> Result<bool> {
        Err(Failure::todo())
    }
}
