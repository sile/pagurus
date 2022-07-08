use crate::assets::Assets;
use pagurus::{Result, System};

#[derive(Debug, Default)]
pub enum Stage {
    #[default]
    Uninitialized,
    Title(TitleStage),
    Play(PlayStage),
    GameOver(GameOverStage),
}

impl Stage {
    pub fn initialize<S: System>(&mut self, system: &mut S, assets: &Assets) -> Result<()> {
        todo!()
    }
}

#[derive(Debug)]
pub struct TitleStage {}

#[derive(Debug)]
pub struct PlayStage {}

#[derive(Debug)]
pub struct GameOverStage {}
