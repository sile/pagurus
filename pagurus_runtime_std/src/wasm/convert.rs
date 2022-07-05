use crate::wasm::WasmError;
use wasmer::{RuntimeError, Value};

pub fn value_to_usize(value: &Value) -> Result<usize, WasmError> {
    match value {
        Value::I32(v) => Ok(*v as usize),
        Value::I64(v) => Ok(*v as usize),
        _ => {
            let msg = format!("expected a `usize`-like value, but got {value:?}");
            Err(RuntimeError::new(&msg).into())
        }
    }
}

pub fn value_to_u32(value: &Value) -> Result<u32, WasmError> {
    if let Value::I32(v) = value {
        Ok(*v as u32)
    } else {
        let msg = format!("expected a `u32`-like value, but got {value:?}");
        Err(RuntimeError::new(&msg).into())
    }
}

pub fn check_single_value(values: &[Value]) -> Result<(), WasmError> {
    if values.len() != 1 {
        let msg = format!(
            "expected a single return value, but got {} values",
            values.len()
        );
        Err(RuntimeError::new(&msg).into())
    } else {
        Ok(())
    }
}
