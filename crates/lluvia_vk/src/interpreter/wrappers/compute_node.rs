#![allow(clippy::collapsible_if)]
use crate::interpreter::wrappers::buffer::LuaBuffer;
use crate::node::{ComputeNode, Node, NodePort};
use mlua::FromLua;

pub struct LuaComputeNode {
    pub(crate) node_ptr: *mut ComputeNode,
    pub(crate) builder_name: String,
    pub(crate) owned: Option<std::cell::RefCell<Option<Box<ComputeNode>>>>,
}

unsafe impl Send for LuaComputeNode {}

impl LuaComputeNode {
    /// Creates a new `LuaComputeNode` wrapper around a raw pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the returned `LuaComputeNode` does not outlive the lifetime
    /// of the borrowed `ComputeNode`.
    pub unsafe fn new(node: *mut ComputeNode, builder_name: String) -> Self {
        Self {
            node_ptr: node,
            builder_name,
            owned: None,
        }
    }

    /// Creates a new `LuaComputeNode` that owns the underlying `ComputeNode`.
    pub fn new_owned(node: ComputeNode, builder_name: String) -> Self {
        let mut boxed = Box::new(node);
        let ptr = boxed.as_mut() as *mut ComputeNode;
        Self {
            node_ptr: ptr,
            builder_name,
            owned: Some(std::cell::RefCell::new(Some(boxed))),
        }
    }
}

impl mlua::UserData for LuaComputeNode {
    fn add_fields<F: mlua::prelude::LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("push_constants", |_, this| {
            let node = unsafe { &*this.node_ptr };
            Ok(node.push_constants.clone())
        });
        fields.add_field_method_set(
            "push_constants",
            |_, this, val: Option<mlua::UserDataRef<crate::node::PushConstants>>| {
                let node = unsafe { &mut *this.node_ptr };
                node.push_constants = val.map(|v| v.clone());
                Ok(())
            },
        );
    }

    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("get_port", |_, this, name: String| {
            let node = unsafe { &*this.node_ptr };
            if let Some(NodePort::Buffer(buf)) = node.port(&name) {
                Ok(Some(LuaBuffer(buf.clone())))
            } else {
                Ok(None)
            }
        });

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

        methods.add_method("init", |lua, this, ()| {
            let node = unsafe { &mut *this.node_ptr };
            if this.builder_name.is_empty() {
                return Ok(());
            }

            let weak = lua
                .app_data_ref::<std::sync::Weak<crate::session::Session>>()
                .ok_or_else(|| mlua::Error::RuntimeError("Session not registered".to_string()))?;
            let session = weak
                .upgrade()
                .ok_or_else(|| mlua::Error::RuntimeError("Session has been dropped".to_string()))?;

            // Resolve builder directly in the Lua VM without locking interpreter
            let name = &this.builder_name;
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

            if let Ok(on_node_init_fn) = builder_table.get::<mlua::Function>("on_node_init") {
                let lua_node = unsafe { LuaComputeNode::new(node, this.builder_name.clone()) };
                let lua_node_userdata = lua.create_userdata(lua_node)?;
                on_node_init_fn.call::<()>((builder_table, lua_node_userdata))?;
            }

            Ok(())
        });
    }
}
