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
            })?;

        let create_compute_node_fn = self
            .lua
            .create_function(|lua, (builder_name, args_val): (String, Option<mlua::Table>)| {
                // Block 1: Session Retrieval
                // Retrieve the Weak reference to the Session registered in the Lua VM app data,
                // and upgrade it to ensure the Session is still active.
                let weak = lua
                    .app_data_ref::<Weak<crate::session::Session>>()
                    .ok_or_else(|| mlua::Error::RuntimeError("Session not registered".to_string()))?;

                let session = weak
                    .upgrade()
                    .ok_or_else(|| mlua::Error::RuntimeError("Session has been dropped".to_string()))?;

                // Block 2: Argument Parsing and Conversion from Lua to Rust Node Arguments
                // Convert the incoming Lua arguments table into a Rust HashMap of Argument types,
                // supporting integers, floats, booleans, and custom UserData types (Vec3, UVec3, UVec2).
                let mut args = std::collections::HashMap::new();
                if let Some(table) = args_val {
                    for pair in table.pairs::<String, mlua::Value>() {
                        let (k, v) = pair?;
                        let arg = match v {
                            mlua::Value::Number(n) => {
                                if n.fract() == 0.0 {
                                    crate::node::Argument::I32(n as i32)
                                } else {
                                    crate::node::Argument::F32(n as f32)
                                }
                            }
                            mlua::Value::Integer(i) => crate::node::Argument::I32(i as i32),
                            mlua::Value::Boolean(b) => crate::node::Argument::Bool(b),
                            mlua::Value::UserData(ud) => {
                                if let Ok(v3) = ud.borrow::<crate::math::Vec3>() {
                                    crate::node::Argument::Vec3(*v3)
                                } else if let Ok(uv3) = ud.borrow::<crate::math::UVec3>() {
                                    crate::node::Argument::UVec3(*uv3)
                                } else if let Ok(uv2) = ud.borrow::<crate::math::UVec2>() {
                                    crate::node::Argument::UVec2(*uv2)
                                } else {
                                    return Err(mlua::Error::RuntimeError(
                                        "Unsupported UserData argument type".to_string(),
                                    ));
                                }
                            }
                            _ => return Err(mlua::Error::RuntimeError("Unsupported argument type".to_string())),
                        };
                        args.insert(k, arg);
                    }
                }

                // Block 3: Dynamic Luau Script Resolution and Execution
                // Resolve and load the Luau builder script corresponding to the node builder name
                // from the Session's repositories, then execute it within the Lua state.
                let name = &builder_name;
                let name_parts: Vec<&str> = name.split('/').collect();
                let last_part = name_parts
                    .last()
                    .ok_or_else(|| mlua::Error::RuntimeError(format!("Invalid compute node builder name: {name}")))?;

                let luau_path = format!("{name}/{last_part}.luau");

                let luau_bytes = session
                    .repositories
                    .iter()
                    .find_map(|repo| repo.load(&luau_path).ok())
                    .ok_or_else(|| mlua::Error::RuntimeError(format!("Builder not found: {name}")))?;

                let script_content = std::str::from_utf8(&luau_bytes)
                    .map_err(|e| mlua::Error::RuntimeError(format!("Invalid Luau script content: {e:?}")))?;

                lua.load(script_content).set_name("builder").exec()?;

                // Block 4: Builder Retrieval from Luau Registry
                // Retrieve the registered builder table from the library's registry table.
                let ll: mlua::Table = lua.load("return require('@lib/ll.luau')").eval()?;

                let compute_node_builders: mlua::Table = ll.get("compute_node_builders")?;

                let mut builder_val: mlua::Value = compute_node_builders.get(name.as_str())?;
                if builder_val.is_nil() {
                    let last_part = name.split('/').next_back().unwrap_or(name);
                    builder_val = compute_node_builders.get(last_part)?;
                }

                if builder_val.is_nil() {
                    return Err(mlua::Error::RuntimeError(format!(
                        "Builder not found in compute_node_builders for name: {name}"
                    )));
                }

                let builder_table = match builder_val {
                    mlua::Value::Table(t) => t,
                    _ => return Err(mlua::Error::RuntimeError("Builder is not a table".to_string())),
                };

                // Block 5: Invoking build_descriptor to obtain Node Descriptor
                // Call the builder's `build_descriptor` function, converting the Rust arguments
                // back into a Lua table to pass along.
                let build_descriptor_fn: mlua::Function = builder_table.get("build_descriptor")?;
                let lua_args = lua.create_table()?;
                for (k, v) in args {
                    let lua_val = match v {
                        crate::node::Argument::F32(x) => mlua::Value::Number(x as f64),
                        crate::node::Argument::I32(x) => mlua::Value::Integer(x as i64),
                        crate::node::Argument::Bool(x) => mlua::Value::Boolean(x),
                        crate::node::Argument::Vec3(x) => lua.create_userdata(x).map(mlua::Value::UserData)?,
                        crate::node::Argument::UVec3(x) => lua.create_userdata(x).map(mlua::Value::UserData)?,
                        crate::node::Argument::UVec2(x) => lua.create_userdata(x).map(mlua::Value::UserData)?,
                    };
                    lua_args.set(k, lua_val)?;
                }

                let descriptor: mlua::UserDataRef<crate::node::ComputeNodeDescriptor> =
                    build_descriptor_fn.call((builder_table.clone(), lua_args))?;

                // Block 6: Compute Node Creation and Wrapping
                // Request the Session to instantiate the compute node from the resolved descriptor,
                // and return a Lua-managed owned wrapper.
                let compute_node = session
                    .create_compute_node(descriptor.clone())
                    .map_err(|e| mlua::Error::RuntimeError(format!("Failed to create compute node: {}", e)))?;

                let lua_node =
                    crate::interpreter::wrappers::compute_node::LuaComputeNode::new_owned(compute_node, builder_name);
                Ok(lua_node)
            })
            .map_err(|e| InterpreterError::RuntimeError {
                msg: format!("Failed to create create_compute_node closure: {e}"),
            })?;

        globals
            .set("create_compute_node", create_compute_node_fn)
            .map_err(|e| InterpreterError::RuntimeError {
                msg: format!("Failed to register create_compute_node global: {e}"),
            })?;

        let create_container_node_fn = self
            .lua
            .create_function(|lua, (builder_name, args_val): (String, Option<mlua::Table>)| {
                let weak = lua
                    .app_data_ref::<Weak<crate::session::Session>>()
                    .ok_or_else(|| mlua::Error::RuntimeError("Session not registered".to_string()))?;

                let session = weak
                    .upgrade()
                    .ok_or_else(|| mlua::Error::RuntimeError("Session has been dropped".to_string()))?;

                let mut args = std::collections::HashMap::new();
                if let Some(table) = args_val {
                    for pair in table.pairs::<String, mlua::Value>() {
                        let (k, v) = pair?;
                        let arg = match v {
                            mlua::Value::Number(n) => {
                                if n.fract() == 0.0 {
                                    crate::node::Argument::I32(n as i32)
                                } else {
                                    crate::node::Argument::F32(n as f32)
                                }
                            }
                            mlua::Value::Integer(i) => crate::node::Argument::I32(i as i32),
                            mlua::Value::Boolean(b) => crate::node::Argument::Bool(b),
                            mlua::Value::UserData(ud) => {
                                if let Ok(v3) = ud.borrow::<crate::math::Vec3>() {
                                    crate::node::Argument::Vec3(*v3)
                                } else if let Ok(uv3) = ud.borrow::<crate::math::UVec3>() {
                                    crate::node::Argument::UVec3(*uv3)
                                } else if let Ok(uv2) = ud.borrow::<crate::math::UVec2>() {
                                    crate::node::Argument::UVec2(*uv2)
                                } else {
                                    return Err(mlua::Error::RuntimeError(
                                        "Unsupported UserData argument type".to_string(),
                                    ));
                                }
                            }
                            _ => return Err(mlua::Error::RuntimeError("Unsupported argument type".to_string())),
                        };
                        args.insert(k, arg);
                    }
                }

                // Resolve builder in the Lua VM directly without locking the interpreter mutex
                let name = &builder_name;
                let name_parts: Vec<&str> = name.split('/').collect();
                let last_part = name_parts
                    .last()
                    .ok_or_else(|| mlua::Error::RuntimeError(format!("Invalid container node builder name: {name}")))?;

                let luau_path = format!("{name}/{last_part}.luau");

                let luau_bytes = session
                    .repositories
                    .iter()
                    .find_map(|repo| repo.load(&luau_path).ok())
                    .ok_or_else(|| mlua::Error::RuntimeError(format!("Builder not found: {name}")))?;

                let script_content = std::str::from_utf8(&luau_bytes)
                    .map_err(|e| mlua::Error::RuntimeError(format!("Invalid Luau script content: {e:?}")))?;

                lua.load(script_content).set_name("builder").exec()?;

                let ll: mlua::Table = lua.load("return require('@lib/ll.luau')").eval()?;

                let container_node_builders: mlua::Table = ll.get("container_node_builders")?;

                let mut builder_val: mlua::Value = container_node_builders.get(name.as_str())?;
                if builder_val.is_nil() {
                    let last_part = name.split('/').next_back().unwrap_or(name);
                    builder_val = container_node_builders.get(last_part)?;
                }

                if builder_val.is_nil() {
                    return Err(mlua::Error::RuntimeError(format!(
                        "Builder not found in container_node_builders for name: {name}"
                    )));
                }

                let builder_table = match builder_val {
                    mlua::Value::Table(t) => t,
                    _ => return Err(mlua::Error::RuntimeError("Builder is not a table".to_string())),
                };

                let build_descriptor_fn: mlua::Function = builder_table.get("build_descriptor")?;
                let lua_args = lua.create_table()?;
                for (k, v) in args {
                    let lua_val = match v {
                        crate::node::Argument::F32(x) => mlua::Value::Number(x as f64),
                        crate::node::Argument::I32(x) => mlua::Value::Integer(x as i64),
                        crate::node::Argument::Bool(x) => mlua::Value::Boolean(x),
                        crate::node::Argument::Vec3(x) => lua.create_userdata(x).map(mlua::Value::UserData)?,
                        crate::node::Argument::UVec3(x) => lua.create_userdata(x).map(mlua::Value::UserData)?,
                        crate::node::Argument::UVec2(x) => lua.create_userdata(x).map(mlua::Value::UserData)?,
                    };
                    lua_args.set(k, lua_val)?;
                }

                let descriptor: mlua::UserDataRef<crate::node::ContainerNodeDescriptor> =
                    build_descriptor_fn.call((builder_table.clone(), lua_args))?;
                let container_node = session
                    .create_container_node(descriptor.clone())
                    .map_err(|e| mlua::Error::RuntimeError(format!("Failed to create container node: {}", e)))?;

                let lua_node =
                    crate::interpreter::wrappers::container_node::LuaContainerNode::new_owned(container_node);
                Ok(lua_node)
            })
            .map_err(|e| InterpreterError::RuntimeError {
                msg: format!("Failed to create create_container_node closure: {e}"),
            })?;

        globals
            .set("create_container_node", create_container_node_fn)
            .map_err(|e| InterpreterError::RuntimeError {
                msg: format!("Failed to register create_container_node global: {e}"),
            })?;

        let create_image_view_fn = self
            .lua
            .create_function(
                |lua, (width, height, depth, channel_count, channel_type_str): (u32, u32, u32, u32, String)| {
                    let weak = lua
                        .app_data_ref::<Weak<crate::session::Session>>()
                        .ok_or_else(|| mlua::Error::RuntimeError("Session not registered".to_string()))?;

                    let session = weak
                        .upgrade()
                        .ok_or_else(|| mlua::Error::RuntimeError("Session has been dropped".to_string()))?;

                    let channel_count = match channel_count {
                        1 => crate::image::ChannelCount::C1,
                        2 => crate::image::ChannelCount::C2,
                        3 => crate::image::ChannelCount::C3,
                        4 => crate::image::ChannelCount::C4,
                        _ => {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Invalid channel count: {channel_count}"
                            )));
                        }
                    };

                    let channel_type = match channel_type_str.as_str() {
                        "Uint8" => crate::image::ChannelType::Uint8,
                        "Int8" => crate::image::ChannelType::Int8,
                        "Uint16" => crate::image::ChannelType::Uint16,
                        "Int16" => crate::image::ChannelType::Int16,
                        "Float16" => crate::image::ChannelType::Float16,
                        "Uint32" => crate::image::ChannelType::Uint32,
                        "Int32" => crate::image::ChannelType::Int32,
                        "Float32" => crate::image::ChannelType::Float32,
                        "Uint64" => crate::image::ChannelType::Uint64,
                        "Int64" => crate::image::ChannelType::Int64,
                        "Float64" => crate::image::ChannelType::Float64,
                        _ => {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Invalid channel type: {channel_type_str}"
                            )));
                        }
                    };

                    let desc = crate::image::ImageDescriptor::builder()
                        .width(width)
                        .height(height)
                        .depth(depth)
                        .channel_count(channel_count)
                        .channel_type(channel_type)
                        .usage(
                            vulkano::image::ImageUsage::STORAGE
                                | vulkano::image::ImageUsage::SAMPLED
                                | vulkano::image::ImageUsage::TRANSFER_SRC
                                | vulkano::image::ImageUsage::TRANSFER_DST,
                        )
                        .build();

                    let image = session
                        .create_image(desc)
                        .map_err(|e| mlua::Error::RuntimeError(format!("Failed to create image: {e}")))?;

                    let view_desc = crate::image::ImageViewDescriptor::default();
                    let view = image
                        .create_image_view(&view_desc)
                        .map_err(|e| mlua::Error::RuntimeError(format!("Failed to create image view: {e}")))?;

                    let lua_view = crate::interpreter::wrappers::image_view::LuaImageView(view);
                    Ok(lua_view)
                },
            )
            .map_err(|e| InterpreterError::RuntimeError {
                msg: format!("Failed to create create_image_view closure: {e}"),
            })?;

        globals
            .set("create_image_view", create_image_view_fn)
            .map_err(|e| InterpreterError::RuntimeError {
                msg: format!("Failed to register create_image_view global: {e}"),
            })?;

        Ok(())
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
