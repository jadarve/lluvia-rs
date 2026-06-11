#![allow(clippy::collapsible_if)]
use crate::node::{ComputeNode, ContainerNode, Node, NodePort};
use mlua::FromLua;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::PrimaryAutoCommandBuffer;

pub struct LuaContainerNode {
    pub(crate) node_ptr: *mut ContainerNode,
    pub(crate) owned: Option<std::cell::RefCell<Option<Box<ContainerNode>>>>,
}

unsafe impl Send for LuaContainerNode {}

impl LuaContainerNode {
    /// Creates a new `LuaContainerNode` wrapper around a raw pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the returned `LuaContainerNode` does not outlive the lifetime
    /// of the borrowed `ContainerNode`.
    pub unsafe fn new(node: *mut ContainerNode) -> Self {
        Self {
            node_ptr: node,
            owned: None,
        }
    }

    /// Creates a new `LuaContainerNode` that owns the underlying `ContainerNode`.
    pub fn new_owned(node: ContainerNode) -> Self {
        let mut boxed = Box::new(node);
        let ptr = boxed.as_mut() as *mut ContainerNode;
        Self {
            node_ptr: ptr,
            owned: Some(std::cell::RefCell::new(Some(boxed))),
        }
    }
}

impl mlua::UserData for LuaContainerNode {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("get_constant", |_, this, name: String| {
            let node = unsafe { &*this.node_ptr };
            let constant = node
                .get_constant(&name)
                .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
            Ok(constant.clone())
        });

        methods.add_method("set_constant", |lua, this, (name, value): (String, mlua::Value)| {
            let node = unsafe { &mut *this.node_ptr };
            let constant = crate::node::Constant::from_lua(value, lua)?;
            node.set_constant(name, constant);
            Ok(())
        });

        methods.add_method("get_port", |_, this, name: String| {
            let node = unsafe { &*this.node_ptr };
            if let Some(NodePort::Buffer(buf)) = node.port(&name) {
                Ok(Some(crate::interpreter::wrappers::buffer::LuaBuffer(buf.clone())))
            } else {
                Ok(None)
            }
        });

        methods.add_method("bind", |_, this, (name, obj): (String, mlua::Value)| {
            let node = unsafe { &mut *this.node_ptr };
            if let mlua::Value::UserData(ud) = &obj {
                if let Ok(lua_buf) = ud.borrow::<crate::interpreter::wrappers::buffer::LuaBuffer>() {
                    node.bind(&name, crate::node::NodePort::Buffer(lua_buf.0.clone()))
                        .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
                    return Ok(());
                }
            }
            Err(mlua::Error::RuntimeError(
                "Expected a Buffer as the second argument".to_string(),
            ))
        });

        methods.add_method("add_node", |_, this, (name, child): (String, mlua::Value)| {
            let node = unsafe { &mut *this.node_ptr };
            if let mlua::Value::UserData(ud) = &child {
                if let Ok(mut lua_compute) =
                    ud.borrow_mut::<crate::interpreter::wrappers::compute_node::LuaComputeNode>()
                {
                    let boxed_node = if let Some(ref_cell) = &lua_compute.owned {
                        ref_cell.borrow_mut().take()
                    } else {
                        None
                    };
                    if let Some(boxed_node) = boxed_node {
                        node.compute_nodes.insert(name.clone(), boxed_node);
                        let new_ptr = node.compute_nodes.get_mut(&name).unwrap().as_mut() as *mut ComputeNode;
                        lua_compute.node_ptr = new_ptr;
                        return Ok(());
                    }
                    return Err(mlua::Error::RuntimeError(
                        "ComputeNode is already bound or not owned by this wrapper".to_string(),
                    ));
                } else if let Ok(mut lua_container) = ud.borrow_mut::<LuaContainerNode>() {
                    let boxed_node = if let Some(ref_cell) = &lua_container.owned {
                        ref_cell.borrow_mut().take()
                    } else {
                        None
                    };
                    if let Some(boxed_node) = boxed_node {
                        node.container_nodes.insert(name.clone(), boxed_node);
                        let new_ptr = node.container_nodes.get_mut(&name).unwrap().as_mut() as *mut ContainerNode;
                        lua_container.node_ptr = new_ptr;
                        return Ok(());
                    }
                    return Err(mlua::Error::RuntimeError(
                        "ContainerNode is already bound or not owned by this wrapper".to_string(),
                    ));
                }
            }
            Err(mlua::Error::RuntimeError(
                "Expected a ComputeNode or ContainerNode as the second argument".to_string(),
            ))
        });

