//! Main GCL Library File

use crate::types::*;

mod commands;
#[allow(dead_code)] // TEMP
mod context;
pub mod interp;
pub mod shell;
pub mod types;
pub mod utils;

/// Returns an Error result.
pub fn error(msg: &str) -> InterpResult {
    Err(ResultCode::Error(msg.into()))
}

/// Returns an Ok result with an empty string.
pub fn okay() -> InterpResult {
    Ok("".into())
}
