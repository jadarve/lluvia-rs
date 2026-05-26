mod wrappers;

use crate::interpreter::wrappers::compute_node::LuaComputeNode;
use crate::math;
use std::sync::{Arc, Weak};
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
        .set("PortDescriptor", crate::node::PortDescriptor::default())
        .map_err(|e| InterpreterError::RuntimeError {
            msg: format!("Error registering PortDescriptor: {e:?}"),
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

#[allow(dead_code)]
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

pub struct LuauComputeNodeBuilder {
    pub(crate) interpreter: Arc<std::sync::Mutex<Interpreter>>,
    pub(crate) builder_table_key: mlua::RegistryKey,
}

impl crate::node::ComputeNodeBuilder for LuauComputeNodeBuilder {
    fn get_descriptor(&self) -> Result<crate::node::ComputeNodeDescriptor, crate::node::ComputeNodeBuilderError> {
        let interpreter = self.interpreter.lock().unwrap();
        let lua = &interpreter.lua;

        let builder_table: mlua::Table = lua.registry_value(&self.builder_table_key).map_err(|e| {
            crate::node::ComputeNodeBuilderError::RuntimeError {
                msg: format!("Failed to retrieve builder table from registry: {e}"),
            }
        })?;

        let get_descriptor_fn: mlua::Function =
            builder_table
                .get("get_descriptor")
                .map_err(|e| crate::node::ComputeNodeBuilderError::RuntimeError {
                    msg: format!("get_descriptor function not found on builder table: {e}"),
                })?;

        let descriptor_ref: mlua::UserDataRef<crate::node::ComputeNodeDescriptor> = get_descriptor_fn
            .call((builder_table.clone(),))
            .map_err(|e| crate::node::ComputeNodeBuilderError::RuntimeError {
                msg: format!("get_descriptor call failed: {e}"),
            })?;

        Ok(descriptor_ref.clone())
    }

    fn init_node(&self, node: &mut crate::node::ComputeNode) -> Result<(), crate::node::ComputeNodeError> {
        let interpreter = self.interpreter.lock().unwrap();
        let lua = &interpreter.lua;

        let builder_table: mlua::Table = lua
            .registry_value(&self.builder_table_key)
            .map_err(|e: mlua::Error| crate::node::ComputeNodeError::DispatchFailed(e.to_string()))?;

        if let Ok(on_node_init_fn) = builder_table.get::<mlua::Function>("on_node_init") {
            let lua_node = lua
                .create_userdata(unsafe { LuaComputeNode::new(node) })
                .map_err(|e: mlua::Error| crate::node::ComputeNodeError::DispatchFailed(e.to_string()))?;
            on_node_init_fn
                .call::<()>((builder_table, lua_node))
                .map_err(|e: mlua::Error| crate::node::ComputeNodeError::DispatchFailed(e.to_string()))?;
        }
        Ok(())
    }
}

pub struct Interpreter {
    pub(crate) lua: mlua::Lua,
}

impl Interpreter {
    pub fn new() -> Result<Self, InterpreterError> {
        let lua = mlua::Lua::new();

        load_internal_module(&lua, "lib/ll/compute_node_builder.luau")?;
        load_internal_module(&lua, "lib/ll.luau")?;

        let globals = lua.globals();
        register_native_types(&globals)?;

        Ok(Self { lua })
    }

    /// Registers a back-reference to the owning [`Session`].
    ///
    /// Stores a [`Weak<Session>`] in the Lua VM's app-data so that native
    /// globals (e.g. `load_program`) can upgrade it at call time without
    /// creating a strong reference cycle between [`Session`] and
    /// [`Interpreter`].
    ///
    /// This must be called once, immediately after [`Session::new`] finishes
    /// constructing `Arc<Session>`.
    pub fn set_session(&self, session: Weak<crate::session::Session>) -> Result<(), InterpreterError> {
        self.lua.set_app_data(session);
        self.register_session_globals()
    }

    /// Injects native globals that resolve through the stored weak Session.
    fn register_session_globals(&self) -> Result<(), InterpreterError> {
        let globals = self.lua.globals();

        let load_program_fn = self
            .lua
            .create_function(|lua, path: String| {
                let weak = lua
                    .app_data_ref::<Weak<crate::session::Session>>()
                    .ok_or_else(|| mlua::Error::RuntimeError("Session not registered".to_string()))?;

                let session = weak
                    .upgrade()
                    .ok_or_else(|| mlua::Error::RuntimeError("Session has been dropped".to_string()))?;

                session
                    .load_program(&path)
                    .map_err(|e| mlua::Error::RuntimeError(e.to_string()))
            })
            .map_err(|e| InterpreterError::RuntimeError {
                msg: format!("Failed to create load_program closure: {e}"),
            })?;

        globals
            .set("load_program", load_program_fn)
            .map_err(|e| InterpreterError::RuntimeError {
                msg: format!("Failed to register load_program global: {e}"),
            })
    }

    /// Evaluates a Luau script in the interpreter's VM.
    ///
    /// Session-level globals (e.g. `load_program`) are available if
    /// [`Interpreter::set_session`] has already been called.
    pub fn exec_script(&self, script: &str) -> Result<(), InterpreterError> {
        self.lua
            .load(script)
            .exec()
            .map_err(|e| InterpreterError::RuntimeError { msg: e.to_string() })
    }

    pub fn load_compute_node_builder(&self, script_content: &str) -> Result<mlua::RegistryKey, InterpreterError> {
        let builder_table: mlua::Table = self
            .lua
            .load(script_content)
            .set_name("builder")
            .eval()
            .map_err(|e| InterpreterError::RuntimeError { msg: e.to_string() })?;

        self.lua
            .create_registry_value(builder_table)
            .map_err(|e| InterpreterError::RuntimeError { msg: e.to_string() })
    }
}
