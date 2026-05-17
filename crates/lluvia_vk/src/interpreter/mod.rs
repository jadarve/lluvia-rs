mod wrappers;

use thiserror::Error;

use crate::node::ComputeNodeDescriptor;

fn register_native_types(globals: &mlua::Table) -> Result<(), InterpreterError> {
    globals
        .set("ComputeNodeDescriptor", ComputeNodeDescriptor::default())
        .map_err(|e| InterpreterError::RuntimeError {
            msg: format!("Error registering ComputeNodeDescriptor: {e:?}"),
        })?;

    Ok(())
}

///////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Error)]
pub enum InterpreterError {
    #[error("Error loading file: {path}")]
    LoadFile { path: String },

    #[error("Runtime error: {msg}")]
    RuntimeError { msg: String },
}

pub struct Interpreter {
    mlua: mlua::Lua,
}

impl Interpreter {
    pub fn new() -> Result<Self, InterpreterError> {
        let lua = mlua::Lua::new();

        let globals = lua.globals();
        register_native_types(&globals)?;

        Ok(Self { mlua: lua })
    }
}
