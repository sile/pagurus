use assets::Assets;
use pagurus::{spatial::Size, System};
use pagurus_game_std::{audio::AudioPlayer, random::StdRng};

pub mod assets;
pub mod game;
pub mod stages;
pub mod state;
pub mod widgets;

pub const CELL_SIZE: u32 = 32;
pub const CELL_COUNT: u32 = 12;
pub const WINDOW_SIZE: Size = Size::square(CELL_SIZE * CELL_COUNT);

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

    pub fn change_state<T: PartialEq>(&mut self, old: &mut T, new: T) {
        if *old != new {
            *old = new;
            self.is_render_needed = true;
        }
    }
}
