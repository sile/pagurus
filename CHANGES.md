main
====

drop
----

- [CHANGE] Drop `pagurus_android_system` in favor of PWA

core
----

- [UPDATE] Derives `serde::{Serialize, Deserialize}` if `serde` feature is enabled

web
---

- [UPDATE] Use `CanvasRenderingContext2D.scale()` method to resize video frames
- [FIX] Fix typo in package.json (`s/pagurus.ts/pagurus.d.ts/`)
- [CHANGE] Add `SystemOptions` and make it possible to create no canvas system

v0.6.0
======

drop
----

- [CHANGE] Remove `pagurus_game_std`
- [CHANGE] Remove `pagurus_wasmer`
- [CHANGE] Remove `pagurus_tui_system`

core
----

- [UPDATE] Set custom panic hook (wasm)
- [CHANGE] Move modules in `pagurus_game_std` into `pagurus` as optional features
- [CHANGE] Enable `serde` only if the target arch is "wasm32"
- [CHANGE] Remove `#[non_exhaustive]` from event enums
- [CHANGE] Remove `WindowEvent::Focus{Lost,Gained}`
- [UPDATE] Add `timeout` module
- [CHANGE] Redesign `System:clock_set_timeout()`
- [CHANGE] Redesign audio and video interface
- [CHANGE] Make `System::console_log()` static method
- [CHANGE] Make `System::clock_{game,unix}_time()` immutable methods

web
----

- [UPDATE] Optimize `audioEnqueue()` performance

v0.5.0
======

pagurus
-------

- [CHANGE] Use `orfail` crate for error handling

pagurus_game_std
----------------

- [UPDATE] Support to load grayscale PNG files
- [CHANGE] Change feature name from `ogg` to `audio`

pagurus_windows_system
----------------------

- [UPDATE] Use the icon for exe file as the window icon
- [UPDATE] Expose `Window` struct

v0.4.0
======

pagurus
-------

- [UPDATE] Add `PixelFormat::Bgr24` for Windows
- [CHANGE] Remove `PixelFormat::{Rgb16Be,Rgb16Le}`
- [FIX] Fix video frame data size calculation bug

pagurus_game_std
----------------

- [UPDATE] Add `wasm` and `ogg` features
- [FIX] Fix a bug discarding loaded state data
- [FIX] Fix zero division bug during alpha blending where both src and dst alphas are zero

pagurus_android_system
----------------------

- [UPDATE] Update dependencies
- [CHANGE] Use R8G8B8 as the pixel format instead of R5G6B5

pagurus_web
-----------

- [UPDATE] Update dev dependencies

pagurus_windows_system
----------------------

- Initial release
