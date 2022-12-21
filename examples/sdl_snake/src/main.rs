use pagurus::failure::OrFail;
use pagurus::spatial::Size;
use pagurus::{Game, Result};
use pagurus_sdl_system::SdlSystemBuilder;

fn main() -> Result<()> {
    // Game
    let mut game = snake_game::game::SnakeGame::default();

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
