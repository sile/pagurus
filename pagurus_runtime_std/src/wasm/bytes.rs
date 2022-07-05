use crate::wasm::ffi::Exports;
use crate::wasm::WasmError;
use wasmer::{Memory, RuntimeError, Value};

#[derive(Debug)]
pub struct Bytes<'a> {
    exports: &'a Exports,
    wasm_bytes: BytesPtr,
    rust_slice: &'a mut [u8],
    has_ownership: bool,
}

impl<'a> Bytes<'a> {
    pub fn new(
        memory: &'a Memory,
        exports: &'a Exports,
        wasm_bytes: BytesPtr,
        len: Option<u32>,
    ) -> Result<Self, WasmError> {
        let offset = exports.memory_bytes_offset(&wasm_bytes)?;
        let len = if let Some(len) = len {
            len
        } else {
            exports.memory_bytes_len(&wasm_bytes)?
        };
        let rust_slice = unsafe {
            let ptr = memory.data_ptr().offset(value_to_usize(&offset)? as isize);
            std::slice::from_raw_parts_mut(ptr, len as usize)
        };
        Ok(Self {
            exports,
            wasm_bytes,
            rust_slice,
            has_ownership: true,
        })
    }

    pub fn from_slice(
        memory: &'a Memory,
        exports: &'a Exports,
        data: &[u8],
    ) -> Result<Self, WasmError> {
        let bytes_ptr = exports.memory_allocate_bytes(data.len() as u32)?;
        let bytes = Self::new(memory, exports, bytes_ptr, Some(data.len() as u32))?;
        bytes.rust_slice.copy_from_slice(&data);
        Ok(bytes)
    }

    pub fn take(mut self) -> Value {
        self.has_ownership = false;
        self.wasm_bytes.0.clone()
    }

    pub fn as_slice(&self) -> &[u8] {
        self.rust_slice
    }

    pub fn is_32bit_address(&self) -> bool {
        matches!(self.wasm_bytes.0, Value::I32(_))
    }
}

impl<'a> Drop for Bytes<'a> {
    fn drop(&mut self) {
        if self.has_ownership {
            self.exports
                .memory_free_bytes(&self.wasm_bytes)
                .unwrap_or_else(|e| panic!("memory_free_bytes() failure: {e}"));
        }
    }
}

#[derive(Debug)]
pub struct BytesPtr(pub Value);

fn value_to_usize(value: &Value) -> Result<usize, WasmError> {
    match value {
        Value::I32(v) => Ok(*v as usize),
        Value::I64(v) => Ok(*v as usize),
        _ => {
            let msg = format!("expected a `usize`-like value, but got {value:?}");
            Err(RuntimeError::new(&msg).into())
        }
    }
}
