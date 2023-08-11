# Change Log

All notable changes to the "efmt" extension will be documented in this file.

Check [Keep a Changelog](http://keepachangelog.com/) for recommendations on how to structure this file.

## [Unreleased]

### Added

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
