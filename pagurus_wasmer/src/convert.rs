use pagurus::failure::Failure;
use pagurus::Result;
use wasmer::Value;

pub fn value_to_usize(value: &Value) -> Result<usize> {
    match value {
        Value::I32(v) => Ok(*v as usize),
        Value::I64(v) => Ok(*v as usize),
        _ => {
            Err(Failure::new().message(format!("expected a `usize`-like value, but got {value:?}")))
        }
    }
}

pub fn value_to_u32(value: &Value) -> Result<u32> {
    if let Value::I32(v) = value {
        Ok(*v as u32)
    } else {
        Err(Failure::new().message(format!("expected a `u32`-like value, but got {value:?}")))
    }
}

pub fn check_single_value(values: &[Value]) -> Result<()> {
    if values.len() != 1 {
        Err(Failure::new().message(format!(
            "expected a single return value, but got {} values",
            values.len()
        )))
    } else {
        Ok(())
    }
}
