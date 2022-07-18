use clap::Parser;
use pagurus::failure::OrFail;
use pagurus::spatial::Size;
use pagurus::{Game, Result};
use pagurus_tui_system::{TuiSystem, TuiSystemBuilder};
use pagurus_wasmer::WasmGame;
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct Args {
    game_wasm_path: PathBuf,

    #[clap(long, short = 'w')]
    aspect_ratio_width: Option<u32>,

    #[clap(long, short = 'h')]
    aspect_ratio_height: Option<u32>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Game
    let wasm_bytes = std::fs::read(&args.game_wasm_path).or_fail()?;
    let mut game = WasmGame::<TuiSystem>::new(&wasm_bytes).or_fail()?;

    // System
    let aspect_ratio =
        if let (Some(w), Some(h)) = (args.aspect_ratio_width, args.aspect_ratio_height) {
            Some(Size::from_wh(w, h))
        } else {
            None
        };
    let mut system = TuiSystemBuilder::new()
        .aspect_ratio(aspect_ratio)
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
