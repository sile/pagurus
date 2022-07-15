pagurus
========

[![pagurus](https://img.shields.io/crates/v/pagurus.svg)](https://crates.io/crates/pagurus)
[![Documentation](https://docs.rs/pagurus/badge.svg)](https://docs.rs/pagurus)
[![Actions Status](https://github.com/sile/pagurus/workflows/CI/badge.svg)](https://github.com/sile/pagurus/actions)
![License](https://img.shields.io/crates/l/pagurus)

🐚+🦞 Ultra-portable Rust game engine suited for offline 2D games powered by WebAssembly.

Examples
--------

### Snake

Traditional snake game: [examples/snake_game](examples/snake_game)

You can play this game on [itch.io](https://reduls.itch.io/snake).

#### How to build and run locally

Build the game:
```console
$ cargo build --release -p snake_game --target wasm32-unknown-unknown
$ ls target/wasm32-unknown-unknown/release/snake_game.wasm
```

Run the game using SDL:
```console
$ cargo run --release -p sdl_runtime -- target/wasm32-unknown-unknown/release/snake_game.wasm

// or

$ cargo run --release -p sdl_snake
```

Run the game on Android:
```console
$ cargo install cargo-apk
$ cargo apk run --release --lib -p android_snake
```

Run the game on a Web Browser:
```console
$ cd pagurus_web/
$ npm install
$ npm run build
$ npm run example  // A HTTP server listening on 8888 port will start

$ open http://localhost:8888/examples/web_snake/
```
