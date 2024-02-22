# Change Log

All notable changes to the "efmt" extension will be documented in this file.

Check [Keep a Changelog](http://keepachangelog.com/) for recommendations on how to structure this file.

## [Unreleased]

### Added

- [web] Add `disableMouseEvents` option
- Add `TuiSystemOptions::disable_alternate_screen` flag
- Add pulseaudio support to `pagurus_tui`
- Add `audio` feature to `pagurus_tui`
- Add `video` feature to `pagurus_tui`

### Changed

- Don't re-export `orfail::{Failure, OrFail}`

## [0.7.3] - 2023-08-18

### Added

- Add `TuiSystemOptions` struct

### Changed

- Update `orfail` to v1.1.0

## [0.7.2] - 2023-08-13

### Added

- Add `SystemOptions.disableKeyEvents` (web)
- Add conversion functions / methods between tuples and `Rgb` / `Rgba`
- Implement `PartialEq`, `Eq` and `Hash` for event types
- Add `TuiSystem::request_redraw()`
- Implement `Copy` trait for event types
- Add `Color::rgba()`

### Changed

- Use alternate screen (tui)
- Remove UMD support (web)

## [0.7.1] - 2023-08-05

### Changed

- Simplify `MouseEvent` layout
- Simplify `KeyEvent` layout
