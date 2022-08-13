#![allow(clippy::missing_safety_doc)] // FIXME

use pagurus::event::{Event, StateEvent};
use pagurus::failure::{Failure, OrFail};
use pagurus::spatial::Size;
use pagurus::video::{PixelFormat, VideoFrameSpec};
use pagurus::{audio::AudioData, video::VideoFrame, ActionId, Game, System};
use std::time::Duration;

pub fn game_new<G>() -> *mut G
where
    G: Game<WasmSystem> + Default,
{
    Box::into_raw(Box::new(G::default()))
}

pub unsafe fn game_initialize<G>(game: *mut G) -> *mut Vec<u8>
where
    G: Game<WasmSystem>,
{
    let game = &mut *game;
    if let Err(e) = game.initialize(&mut WasmSystem) {
        serialize(&e).unwrap_or_else(|e| {
            panic!("failed to serialize `Failure`: {e}");
        })
    } else {
        std::ptr::null_mut()
    }
}

pub unsafe fn game_handle_event<G>(
    game: *mut G,
    event_bytes_ptr: *mut Vec<u8>,
    data_ptr: *mut Vec<u8>,
) -> *mut Vec<u8>
where
    G: Game<WasmSystem>,
{
    let game = &mut *game;
    let mut event: Event = deserialize(event_bytes_ptr).unwrap_or_else(|e| {
        // TODO: return result
        panic!("failed to deserialize `Event`: {e}");
    });
    if data_ptr.is_null() {
        if let Event::State(StateEvent::Loaded { data, .. }) = &mut event {
            *data = Some(*Box::from_raw(data_ptr));
        }
    }
    match game.handle_event(&mut WasmSystem, event) {
        Err(e) => serialize(&Some(e)).unwrap_or_else(|e| {
            panic!("failed to serialize `Some(Failure)`: {e}");
        }),
        Ok(false) => serialize::<Option<()>>(&None).unwrap_or_else(|e| {
            panic!("failed to serialize `None`: {e}");
        }),
        Ok(true) => std::ptr::null_mut(),
    }
}

pub unsafe fn game_query<G>(game: *mut G, name: *mut Vec<u8>) -> *mut Vec<u8>
where
    G: Game<WasmSystem>,
{
    const OK_BYTE: u8 = 0;
    const ERR_BYTE: u8 = 1;

    let game = &mut *game;
    let result = vec_into_string(name)
        .or_fail()
        .and_then(|name| game.query(&mut WasmSystem, &name).or_fail());
    let vec = match result {
        Ok(mut v) => {
            v.push(OK_BYTE);
            v
        }
        Err(e) => {
            let mut v = serde_json::to_vec(&e).unwrap_or_else(|e| {
                panic!("failed to serialize `Failure`: {e}");
            });
            v.push(ERR_BYTE);
            v
        }
    };
    Box::into_raw(Box::new(vec))
}

pub unsafe fn game_command<G>(
    game: *mut G,
    name: *mut Vec<u8>,
    command: *mut Vec<u8>,
) -> *mut Vec<u8>
where
    G: Game<WasmSystem>,
{
    let game = &mut *game;
    let command = *Box::from_raw(command);
    let result = vec_into_string(name)
        .or_fail()
        .and_then(|name| game.command(&mut WasmSystem, &name, &command).or_fail());
    match result {
        Ok(()) => std::ptr::null_mut(),
        Err(e) => serialize(&e).unwrap_or_else(|e| {
            panic!("failed to serialize `Failure`: {e}");
        }),
    }
}

pub fn memory_allocate_bytes(size: i32) -> *mut Vec<u8> {
    Box::into_raw(Box::new(vec![0u8; size as usize]))
}

pub unsafe fn memory_bytes_offset(bytes_ptr: *mut Vec<u8>) -> *mut u8 {
    (*bytes_ptr).as_mut_ptr()
}

pub unsafe fn memory_bytes_len(bytes_ptr: *mut Vec<u8>) -> i32 {
    (*bytes_ptr).len() as i32
}

pub unsafe fn memory_free_bytes(bytes_ptr: *mut Vec<u8>) {
    let _ = Box::from_raw(bytes_ptr);
}

fn vec_into_string(vec: *mut Vec<u8>) -> Result<String, Failure> {
    unsafe { String::from_utf8(*Box::from_raw(vec)).or_fail() }
}

fn serialize<T>(item: &T) -> Result<*mut Vec<u8>, serde_json::Error>
where
    T: serde::Serialize,
{
    serde_json::to_vec(item).map(|bytes| Box::into_raw(Box::new(bytes)))
}

fn deserialize<T>(bytes: *mut Vec<u8>) -> Result<T, serde_json::Error>
where
    T: for<'de> serde::Deserialize<'de>,
{
    unsafe { serde_json::from_slice(&Box::from_raw(bytes)) }
}

