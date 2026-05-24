use crate::interpreter::wrappers::buffer::LuaBuffer;
use crate::node::{ComputeNode, Node, NodePort};

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
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("getPort", |_, this, name: String| {
            let node = unsafe { &*this.node_ptr };
            if let Some(NodePort::Buffer(buf)) = node.port(&name) {
                Ok(Some(LuaBuffer(buf.clone())))
            } else {
                Ok(None)
            }
        });

        methods.add_method(
            "configureGridShape",
            |_, this, shape: mlua::UserDataRef<crate::math::UVec3>| {
                let node = unsafe { &mut *this.node_ptr };
                node.set_grid_shape(&shape);
                Ok(())
            },
        );
    }
}
