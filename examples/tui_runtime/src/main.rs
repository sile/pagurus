use clap::Parser;
use pagurus::failure::OrFail;
use pagurus::{Game, Result};
use pagurus_tui_system::{TuiSystem, TuiSystemBuilder};
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
    let mut game = WasmGame::<TuiSystem>::new(&wasm_bytes).or_fail()?;

    // System
    let mut system = TuiSystemBuilder::new().build().or_fail()?;

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
