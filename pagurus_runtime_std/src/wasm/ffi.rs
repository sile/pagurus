use crate::wasm::bytes::{Bytes, BytesPtr};
use crate::wasm::convert;
use crate::wasm::env::Env;
use crate::wasm::WasmError;
use pagurus::{AudioData, System, VideoFrame};
use std::marker::PhantomData;
use std::time::Duration;
use wasmer::{Array, Function, ImportObject, Store, Value, WasmPtr};

#[derive(Debug)]
pub struct Exports {
    game_new: Function,
    game_requirements: Function,
    game_initialize: Function,
    game_handle_event: Function,
    memory_allocate_bytes: Function,
    memory_free_bytes: Function,
    memory_bytes_offset: Function,
    memory_bytes_len: Function,
}

impl Exports {
    pub fn new(exports: &wasmer::Exports) -> Result<Self, WasmError> {
        Ok(Self {
            game_new: exports.get_function("gameNew")?.clone(),
            game_requirements: exports.get_function("gameRequirements")?.clone(),
            game_initialize: exports.get_function("gameInitialize")?.clone(),
            game_handle_event: exports.get_function("gameHandleEvent")?.clone(),
            memory_allocate_bytes: exports.get_function("memoryAllocateBytes")?.clone(),
            memory_free_bytes: exports.get_function("memoryFreeBytes")?.clone(),
            memory_bytes_offset: exports.get_function("memoryBytesOffset")?.clone(),
            memory_bytes_len: exports.get_function("memoryBytesLen")?.clone(),
        })
    }

    // TODO: pagurus::Result
    pub fn game_new(&self) -> Result<Value, WasmError> {
        let values = self.game_new.call(&[])?;
        convert::check_single_value(&values)?;
        Ok(values[0].clone())
    }

    pub fn game_requirements(&self, game: &Value) -> Result<BytesPtr, WasmError> {
        let values = self.game_requirements.call(&[game.clone()])?;
        convert::check_single_value(&values)?;
        Ok(BytesPtr(values[0].clone()))
    }

    pub fn game_initialize(
        &self,
        game: &Value,
        config: Bytes,
    ) -> Result<Option<BytesPtr>, WasmError> {
        let values = self.game_initialize.call(&[game.clone(), config.take()])?;
        convert::check_single_value(&values)?;
        let error = convert::value_to_usize(&values[0])?;
        if error == 0 {
            Ok(None)
        } else {
            Ok(Some(BytesPtr(values[0].clone())))
        }
    }

    pub fn game_handle_event(
        &self,
        game: &Value,
        event: Bytes,
        data: Option<Bytes>,
    ) -> Result<Option<BytesPtr>, WasmError> {
        let null = if event.is_32bit_address() {
            Value::I32(0)
        } else {
            Value::I64(0)
        };
        let values = self.game_handle_event.call(&[
            game.clone(),
            event.take(),
            data.map_or(null, |d| d.take()),
        ])?;
        convert::check_single_value(&values)?;
        let error = convert::value_to_usize(&values[0])?;
        if error == 0 {
            Ok(None)
        } else {
            Ok(Some(BytesPtr(values[0].clone())))
        }
    }

    pub fn memory_allocate_bytes(&self, n: u32) -> Result<BytesPtr, WasmError> {
        let values = self.memory_allocate_bytes.call(&[Value::I32(n as i32)])?;
        convert::check_single_value(&values)?;
        Ok(BytesPtr(values[0].clone()))
    }

    pub fn memory_free_bytes(&self, bytes: &BytesPtr) -> Result<(), WasmError> {
        self.memory_free_bytes.call(&[bytes.0.clone()])?;
        Ok(())
    }

    pub fn memory_bytes_offset(&self, bytes: &BytesPtr) -> Result<Value, WasmError> {
        let values = self.memory_bytes_offset.call(&[bytes.0.clone()])?;
        convert::check_single_value(&values)?;
        Ok(values[0].clone())
    }

    pub fn memory_bytes_len(&self, bytes: &BytesPtr) -> Result<u32, WasmError> {
        let values = self.memory_bytes_len.call(&[bytes.0.clone()])?;
        convert::check_single_value(&values)?;
        convert::value_to_u32(&values[0])
    }
}

#[derive(Debug)]
pub struct Imports<S> {
    _system: PhantomData<S>,
}

