use crate::bytes::{Bytes, BytesPtr};
use crate::convert;
use crate::env::Env;
use pagurus::failure::OrFail;
use pagurus::{AudioData, Result, System, VideoFrame};
use std::marker::PhantomData;
use std::time::Duration;
use wasmer::{Array, Function, ImportObject, Store, Value, WasmPtr};

#[derive(Debug)]
pub struct Exports {
    game_new: Function,
    game_initialize: Function,
    game_handle_event: Function,
    memory_allocate_bytes: Function,
    memory_free_bytes: Function,
    memory_bytes_offset: Function,
    memory_bytes_len: Function,
}

impl Exports {
    pub fn new(exports: &wasmer::Exports) -> Result<Self> {
        Ok(Self {
            game_new: exports.get_function("gameNew").or_fail()?.clone(),
            game_initialize: exports.get_function("gameInitialize").or_fail()?.clone(),
            game_handle_event: exports.get_function("gameHandleEvent").or_fail()?.clone(),
            memory_allocate_bytes: exports
                .get_function("memoryAllocateBytes")
                .or_fail()?
                .clone(),
            memory_free_bytes: exports.get_function("memoryFreeBytes").or_fail()?.clone(),
            memory_bytes_offset: exports.get_function("memoryBytesOffset").or_fail()?.clone(),
            memory_bytes_len: exports.get_function("memoryBytesLen").or_fail()?.clone(),
        })
    }

    pub fn game_new(&self) -> Result<Value> {
        let values = self.game_new.call(&[]).or_fail()?;
        convert::check_single_value(&values).or_fail()?;
        Ok(values[0].clone())
    }

    pub fn game_initialize(&self, game: &Value) -> Result<Option<BytesPtr>> {
        let values = self.game_initialize.call(&[game.clone()]).or_fail()?;
        convert::check_single_value(&values).or_fail()?;
        let error = convert::value_to_usize(&values[0]).or_fail()?;
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
    ) -> Result<Option<BytesPtr>> {
        let null = if event.is_32bit_address() {
            Value::I32(0)
        } else {
            Value::I64(0)
        };
        let values = self
            .game_handle_event
            .call(&[game.clone(), event.take(), data.map_or(null, |d| d.take())])
            .or_fail()?;
        convert::check_single_value(&values).or_fail()?;
        let error = convert::value_to_usize(&values[0]).or_fail()?;
        if error == 0 {
            Ok(None)
        } else {
            Ok(Some(BytesPtr(values[0].clone())))
        }
    }

    pub fn memory_allocate_bytes(&self, n: u32) -> Result<BytesPtr> {
        let values = self
            .memory_allocate_bytes
            .call(&[Value::I32(n as i32)])
            .or_fail()?;
        convert::check_single_value(&values).or_fail()?;
        Ok(BytesPtr(values[0].clone()))
    }

    pub fn memory_free_bytes(&self, bytes: &BytesPtr) -> Result<()> {
        self.memory_free_bytes.call(&[bytes.0.clone()]).or_fail()?;
        Ok(())
    }

    pub fn memory_bytes_offset(&self, bytes: &BytesPtr) -> Result<Value> {
        let values = self
            .memory_bytes_offset
            .call(&[bytes.0.clone()])
            .or_fail()?;
        convert::check_single_value(&values)?;
        Ok(values[0].clone())
    }

