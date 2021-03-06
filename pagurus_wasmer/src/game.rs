use crate::bytes::Bytes;
use crate::env::Env;
use crate::ffi::{Exports, Imports};
use pagurus::event::{Event, StateEvent};
use pagurus::failure::OrFail;
use pagurus::{Game, Result, System};
use serde::{Deserialize, Serialize};
use wasmer::{Instance, Memory, Module, Store, Value};

use super::bytes::BytesPtr;

#[derive(Debug)]
pub struct WasmGame<S> {
    env: Env<S>,
    exports: Exports,
    memory: Memory,
    game: Value,
}

impl<S: 'static + System> WasmGame<S> {
    pub fn new(wasm_module_bytes: &[u8]) -> Result<Self> {
        let store = Store::default();
        let module = Module::new(&store, wasm_module_bytes).or_fail()?;
        let env = Env::new();
        let import_object = Imports::new().to_import_object(&store, &env);
        let wasm_instance = Instance::new(&module, &import_object).or_fail()?;
        let memory = wasm_instance
            .exports
            .get_memory("memory")
            .or_fail()?
            .clone();
        let exports = Exports::new(&wasm_instance.exports).or_fail()?;
        let game = exports.game_new().or_fail()?;
        Ok(Self {
            game,
            memory,
            exports,
            env,
        })
    }
}

impl<S: System> WasmGame<S> {
    fn deserialize<T>(&self, bytes_ptr: BytesPtr) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let bytes = Bytes::new(&self.memory, &self.exports, bytes_ptr, None).or_fail()?;
        serde_json::from_slice(bytes.as_slice()).or_fail()
    }

    fn serialize<T>(&self, item: &T) -> Result<Bytes>
    where
        T: Serialize,
    {
        let bytes = serde_json::to_vec(item).or_fail()?;
        Bytes::from_slice(&self.memory, &self.exports, &bytes).or_fail()
    }
}

impl<S: System> Game<S> for WasmGame<S> {
    fn initialize(&mut self, system: &mut S) -> Result<()> {
        self.env.set_system(system).or_fail()?;

        if let Some(error_bytes_ptr) = self.exports.game_initialize(&self.game).or_fail()? {
            Err(self.deserialize(error_bytes_ptr).or_fail()?)
        } else {
            Ok(())
        }
    }

    fn handle_event(&mut self, system: &mut S, mut event: Event) -> Result<bool> {
        self.env.set_system(system).or_fail()?;

        let data = if let Event::State(StateEvent::Loaded { data, .. }) = &mut event {
            data.take()
                .map(|data| Bytes::from_slice(&self.memory, &self.exports, &data).or_fail())
                .transpose()?
        } else {
            None
        };

        let event = self.serialize(&event).or_fail()?;
        if let Some(maybe_error_bytes_ptr) = self
            .exports
            .game_handle_event(&self.game, event, data)
            .or_fail()?
        {
            if let Some(error) = self.deserialize(maybe_error_bytes_ptr).or_fail()? {
                Err(error)
            } else {
                Ok(false)
            }
        } else {
            Ok(true)
        }
    }
}
