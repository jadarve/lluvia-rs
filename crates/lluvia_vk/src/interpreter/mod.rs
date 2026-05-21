mod wrappers;

use crate::math;
use thiserror::Error;

use crate::node::ComputeNodeDescriptor;

///////////////////////////////////////////////////////////
// Resource files included in the crate
static LUAU_DIR: include_dir::Dir = include_dir::include_dir!("$CARGO_MANIFEST_DIR/resources/luau/");

fn register_native_types(globals: &mlua::Table) -> Result<(), InterpreterError> {
    globals
        .set("ComputeNodeDescriptor", ComputeNodeDescriptor::default())
        .map_err(|e| InterpreterError::RuntimeError {
            msg: format!("Error registering ComputeNodeDescriptor: {e:?}"),
        })?;

    globals
        .set("Vec3", math::Vec3::default())
        .map_err(|e| InterpreterError::RuntimeError {
            msg: format!("Error registering Vec3: {e:?}"),
        })?;

    globals
        .set("UVec3", math::UVec3::default())
        .map_err(|e| InterpreterError::RuntimeError {
            msg: format!("Error registering UVec3: {e:?}"),
        })?;

    Ok(())
}

fn load_internal_file(mlua: &mlua::Lua, path: &str) -> Result<(), InterpreterError> {
    let file = LUAU_DIR
        .get_file(path)
        .ok_or(InterpreterError::LoadFile { path: path.to_string() })?;

    let file_content = file
        .contents_utf8()
        .ok_or(InterpreterError::LoadFile { path: path.to_string() })?;

    mlua.load(file_content)
        .set_name(path)
        .exec()
        .map_err(|err| InterpreterError::RuntimeError {
            msg: format!("{err:?}"),
        })
}

fn load_internal_module(mlua: &mlua::Lua, path: &str) -> Result<(), InterpreterError> {
    let file = LUAU_DIR
        .get_file(path)
        .ok_or(InterpreterError::LoadFile { path: path.to_string() })?;

    let file_content = file
        .contents_utf8()
        .ok_or(InterpreterError::LoadFile { path: path.to_string() })?;

    let module_name = format!("@{path}");

    let chunk = mlua
        .load(file_content)
        .set_name(&module_name)
        .eval::<mlua::Value>()
        .map_err(|err| InterpreterError::RuntimeError {
            msg: format!("{err:?}"),
        })?;

    match chunk {
        mlua::Value::Table(table) => {
            mlua.register_module(&module_name, table)
                .map_err(|err| InterpreterError::RuntimeError {
                    msg: format!("Error registering module {module_name}: {err:?}"),
                })
        }
        // This case might happen for modules that only export types.
        mlua::Nil => Ok(()),
        _ => Err(InterpreterError::RuntimeError {
            msg: "Module is not a table".to_string(),
        }),
    }
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
    _mlua: mlua::Lua,
}

impl Interpreter {
    pub fn new() -> Result<Self, InterpreterError> {
        let lua = mlua::Lua::new();

        load_internal_module(&lua, "lib/ll/compute_node_builder.luau")?;
        load_internal_module(&lua, "lib/ll.luau")?;

        let globals = lua.globals();
        register_native_types(&globals)?;

        Ok(Self { _mlua: lua })
    }
}
