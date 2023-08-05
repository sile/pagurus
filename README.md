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

#### How to build and run locally

Build the game:
```console
$ cargo build --release -p snake_game --target wasm32-unknown-unknown
$ ls target/wasm32-unknown-unknown/release/snake_game.wasm
```

Run the game on the terminal:
```console
$ cargo run --release -p snake_game --features tui
```

Run the game on a Web Browser:
```console
$ cd web/
$ npm install
$ npm run build

// A HTTP server listening on 8000 port will start
$ cd ../
$ python3 -m http.server
$ open http://localhost:8000/examples/snake_game/web/
```

Projects that use Pagurus
-------------------------

- [sile/pixcil](https://github.com/sile/pixcil): Pixel Art Editor
- [sile/ffmml](https://github.com/sile/ffmml): An MML(Music Macro Language) Implementation
- [sile/mineplacer](https://github.com/sile/mineplacer): A variant of Minesweeper game
