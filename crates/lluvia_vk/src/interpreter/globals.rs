use crate::interpreter::InterpreterError;
use crate::interpreter::wrappers::compute_node::LuaComputeNode;
use crate::interpreter::wrappers::container_node::LuaContainerNode;
use crate::interpreter::wrappers::image_view::LuaImageView;
use crate::program::Program;
use std::str::FromStr;
use std::sync::Weak;

fn load_program_fn_impl(lua: &mlua::Lua, path: String) -> Result<Program, mlua::Error> {
    let weak = lua
        .app_data_ref::<Weak<crate::session::Session>>()
        .ok_or_else(|| mlua::Error::RuntimeError("Session not registered".to_string()))?;

    let session = weak
        .upgrade()
        .ok_or_else(|| mlua::Error::RuntimeError("Session has been dropped".to_string()))?;

    session
        .load_program(&path)
        .map_err(|e| mlua::Error::RuntimeError(e.to_string()))
}

fn create_compute_node_impl(
    lua: &mlua::Lua,
    builder_name: String,
    args_val: Option<mlua::Table>,
) -> Result<LuaComputeNode, mlua::Error> {
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

    let lua_node = LuaComputeNode::new_owned(compute_node, builder_name);
    Ok(lua_node)
}

fn create_container_node_impl(
    lua: &mlua::Lua,
    builder_name: String,
    args_val: Option<mlua::Table>,
) -> Result<LuaContainerNode, mlua::Error> {
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

    let lua_node = LuaContainerNode::new_owned(container_node);
    Ok(lua_node)
}

fn create_image_view_impl(
    lua: &mlua::Lua,
    width: u32,
    height: u32,
    depth: u32,
    channel_count: u32,
    channel_type_str: String,
) -> Result<LuaImageView, mlua::Error> {
    let weak = lua
        .app_data_ref::<Weak<crate::session::Session>>()
        .ok_or_else(|| mlua::Error::RuntimeError("Session not registered".to_string()))?;

    let session = weak
        .upgrade()
        .ok_or_else(|| mlua::Error::RuntimeError("Session has been dropped".to_string()))?;

    let channel_count =
        crate::image::ChannelCount::try_from(channel_count).map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;

    let channel_type =
        crate::image::ChannelType::from_str(&channel_type_str).map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;

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

    let lua_view = LuaImageView(view);
    Ok(lua_view)
}

/// Injects native globals that resolve through the stored weak Session.
pub(crate) fn register_session_globals(lua: &mlua::Lua) -> Result<(), InterpreterError> {
    let globals = lua.globals();

    let load_program_fn = lua
        .create_function(|lua, path: String| load_program_fn_impl(lua, path))
        .map_err(|e| InterpreterError::RuntimeError {
            msg: format!("Failed to create load_program closure: {e}"),
        })?;

    globals
        .set("load_program", load_program_fn)
        .map_err(|e| InterpreterError::RuntimeError {
            msg: format!("Failed to register load_program global: {e}"),
        })?;

    let create_compute_node_fn = lua
        .create_function(|lua, (builder_name, args_val): (String, Option<mlua::Table>)| {
            create_compute_node_impl(lua, builder_name, args_val)
        })
        .map_err(|e| InterpreterError::RuntimeError {
            msg: format!("Failed to create create_compute_node closure: {e}"),
        })?;

    globals
        .set("create_compute_node", create_compute_node_fn)
        .map_err(|e| InterpreterError::RuntimeError {
            msg: format!("Failed to register create_compute_node global: {e}"),
        })?;

    let create_container_node_fn = lua
        .create_function(|lua, (builder_name, args_val): (String, Option<mlua::Table>)| {
            create_container_node_impl(lua, builder_name, args_val)
        })
        .map_err(|e| InterpreterError::RuntimeError {
            msg: format!("Failed to create create_container_node closure: {e}"),
        })?;

    globals
        .set("create_container_node", create_container_node_fn)
        .map_err(|e| InterpreterError::RuntimeError {
            msg: format!("Failed to register create_container_node global: {e}"),
        })?;

    let create_image_view_fn = lua
        .create_function(
            |lua, (width, height, depth, channel_count, channel_type_str): (u32, u32, u32, u32, String)| {
                create_image_view_impl(lua, width, height, depth, channel_count, channel_type_str)
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