        methods.add_method("get_node", |lua, this, name: String| {
            let node = unsafe { &*this.node_ptr };
            if let Some(child) = node.compute_nodes.get(&name) {
                let ptr = &**child as *const ComputeNode as *mut ComputeNode;
                let lua_child =
                    unsafe { crate::interpreter::wrappers::compute_node::LuaComputeNode::new(ptr, String::new()) };
                lua.create_userdata(lua_child).map(mlua::Value::UserData)
            } else if let Some(child) = node.container_nodes.get(&name) {
                let ptr = &**child as *const ContainerNode as *mut ContainerNode;
                let lua_child = unsafe { LuaContainerNode::new(ptr) };
                lua.create_userdata(lua_child).map(mlua::Value::UserData)
            } else {
                Ok(mlua::Value::Nil)
            }
        });

        methods.add_method("get_constants", |lua, this, ()| {
            let node = unsafe { &*this.node_ptr };
            let table = lua.create_table()?;
            for (name, constant) in node.descriptor.constants.iter() {
                table.set(name.clone(), constant.clone())?;
            }
            Ok(table)
        });

        methods.add_method("init", |lua, this, ()| {
            let node = unsafe { &mut *this.node_ptr };
            let builder_name = node.descriptor.builder_name.clone();
            if builder_name.is_empty() {
                return Ok(());
            }

            let weak = lua
                .app_data_ref::<std::sync::Weak<crate::session::Session>>()
                .ok_or_else(|| mlua::Error::RuntimeError("Session not registered".to_string()))?;
            let session = weak
                .upgrade()
                .ok_or_else(|| mlua::Error::RuntimeError("Session has been dropped".to_string()))?;

            // Resolve builder directly in the Lua VM without locking interpreter
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

            if let Ok(on_node_init_fn) = builder_table.get::<mlua::Function>("on_node_init") {
                let lua_node = unsafe { LuaContainerNode::new(node) };
                let lua_node_userdata = lua.create_userdata(lua_node)?;
                on_node_init_fn.call::<()>((builder_table, lua_node_userdata))?;
            }

            Ok(())
        });
    }
}

pub struct LuaCommandBufferBuilder {
    pub(crate) builder_ptr: *mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
}

unsafe impl Send for LuaCommandBufferBuilder {}

impl mlua::UserData for LuaCommandBufferBuilder {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("run", |_, this, node_val: mlua::Value| {
            let builder = unsafe { &mut *this.builder_ptr };
            if let mlua::Value::UserData(ud) = &node_val {
                if let Ok(lua_compute) = ud.borrow::<crate::interpreter::wrappers::compute_node::LuaComputeNode>() {
                    // SAFETY: copy raw pointer to local variable to avoid mutability issues with Ref guard, then cast to mutable reference
                    let ptr = lua_compute.node_ptr;
                    let compute_node = unsafe { &mut *ptr };
                    compute_node
                        .record(builder)
                        .map_err(|e: crate::node::ComputeNodeError| mlua::Error::RuntimeError(e.to_string()))?;
                    Ok(())
                } else if let Ok(lua_container) = ud.borrow::<LuaContainerNode>() {
                    // SAFETY: copy raw pointer to local variable to avoid mutability issues with Ref guard, then cast to mutable reference
                    let ptr = lua_container.node_ptr;
                    let container_node = unsafe { &mut *ptr };
                    container_node
                        .record(builder)
                        .map_err(|e: crate::node::ComputeNodeError| mlua::Error::RuntimeError(e.to_string()))?;
                    Ok(())
                } else {
                    Err(mlua::Error::RuntimeError(
                        "Expected a ComputeNode or ContainerNode".to_string(),
                    ))
                }
            } else {
                Err(mlua::Error::RuntimeError(
                    "Expected a ComputeNode or ContainerNode".to_string(),
                ))
            }
        });
    }
}
