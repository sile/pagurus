use wasmer::{CompileError, ExportError, InstantiationError, MemoryError, RuntimeError};

mod bytes;
mod convert;
mod env;
mod ffi;
mod game;

pub use crate::wasm::game::WasmGame;

// TODO: remove?
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum WasmError {
    #[error(transparent)]
    Compile(#[from] CompileError),

    #[error(transparent)]
    Instantiation(#[from] InstantiationError),

    #[error(transparent)]
    Export(#[from] ExportError),

    #[error(transparent)]
    Runtime(#[from] RuntimeError),

    #[error(transparent)]
    Memory(#[from] MemoryError),
}
