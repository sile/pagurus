use pagurus::failure::OrFail;
use pagurus::spatial::Size;
use pagurus::{Game, Result};
use pagurus_sdl_system::{SdlSystem, SdlSystemBuilder};
use pagurus_wasmer::WasmGame;

pub const GAME_WASM_BYTES: &[u8] =
    include_bytes!("../../../target/wasm32-unknown-unknown/release/snake_game.wasm");

fn main() -> Result<()> {
    // Game
    let mut game = WasmGame::<SdlSystem>::new(&GAME_WASM_BYTES).or_fail()?;

    // System
    let mut system = SdlSystemBuilder::new()
        .title("Snake")
        .window_size(Some(Size::from_wh(600, 600)))
        .build()
        .or_fail()?;

    // Loop
    game.initialize(&mut system).or_fail()?;
    loop {
        let event = system.wait_event();
        let do_continue = game.handle_event(&mut system, event).or_fail()?;
        if !do_continue {
            break;
        }
    }

    Ok(())
}