    pub fn memory_bytes_len(&self, bytes: &BytesPtr) -> Result<u32> {
        let values = self.memory_bytes_len.call(&[bytes.0.clone()]).or_fail()?;
        convert::check_single_value(&values).or_fail()?;
        convert::value_to_u32(&values[0]).or_fail()
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
                "systemVideoDraw" => Function::new_native_with_env(&store, env.clone(), Self::system_video_draw),
                "systemAudioEnqueue" => Function::new_native_with_env(&store, env.clone(), Self::system_audio_enqueue),
                "systemClockGameTime" => Function::new_native_with_env(&store, env.clone(), Self::system_clock_game_time),
                "systemClockUnixTime" => Function::new_native_with_env(&store, env.clone(), Self::system_clock_unix_time),
                "systemClockSetTimeout" => Function::new_native_with_env(&store, env.clone(), Self::system_clock_set_timeout),
                "systemConsoleLog" => Function::new_native_with_env(&store, env.clone(), Self::system_console_log),
                "systemStateSave" => Function::new_native_with_env(&store, env.clone(), Self::system_state_save),
                "systemStateLoad" => Function::new_native_with_env(&store, env.clone(), Self::system_state_load),
                "systemStateDelete" => Function::new_native_with_env(&store, env.clone(), Self::system_state_delete),
            }
        }
    }

    fn system_video_draw(env: &Env<S>, data: WasmPtr<u8, Array>, data_len: u32, width: u32) {
        env.with_system_and_memory(|system, memory| unsafe {
            let data = std::slice::from_raw_parts(
                memory.data_ptr().offset(data.offset() as isize),
                data_len as usize,
            );
            let frame = VideoFrame::new(data, width).unwrap_or_else(|e| panic!("{e}"));
            system.video_draw(frame);
        });
    }

    fn system_audio_enqueue(env: &Env<S>, data: WasmPtr<u8, Array>, data_len: u32) -> i32 {
        env.with_system_and_memory(|system, memory| unsafe {
            let data = std::slice::from_raw_parts(
                memory.data_ptr().offset(data.offset() as isize),
                data_len as usize,
            );
            let data = AudioData::new(data).unwrap_or_else(|e| panic!("{e}"));
            system.audio_enqueue(data) as i32
        })
    }

    fn system_clock_game_time(env: &Env<S>) -> f64 {
        env.with_system(|system| system.clock_game_time().as_secs_f64())
    }

    fn system_clock_unix_time(env: &Env<S>) -> f64 {
        env.with_system(|system| system.clock_unix_time().as_secs_f64())
    }

    fn system_clock_set_timeout(env: &Env<S>, timeout: f64) -> i64 {
        env.with_system(|system| {
            system
                .clock_set_timeout(Duration::from_secs_f64(timeout))
                .get() as i64
        })
    }

    fn system_console_log(env: &Env<S>, msg: WasmPtr<u8, Array>, msg_len: u32) {
        env.with_system_and_memory(|system, memory| unsafe {
            let msg = msg
                .get_utf8_str(memory, msg_len)
                .unwrap_or_else(|| panic!("invalid UTF-8 string"));
            system.console_log(msg);
        });
    }

    fn system_state_save(
        env: &Env<S>,
        name: WasmPtr<u8, Array>,
        name_len: u32,
        data: WasmPtr<u8, Array>,
        data_len: u32,
    ) -> i64 {
        env.with_system_and_memory(|system, memory| unsafe {
            let name = name
                .get_utf8_str(memory, name_len)
                .unwrap_or_else(|| panic!("invalid UTF-8 string"));
            let data = std::slice::from_raw_parts(
                memory.data_ptr().offset(data.offset() as isize),
                data_len as usize,
            );
            system.state_save(&name, data).get() as i64
        })
    }

    fn system_state_load(env: &Env<S>, name: WasmPtr<u8, Array>, name_len: u32) -> i64 {
        env.with_system_and_memory(|system, memory| unsafe {
            let name = name
                .get_utf8_str(memory, name_len)
                .unwrap_or_else(|| panic!("invalid UTF-8 string"));
            system.state_load(&name).get() as i64
        })
    }

    fn system_state_delete(env: &Env<S>, name: WasmPtr<u8, Array>, name_len: u32) -> i64 {
        env.with_system_and_memory(|system, memory| unsafe {
            let name = name
                .get_utf8_str(memory, name_len)
                .unwrap_or_else(|| panic!("invalid UTF-8 string"));
            system.state_delete(&name).get() as i64
        })
    }
}
