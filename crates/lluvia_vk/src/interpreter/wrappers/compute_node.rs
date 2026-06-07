use crate::interpreter::wrappers::buffer::LuaBuffer;
use crate::node::{ComputeNode, Node, NodePort};
use mlua::FromLua;

pub struct LuaComputeNode {
    node_ptr: *mut ComputeNode,
}

unsafe impl Send for LuaComputeNode {}

impl LuaComputeNode {
    /// Creates a new `LuaComputeNode` wrapper around a raw pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the returned `LuaComputeNode` does not outlive the lifetime
    /// of the borrowed `ComputeNode`.
    pub unsafe fn new(node: &mut ComputeNode) -> Self {
        Self { node_ptr: node }
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
    }
}