#[macro_export]
macro_rules! export_wasm_functions {
    ($game:ty) => {
        #[no_mangle]
        pub fn gameNew() -> *mut $game {
            $crate::wasm::game_new()
        }

        #[no_mangle]
        pub unsafe fn gameInitialize(game: *mut $game) -> *mut Vec<u8> {
            $crate::wasm::game_initialize(game)
        }

        #[no_mangle]
        pub unsafe fn gameHandleEvent(
            game: *mut $game,
            event_bytes_ptr: *mut Vec<u8>,
            data_ptr: *mut Vec<u8>,
        ) -> *mut Vec<u8> {
            $crate::wasm::game_handle_event(game, event_bytes_ptr, data_ptr)
        }

        #[no_mangle]
        pub unsafe fn gameQuery(game: *mut $game, name: *mut Vec<u8>) -> *mut Vec<u8> {
            $crate::wasm::game_query(game, name)
        }

        #[no_mangle]
        pub unsafe fn gameCommand(
            game: *mut $game,
            name: *mut Vec<u8>,
            data: *mut Vec<u8>,
        ) -> *mut Vec<u8> {
            $crate::wasm::game_command(game, name, data)
        }

        #[no_mangle]
        pub fn memoryAllocateBytes(size: i32) -> *mut Vec<u8> {
            $crate::wasm::memory_allocate_bytes(size)
        }

        #[no_mangle]
        pub unsafe fn memoryBytesOffset(bytes_ptr: *mut Vec<u8>) -> *mut u8 {
            $crate::wasm::memory_bytes_offset(bytes_ptr)
        }

        #[no_mangle]
        pub unsafe fn memoryBytesLen(bytes_ptr: *mut Vec<u8>) -> i32 {
            $crate::wasm::memory_bytes_len(bytes_ptr)
        }

        #[no_mangle]
        pub unsafe fn memoryFreeBytes(bytes_ptr: *mut Vec<u8>) {
            $crate::wasm::memory_free_bytes(bytes_ptr);
        }
    };
}

#[derive(Debug)]
pub struct WasmSystem;

impl System for WasmSystem {
    fn video_draw(&mut self, frame: VideoFrame<&[u8]>) {
        extern "C" {
            fn systemVideoDraw(
                data: *const u8,
                data_len: usize,
                width: u32,
                stride: u32,
                format: u32,
            );
        }
        let data = frame.data();
        let width = frame.spec().resolution.width;
        let stride = frame.spec().stride;
        let format = u32::from(frame.spec().pixel_format.as_u8());
        unsafe { systemVideoDraw(data.as_ptr(), data.len(), width, stride, format) }
    }

    fn video_frame_spec(&mut self, resolution: Size) -> VideoFrameSpec {
        extern "C" {
            fn systemVideoFrameSpec(
                width: u32,
                height: u32,
                pixel_format: *mut u8,
                stride: *mut u32,
            );
        }
        let mut pixel_format = 0;
        let mut stride = 0;
        unsafe {
            systemVideoFrameSpec(
                resolution.width,
                resolution.height,
                &mut pixel_format,
                &mut stride,
            )
        };
        VideoFrameSpec {
            pixel_format: PixelFormat::from_u8(pixel_format).unwrap_or_else(|e| panic!("{e}")),
            resolution,
            stride,
        }
    }

    fn audio_enqueue(&mut self, data: AudioData) -> usize {
        extern "C" {
            fn systemAudioEnqueue(data: *const u8, data_len: i32) -> i32;
        }
        unsafe { systemAudioEnqueue(data.bytes().as_ptr(), data.bytes().len() as i32) as usize }
    }

    fn console_log(&mut self, message: &str) {
        extern "C" {
            fn systemConsoleLog(msg: *const u8, msg_len: i32);
        }
        unsafe { systemConsoleLog(message.as_ptr(), message.len() as i32) }
    }

    fn clock_game_time(&mut self) -> Duration {
        extern "C" {
            fn systemClockGameTime() -> f64;
        }
        unsafe { Duration::from_secs_f64(systemClockGameTime()) }
    }

    fn clock_unix_time(&mut self) -> Duration {
        extern "C" {
            fn systemClockUnixTime() -> f64;
        }
        unsafe { Duration::from_secs_f64(systemClockUnixTime()) }
    }

    fn clock_set_timeout(&mut self, timeout: Duration) -> ActionId {
        extern "C" {
            fn systemClockSetTimeout(timeout: f64) -> i64;
        }
        unsafe {
            let id = systemClockSetTimeout(timeout.as_secs_f64());
            ActionId::new(id as u64)
        }
    }

    fn state_save(&mut self, name: &str, data: &[u8]) -> ActionId {
        extern "C" {
            fn systemStateSave(
                name: *const u8,
                name_len: i32,
                data: *const u8,
                data_len: i32,
            ) -> i64;
        }
        unsafe {
            let id = systemStateSave(
                name.as_ptr(),
                name.len() as i32,
                data.as_ptr(),
                data.len() as i32,
            );
            ActionId::new(id as u64)
        }
    }

    fn state_load(&mut self, name: &str) -> ActionId {
        extern "C" {
            fn systemStateLoad(name: *const u8, name_len: i32) -> i64;
        }
        unsafe {
            let id = systemStateLoad(name.as_ptr(), name.len() as i32);
            ActionId::new(id as u64)
        }
    }

    fn state_delete(&mut self, name: &str) -> ActionId {
        extern "C" {
            fn systemStateDelete(name: *const u8, name: i32) -> i64;
        }
        unsafe {
            let id = systemStateDelete(name.as_ptr(), name.len() as i32);
            ActionId::new(id as u64)
        }
    }
}