impl<S: 'static + System> Imports<S> {
    pub fn new() -> Self {
        Self {
            _system: PhantomData,
        }
    }

    pub fn to_import_object(&self, store: &Store, env: &Env<S>) -> ImportObject {
        wasmer::imports! {
            "env" => {
                "systemVideoRender" => Function::new_native_with_env(&store, env.clone(), Self::system_video_render),
                "systemAudioEnqueue" => Function::new_native_with_env(&store, env.clone(), Self::system_audio_enqueue),
                "systemAudioCancel" => Function::new_native_with_env(&store, env.clone(), Self::system_audio_cancel),
                "systemClockGameTime" => Function::new_native_with_env(&store, env.clone(), Self::system_clock_game_time),
                "systemClockUnixTime" => Function::new_native_with_env(&store, env.clone(), Self::system_clock_unix_time),
                "systemClockSetTimeout" => Function::new_native_with_env(&store, env.clone(), Self::system_clock_set_timeout),
                "systemConsoleLog" => Function::new_native_with_env(&store, env.clone(), Self::system_console_log),
                "systemResourcePut" => Function::new_native_with_env(&store, env.clone(), Self::system_resource_put),
                "systemResourceGet" => Function::new_native_with_env(&store, env.clone(), Self::system_resource_get),
                "systemResourceDelete" => Function::new_native_with_env(&store, env.clone(), Self::system_resource_delete),
            }
        }
    }

    fn system_video_render(env: &Env<S>, data: WasmPtr<u8, Array>, data_len: u32, width: u32) {
        env.with_system_and_memory(|system, memory| unsafe {
            let data = std::slice::from_raw_parts(
                memory.data_ptr().offset(data.offset() as isize),
                data_len as usize,
            );
            let frame = VideoFrame::new(data, width).unwrap_or_else(|| {
                panic!("invalid video frame: data_len={data_len}, width={width}")
            });
            system.video_render(frame);
        });
    }

    fn system_audio_enqueue(env: &Env<S>, data: WasmPtr<u8, Array>, data_len: u32) -> i32 {
        env.with_system_and_memory(|system, memory| unsafe {
            let data = std::slice::from_raw_parts(
                memory.data_ptr().offset(data.offset() as isize),
                data_len as usize,
            );
            let data = AudioData::new(data).unwrap_or_else(|| {
                panic!("invalid audio data: data_len={data_len}");
            });
            system.audio_enqueue(data) as i32
        })
    }

    fn system_audio_cancel(env: &Env<S>) {
        env.with_system(|system| {
            system.audio_cancel();
        })
    }

    fn system_clock_game_time(env: &Env<S>) -> f64 {
        env.with_system(|system| system.clock_game_time().as_secs_f64())
    }

    fn system_clock_unix_time(env: &Env<S>) -> f64 {
        env.with_system(|system| system.clock_unix_time().as_secs_f64())
    }

    fn system_clock_set_timeout(env: &Env<S>, timeout: f64, tag: i64) {
        env.with_system(|system| {
            system.clock_set_timeout(Duration::from_secs_f64(timeout), tag as u64)
        });
    }

    fn system_console_log(env: &Env<S>, msg: WasmPtr<u8, Array>, msg_len: u32) {
        env.with_system_and_memory(|system, memory| unsafe {
            let msg = msg
                .get_utf8_str(memory, msg_len)
                .unwrap_or_else(|| panic!("invalid UTF-8 string"));
            system.console_log(msg);
        });
    }

    fn system_resource_put(
        env: &Env<S>,
        name: WasmPtr<u8, Array>,
        name_len: u32,
        data: WasmPtr<u8, Array>,
        data_len: u32,
    ) {
        env.with_system_and_memory(|system, memory| unsafe {
            let name = name
                .get_utf8_str(memory, name_len)
                .unwrap_or_else(|| panic!("invalid UTF-8 string"))
                .parse()
                .unwrap_or_else(|e| panic!("failed to parse `ResourceName` string: {e}"));
            let data = std::slice::from_raw_parts(
                memory.data_ptr().offset(data.offset() as isize),
                data_len as usize,
            );
            system.resource_put(&name, data);
        })
    }

    fn system_resource_get(env: &Env<S>, name: WasmPtr<u8, Array>, name_len: u32) {
        env.with_system_and_memory(|system, memory| unsafe {
            let name = name
                .get_utf8_str(memory, name_len)
                .unwrap_or_else(|| panic!("invalid UTF-8 string"))
                .parse()
                .unwrap_or_else(|e| panic!("failed to parse `ResourceName` string: {e}"));
            system.resource_get(&name);
        })
    }

    fn system_resource_delete(env: &Env<S>, name: WasmPtr<u8, Array>, name_len: u32) {
        env.with_system_and_memory(|system, memory| unsafe {
            let name = name
                .get_utf8_str(memory, name_len)
                .unwrap_or_else(|| panic!("invalid UTF-8 string"))
                .parse()
                .unwrap_or_else(|e| panic!("failed to parse `ResourceName` string: {e}"));
            system.resource_delete(&name);
        })
    }
}
