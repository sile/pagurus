use pagurus::failure::OrFail;
use pagurus::{Configuration, Game};
use pagurus_android_system::AndroidSystem;
use pagurus_wasmer::WasmGame;

pub const GAME_WASM_BYTES: &[u8] =
    include_bytes!("../../../target/wasm32-unknown-unknown/release/snake_game.wasm");

#[ndk_glue::main(backtrace = "on")]
pub fn main() {
    let mut game =
        WasmGame::<AndroidSystem>::new(GAME_WASM_BYTES).unwrap_or_else(|e| panic!("{e}"));
    let mut system = AndroidSystem::new().unwrap_or_else(|e| panic!("{e}"));

    let config = Configuration {
        initial_window_size: system.window_size(),
        ..Default::default()
    };

    game.initialize(&mut system, config)
        .or_fail()
        .unwrap_or_else(|e| panic!("{e}"));
    loop {
        let event = system
            .wait_event()
            .or_fail()
            .unwrap_or_else(|e| panic!("{e}"));
        let do_continue = game
            .handle_event(&mut system, event)
            .or_fail()
            .unwrap_or_else(|e| panic!("{e}"));
        if !do_continue {
            break;
        }
    }

    #[allow(deprecated)]
    ndk_glue::native_activity().finish()
}
