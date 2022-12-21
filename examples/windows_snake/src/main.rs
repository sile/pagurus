use clap::Parser;
use pagurus::failure::OrFail;
use pagurus::spatial::Size;
use pagurus::{Game, Result};
use pagurus_windows_system::WindowsSystemBuilder;

#[derive(Debug, Parser)]
struct Args {
    #[clap(long, default_value_t = 800)]
    width: u32,

    #[clap(long, default_value_t = 600)]
    height: u32,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Game
    let mut game = snake_game::game::SnakeGame::default();

    // System
    let mut system = WindowsSystemBuilder::new("Snake")
        .window_size(Some(Size::from_wh(args.width, args.height)))
        .build()
        .or_fail()?;

    // Loop
    game.initialize(&mut system).or_fail()?;
    loop {
        let event = system.next_event();
        let do_continue = game.handle_event(&mut system, event).or_fail()?;
        if !do_continue {
            break;
        }
    }

    Ok(())
}
