use crate::convert;
use crate::ffi::Exports;
use pagurus::failure::OrFail;
use pagurus::Result;
use wasmer::{Memory, Value};

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
    ) -> Result<Self> {
        let offset = exports.memory_bytes_offset(&wasm_bytes).or_fail()?;
        let len = if let Some(len) = len {
            len
        } else {
            exports.memory_bytes_len(&wasm_bytes).or_fail()?
        };
        let rust_slice = unsafe {
            let ptr = memory
                .data_ptr()
                .add(convert::value_to_usize(&offset).or_fail()?);
            std::slice::from_raw_parts_mut(ptr, len as usize)
        };
        Ok(Self {
            exports,
            wasm_bytes,
            rust_slice,
            has_ownership: true,
        })
    }

    pub fn from_slice(memory: &'a Memory, exports: &'a Exports, data: &[u8]) -> Result<Self> {
        let bytes_ptr = exports.memory_allocate_bytes(data.len() as u32).or_fail()?;
        let bytes = Self::new(memory, exports, bytes_ptr, Some(data.len() as u32)).or_fail()?;
        bytes.rust_slice.copy_from_slice(data);
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
