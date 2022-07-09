use assets::Assets;
use pagurus::System;
use pagurus_game_std::{audio::AudioPlayer, random::StdRng};

pub mod assets;
pub mod game;
pub mod stages;
pub mod state;
pub mod widgets;

#[derive(Debug)]
pub struct Env<'a, S: System> {
    pub system: &'a mut S,
    pub rng: &'a mut StdRng,
    pub audio_player: &'a mut AudioPlayer,
    pub assets: &'a Assets,
    pub is_render_needed: bool,
}

impl<'a, S: System> Env<'a, S> {
    pub fn new(
        system: &'a mut S,
        rng: &'a mut StdRng,
        audio_player: &'a mut AudioPlayer,
        assets: &'a Assets,
    ) -> Self {
        Self {
            system,
            rng,
            audio_player,
            assets,
            is_render_needed: false,
        }
    }
}
