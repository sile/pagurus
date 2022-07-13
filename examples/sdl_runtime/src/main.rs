use clap::Parser;
use pagurus::failure::OrFail;
use pagurus::spatial::Size;
use pagurus::{Game, Result};
use pagurus_sdl_system::{SdlSystem, SdlSystemBuilder};
use pagurus_wasmer::WasmGame;
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct Args {
    game_wasm_path: PathBuf,

    #[clap(long, default_value_t = 800)]
    width: u32,

    #[clap(long, default_value_t = 600)]
    height: u32,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Game
    let wasm_bytes = std::fs::read(&args.game_wasm_path).or_fail()?;
    let mut game = WasmGame::<SdlSystem>::new(&wasm_bytes).or_fail()?;

    // System
    let mut system = SdlSystemBuilder::new()
        .title("Pagurus SDL Runtime")
        .window_size(Some(Size::from_wh(args.width, args.height)))
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
