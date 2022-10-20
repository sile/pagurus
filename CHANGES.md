main
====


game_std
--------

- [CHANGE] Change feature name from `ogg` to `audio`


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
