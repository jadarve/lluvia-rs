pub(crate) mod globals;
pub(crate) mod wrappers;

use crate::interpreter::wrappers::compute_node::LuaComputeNode;
use crate::math;
use std::sync::{Arc, Weak};
use thiserror::Error;

use crate::interpreter::wrappers::compute_node_descriptor::LuaComputeNodeDescriptorBuilder;

///////////////////////////////////////////////////////////
// Resource files included in the crate
static LUAU_DIR: include_dir::Dir = include_dir::include_dir!("$CARGO_MANIFEST_DIR/resources/luau/");

fn register_native_types(globals: &mlua::Table) -> Result<(), InterpreterError> {
    globals
        .set(
            "ComputeNodeDescriptorBuilder",
            LuaComputeNodeDescriptorBuilder::default(),
        )
        .map_err(|e| InterpreterError::RuntimeError {
            msg: format!("Error registering ComputeNodeDescriptorBuilder: {e:?}"),
        })?;

    globals
        .set(
            "ContainerNodeDescriptorBuilder",
            crate::interpreter::wrappers::container_node_descriptor::LuaContainerNodeDescriptorBuilder::default(),
        )
        .map_err(|e| InterpreterError::RuntimeError {
            msg: format!("Error registering ContainerNodeDescriptorBuilder: {e:?}"),
        })?;

    globals
        .set(
            "ContainerNodeDescriptor",
            crate::node::ContainerNodeDescriptor::default(),
        )
        .map_err(|e| InterpreterError::RuntimeError {
            msg: format!("Error registering ContainerNodeDescriptor: {e:?}"),
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

    globals
        .set("UVec2", math::UVec2::default())
        .map_err(|e| InterpreterError::RuntimeError {
            msg: format!("Error registering UVec2: {e:?}"),
        })?;

    globals
        .set("PushConstants", crate::node::PushConstants::default())
        .map_err(|e| InterpreterError::RuntimeError {
            msg: format!("Error registering PushConstants: {e:?}"),
        })?;

    globals
        .set("ImageDescriptor", crate::image::ImageDescriptor::default())
        .map_err(|e| InterpreterError::RuntimeError {
            msg: format!("Error registering ImageDescriptor: {e:?}"),
        })?;

    globals
        .set("ImageViewDescriptor", crate::image::ImageViewDescriptor::default())
        .map_err(|e| InterpreterError::RuntimeError {
            msg: format!("Error registering ImageViewDescriptor: {e:?}"),
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
    pub(crate) name: String,
}

impl crate::node::ComputeNodeBuilderImpl for LuauComputeNodeBuilder {
    fn build_descriptor(
        &self,
        args: std::collections::HashMap<String, crate::node::Argument>,
    ) -> Result<crate::node::ComputeNodeDescriptor, crate::node::ComputeNodeBuilderError> {
        let interpreter = self.interpreter.lock().unwrap();
        let lua = &interpreter.lua;

        let lua_args = lua
            .create_table()
            .map_err(|e| crate::node::ComputeNodeBuilderError::RuntimeError {
                msg: format!("Failed to create Lua table for arguments: {e}"),
            })?;

        for (k, v) in args {
            let lua_val = match v {
                crate::node::Argument::F32(x) => mlua::Value::Number(x as f64),
                crate::node::Argument::I32(x) => mlua::Value::Integer(x as i64),
                crate::node::Argument::Bool(x) => mlua::Value::Boolean(x),
                crate::node::Argument::Vec3(x) => lua.create_userdata(x).map(mlua::Value::UserData).map_err(|e| {
                    crate::node::ComputeNodeBuilderError::RuntimeError {
                        msg: format!("Failed to wrap Vec3: {e}"),
                    }
                })?,
                crate::node::Argument::UVec3(x) => lua.create_userdata(x).map(mlua::Value::UserData).map_err(|e| {
                    crate::node::ComputeNodeBuilderError::RuntimeError {
                        msg: format!("Failed to wrap UVec3: {e}"),
                    }
                })?,
                crate::node::Argument::UVec2(x) => lua.create_userdata(x).map(mlua::Value::UserData).map_err(|e| {
                    crate::node::ComputeNodeBuilderError::RuntimeError {
                        msg: format!("Failed to wrap UVec2: {e}"),
                    }
                })?,
                crate::node::Argument::String(x) => lua.create_string(&x).map(mlua::Value::String).map_err(|e| {
                    crate::node::ComputeNodeBuilderError::RuntimeError {
                        msg: format!("Failed to wrap String: {e}"),
                    }
                })?,
            };
            lua_args
                .set(k, lua_val)
                .map_err(|e| crate::node::ComputeNodeBuilderError::RuntimeError {
                    msg: format!("Failed to set Lua table field: {e}"),
                })?;
        }

        let builder_table: mlua::Table = lua.registry_value(&self.builder_table_key).map_err(|e| {
            crate::node::ComputeNodeBuilderError::RuntimeError {
                msg: format!("Failed to retrieve builder table from registry: {e}"),
            }
        })?;

        let build_descriptor_fn: mlua::Function =
            builder_table
                .get("build_descriptor")
                .map_err(|e| crate::node::ComputeNodeBuilderError::RuntimeError {
                    msg: format!("build_descriptor function not found on builder table: {e}"),
                })?;

        let descriptor_ref: mlua::UserDataRef<crate::node::ComputeNodeDescriptor> = build_descriptor_fn
            .call((builder_table.clone(), lua_args))
            .map_err(|e| crate::node::ComputeNodeBuilderError::RuntimeError {
                msg: format!("build_descriptor call failed: {e}"),
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
                .create_userdata(unsafe { LuaComputeNode::new(node, self.name.clone()) })
                .map_err(|e: mlua::Error| crate::node::ComputeNodeError::DispatchFailed(e.to_string()))?;
            on_node_init_fn
                .call::<()>((builder_table, lua_node))
                .map_err(|e: mlua::Error| crate::node::ComputeNodeError::DispatchFailed(e.to_string()))?;
        }
        Ok(())
    }
}

pub struct LuauContainerNodeBuilder {
    pub(crate) interpreter: Arc<std::sync::Mutex<Interpreter>>,
    pub(crate) builder_table_key: mlua::RegistryKey,
}

impl crate::node::ContainerNodeBuilderImpl for LuauContainerNodeBuilder {
    fn build_descriptor(
        &self,
        args: std::collections::HashMap<String, crate::node::Argument>,
    ) -> Result<crate::node::ContainerNodeDescriptor, crate::node::ComputeNodeBuilderError> {
        let interpreter = self.interpreter.lock().unwrap();
        let lua = &interpreter.lua;

        let lua_args = lua
            .create_table()
            .map_err(|e| crate::node::ComputeNodeBuilderError::RuntimeError {
                msg: format!("Failed to create Lua table for arguments: {e}"),
            })?;

        for (k, v) in args {
            let lua_val = match v {
                crate::node::Argument::F32(x) => mlua::Value::Number(x as f64),
                crate::node::Argument::I32(x) => mlua::Value::Integer(x as i64),
                crate::node::Argument::Bool(x) => mlua::Value::Boolean(x),
                crate::node::Argument::Vec3(x) => lua.create_userdata(x).map(mlua::Value::UserData).map_err(|e| {
                    crate::node::ComputeNodeBuilderError::RuntimeError {
                        msg: format!("Failed to wrap Vec3: {e}"),
                    }
                })?,
                crate::node::Argument::UVec3(x) => lua.create_userdata(x).map(mlua::Value::UserData).map_err(|e| {
                    crate::node::ComputeNodeBuilderError::RuntimeError {
                        msg: format!("Failed to wrap UVec3: {e}"),
                    }
                })?,
                crate::node::Argument::UVec2(x) => lua.create_userdata(x).map(mlua::Value::UserData).map_err(|e| {
                    crate::node::ComputeNodeBuilderError::RuntimeError {
                        msg: format!("Failed to wrap UVec2: {e}"),
                    }
                })?,
                crate::node::Argument::String(x) => lua.create_string(&x).map(mlua::Value::String).map_err(|e| {
                    crate::node::ComputeNodeBuilderError::RuntimeError {
                        msg: format!("Failed to wrap String: {e}"),
                    }
                })?,
            };
            lua_args
                .set(k, lua_val)
                .map_err(|e| crate::node::ComputeNodeBuilderError::RuntimeError {
                    msg: format!("Failed to set Lua table field: {e}"),
                })?;
        }

        let builder_table: mlua::Table = lua.registry_value(&self.builder_table_key).map_err(|e| {
            crate::node::ComputeNodeBuilderError::RuntimeError {
                msg: format!("Failed to retrieve builder table from registry: {e}"),
            }
        })?;

        let build_descriptor_fn: mlua::Function =
            builder_table
                .get("build_descriptor")
                .map_err(|e| crate::node::ComputeNodeBuilderError::RuntimeError {
                    msg: format!("build_descriptor function not found on builder table: {e}"),
                })?;

        let descriptor_ref: mlua::UserDataRef<crate::node::ContainerNodeDescriptor> = build_descriptor_fn
            .call((builder_table.clone(), lua_args))
            .map_err(|e| crate::node::ComputeNodeBuilderError::RuntimeError {
                msg: format!("build_descriptor call failed: {e}"),
            })?;

        Ok(descriptor_ref.clone())
    }

    fn init_node(&self, node: &mut crate::node::ContainerNode) -> Result<(), crate::node::ComputeNodeError> {
        let interpreter = self.interpreter.lock().unwrap();
        let lua = &interpreter.lua;

        let builder_table: mlua::Table = lua
            .registry_value(&self.builder_table_key)
            .map_err(|e: mlua::Error| crate::node::ComputeNodeError::DispatchFailed(e.to_string()))?;

        if let Ok(on_node_init_fn) = builder_table.get::<mlua::Function>("on_node_init") {
            let lua_node = unsafe { crate::interpreter::wrappers::container_node::LuaContainerNode::new(node) };
            let lua_node_userdata = lua
                .create_userdata(lua_node)
                .map_err(|e: mlua::Error| crate::node::ComputeNodeError::DispatchFailed(e.to_string()))?;
            on_node_init_fn
                .call::<()>((builder_table, lua_node_userdata))
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
        globals::register_session_globals(&self.lua)
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

    pub fn load_compute_node_builder(
        &self,
        script_content: &str,
        name: &str,
    ) -> Result<mlua::RegistryKey, InterpreterError> {
        self.lua
            .load(script_content)
            .set_name("builder")
            .exec()
            .map_err(|e| InterpreterError::RuntimeError { msg: e.to_string() })?;

        let ll: mlua::Table =
            self.lua
                .load("return require('@lib/ll.luau')")
                .eval()
                .map_err(|e| InterpreterError::RuntimeError {
                    msg: format!("Failed to require @lib/ll.luau: {e}"),
                })?;
        let compute_node_builders: mlua::Table =
            ll.get("compute_node_builders")
                .map_err(|e| InterpreterError::RuntimeError {
                    msg: format!("Failed to get compute_node_builders table: {e}"),
                })?;

        let mut builder_val: mlua::Value =
            compute_node_builders
                .get(name)
                .map_err(|e| InterpreterError::RuntimeError {
                    msg: format!("Failed to look up builder {name}: {e}"),
                })?;

        if builder_val.is_nil() {
            let last_part = name.split('/').next_back().unwrap_or(name);
            builder_val = compute_node_builders
                .get(last_part)
                .map_err(|e| InterpreterError::RuntimeError {
                    msg: format!("Failed to look up builder {last_part}: {e}"),
                })?;
        }

        if builder_val.is_nil() {
            return Err(InterpreterError::RuntimeError {
                msg: format!("Builder not found in compute_node_builders for name: {name}"),
            });
        }

        let builder_table = match builder_val {
            mlua::Value::Table(t) => t,
            _ => {
                return Err(InterpreterError::RuntimeError {
                    msg: "Builder is not a table".to_string(),
                });
            }
        };

        self.lua
            .create_registry_value(builder_table)
            .map_err(|e| InterpreterError::RuntimeError { msg: e.to_string() })
    }

    pub fn load_container_node_builder(
        &self,
        script_content: &str,
        name: &str,
    ) -> Result<mlua::RegistryKey, InterpreterError> {
        self.lua
            .load(script_content)
            .set_name("builder")
            .exec()
            .map_err(|e| InterpreterError::RuntimeError { msg: e.to_string() })?;

        let ll: mlua::Table =
            self.lua
                .load("return require('@lib/ll.luau')")
                .eval()
                .map_err(|e| InterpreterError::RuntimeError {
                    msg: format!("Failed to require @lib/ll.luau: {e}"),
                })?;
        let container_node_builders: mlua::Table =
            ll.get("container_node_builders")
                .map_err(|e| InterpreterError::RuntimeError {
                    msg: format!("Failed to get container_node_builders table: {e}"),
                })?;

        let mut builder_val: mlua::Value =
            container_node_builders
                .get(name)
                .map_err(|e| InterpreterError::RuntimeError {
                    msg: format!("Failed to look up builder {name}: {e}"),
                })?;

        if builder_val.is_nil() {
            let last_part = name.split('/').next_back().unwrap_or(name);
            builder_val = container_node_builders
                .get(last_part)
                .map_err(|e| InterpreterError::RuntimeError {
                    msg: format!("Failed to look up builder {last_part}: {e}"),
                })?;
        }

        if builder_val.is_nil() {
            return Err(InterpreterError::RuntimeError {
                msg: format!("Builder not found in container_node_builders for name: {name}"),
            });
        }

        let builder_table = match builder_val {
            mlua::Value::Table(t) => t,
            _ => {
                return Err(InterpreterError::RuntimeError {
                    msg: "Builder is not a table".to_string(),
                });
            }
        };

        self.lua
            .create_registry_value(builder_table)
            .map_err(|e| InterpreterError::RuntimeError { msg: e.to_string() })
    }
}
