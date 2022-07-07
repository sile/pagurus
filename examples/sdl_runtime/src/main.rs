use clap::Parser;
use pagurus::failure::OrFail;
use pagurus::{Game, Result, SystemConfig};
use pagurus_sdl_system::{SdlSystem, SdlSystemBuilder};
use pagurus_wasmer::WasmGame;
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct Args {
    game_wasm_path: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Game
    let wasm_bytes = std::fs::read(&args.game_wasm_path).or_fail()?;
    let mut game = WasmGame::<SdlSystem>::new(&wasm_bytes).or_fail()?;
    let requirements = game.requirements().or_fail()?;

    // System
    let mut system = SdlSystemBuilder::new()
        .logical_window_size(requirements.logical_window_size)
        .build()
        .or_fail()?;
    let config = SystemConfig {
        window_size: system.window_size(),
    };

    // Loop
    game.initialize(&mut system, config).or_fail()?;
    loop {
        let event = system.wait_event();
        let do_continue = game.handle_event(&mut system, event).or_fail()?;
        if !do_continue {
            break;
        }
    }

    Ok(())
}
