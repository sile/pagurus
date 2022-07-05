use pagurus::event::{Event, ResourceEvent};
use pagurus::resource::ResourceName;
use pagurus::{AudioData, Game, System, SystemConfig, VideoFrame};
use std::time::Duration;

pub fn game_new<G>() -> *mut G
where
    G: Game<WasmSystem> + Default,
{
    Box::into_raw(Box::new(G::default()))
}

pub fn game_requirements<G>(game: *mut G) -> *mut Vec<u8>
where
    G: Game<WasmSystem>,
{
    let game = unsafe { &mut *game };
    let requirements = game.requirements();
    serialize(&requirements).unwrap_or_else(|e| {
        panic!("failed to serialize the result of `Game::requirements()`: {e}");
    })
}

pub fn game_initialize<G>(game: *mut G, config_bytes_ptr: *mut Vec<u8>) -> *mut Vec<u8>
where
    G: Game<WasmSystem>,
{
    let game = unsafe { &mut *game };
    let config: SystemConfig = deserialize(config_bytes_ptr).unwrap_or_else(|e| {
        panic!("failed to deserialize `SystemConfig`: {e}");
    });
    if let Err(e) = game.initialize(&mut WasmSystem, config) {
        serialize(&e).unwrap_or_else(|e| {
            panic!("failed to serialize `Failure`: {e}");
        })
    } else {
        std::ptr::null_mut()
    }
}

pub fn game_handle_event<G>(
    game: *mut G,
    event_bytes_ptr: *mut Vec<u8>,
    data_ptr: *mut Vec<u8>,
) -> *mut Vec<u8>
where
    G: Game<WasmSystem>,
{
    let game = unsafe { &mut *game };
    let mut event: Event = deserialize(event_bytes_ptr).unwrap_or_else(|e| {
        panic!("failed to deserialize `Event`: {e}");
    });
    if data_ptr != std::ptr::null_mut() {
        if let Event::Resource(ResourceEvent::Get { data, .. }) = &mut event {
            *data = Some(*unsafe { Box::from_raw(data_ptr) });
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

pub fn memory_allocate_bytes(size: i32) -> *mut Vec<u8> {
    Box::into_raw(Box::new(vec![0u8; size as usize]))
}

pub fn memory_bytes_offset(bytes_ptr: *mut Vec<u8>) -> *mut u8 {
    unsafe { &mut *bytes_ptr }.as_mut_ptr()
}

pub fn memory_bytes_len(bytes_ptr: *mut Vec<u8>) -> i32 {
    unsafe { &mut *bytes_ptr }.len() as i32
}

pub fn memory_free_bytes(bytes_ptr: *mut Vec<u8>) {
    unsafe {
        let _ = Box::from_raw(bytes_ptr);
    }
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
        pub fn gameRequirements(game: *mut $game) -> *const Vec<u8> {
            $crate::wasm::game_requirements(game)
        }

        #[no_mangle]
        pub fn gameInitialize(game: *mut $game, config: *mut Vec<u8>) -> *mut Vec<u8> {
            $crate::wasm::game_initialize(game, config)
        }

        #[no_mangle]
        pub fn gameHandleEvent(
            game: *mut $game,
            event_bytes_ptr: *mut Vec<u8>,
            data_ptr: *mut Vec<u8>,
        ) -> *mut Vec<u8> {
            $crate::wasm::game_handle_event(game, event_bytes_ptr, data_ptr)
        }

        #[no_mangle]
        pub fn memoryAllocateBytes(size: i32) -> *mut Vec<u8> {
            $crate::wasm::memory_allocate_bytes(size)
        }

        #[no_mangle]
        pub fn memoryBytesOffset(bytes_ptr: *mut Vec<u8>) -> *mut u8 {
            $crate::wasm::memory_bytes_offset(bytes_ptr)
        }

        #[no_mangle]
        pub fn memoryBytesLen(bytes_ptr: *mut Vec<u8>) -> i32 {
            $crate::wasm::memory_bytes_len(bytes_ptr)
        }

        #[no_mangle]
        pub fn memoryFreeBytes(bytes_ptr: *mut Vec<u8>) {
            $crate::wasm::memory_free_bytes(bytes_ptr);
        }
    };
}

#[derive(Debug)]
pub struct WasmSystem;

impl System for WasmSystem {
    fn video_render(&mut self, VideoFrame { data, size }: VideoFrame) {
        extern "C" {
            fn systemVideoRender(data: *const u8, data_len: i32, width: i32);
        }
        unsafe { systemVideoRender(data.as_ptr(), data.len() as i32, size.width as i32) }
    }

    fn audio_enqueue(&mut self, AudioData { data }: AudioData) -> usize {
        extern "C" {
            fn systemAudioEnqueue(data: *const u8, data_len: i32) -> i32;
        }
        unsafe { systemAudioEnqueue(data.as_ptr(), data.len() as i32) as usize }
    }

    fn audio_cancel(&mut self) {
        extern "C" {
            fn systemAudioCancel();
        }
        unsafe { systemAudioCancel() }
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

    fn clock_set_timeout(&mut self, timeout: Duration, tag: u64) {
        extern "C" {
            fn systemClockSetTimeout(timeout: f64, tag: i64);
        }
        unsafe { systemClockSetTimeout(timeout.as_secs_f64(), tag as i64) }
    }

    fn resource_put(&mut self, name: &ResourceName, data: &[u8]) {
        let name = name.to_string();
        extern "C" {
            fn systemResourcePut(name: *const u8, name_len: i32, data: *const u8, data_len: i32);
        }
        unsafe {
            systemResourcePut(
                name.as_ptr(),
                name.len() as i32,
                data.as_ptr(),
                data.len() as i32,
            )
        }
    }

    fn resource_get(&mut self, name: &ResourceName) {
        let name = name.to_string();
        extern "C" {
            fn systemResourceGet(name: *const u8, name_len: i32);
        }
        unsafe { systemResourceGet(name.as_ptr(), name.len() as i32) }
    }

    fn resource_delete(&mut self, name: &ResourceName) {
        let name = name.to_string();
        extern "C" {
            fn systemResourceDelete(name: *const u8, name: i32);
        }
        unsafe { systemResourceDelete(name.as_ptr(), name.len() as i32) }
    }
}
