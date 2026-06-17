#![allow(clippy::collapsible_if)]
use std::collections::HashMap;
use std::sync::{Mutex, Weak};
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::PrimaryAutoCommandBuffer;

use super::ComputeNode;
use super::ComputeNodeError;
use super::Node;
use super::NodePort;
use super::NodeType;
use super::constant::Constant;
use super::container_node_descriptor::ContainerNodeDescriptor;
use crate::interpreter::Interpreter;

#[allow(dead_code)]
enum NodeWrapper {
    Compute(Box<ComputeNode>),
    Container(Box<ContainerNode>),
}

/// A container node that holds child nodes and coordinates their recording and initialization.
pub struct ContainerNode {
    pub(crate) descriptor: ContainerNodeDescriptor,
    pub(crate) ports: HashMap<String, NodePort>,

    // FIXME: two containers to store different types
    // pub(crate) nodes: HashMap<String, NodeWrapper>,
    pub(crate) compute_nodes: HashMap<String, Box<ComputeNode>>,
    pub(crate) container_nodes: HashMap<String, Box<ContainerNode>>,

    pub(crate) interpreter: Weak<Mutex<Interpreter>>,
}

impl ContainerNode {
    /// Creates a new `ContainerNode`.
    pub fn new(interpreter: Weak<Mutex<Interpreter>>, descriptor: ContainerNodeDescriptor) -> Self {
        Self {
            descriptor,
            ports: HashMap::new(),
            compute_nodes: HashMap::new(),
            container_nodes: HashMap::new(),
            interpreter,
        }
    }

    /// Returns a reference to the descriptor.
    pub fn descriptor(&self) -> &ContainerNodeDescriptor {
        &self.descriptor
    }
    /// Sets a constant value.
    pub fn set_constant(&mut self, name: impl Into<String>, value: Constant) {
        self.descriptor.constants.insert(name.into(), value);
    }

    /// Gets a constant value.
    pub fn get_constant(&self, name: &str) -> Result<&Constant, ComputeNodeError> {
        self.descriptor.get_constant(name)
    }

    /// Adds a compute node.
    pub fn add_node(&mut self, name: String, node: ComputeNode) {
        self.compute_nodes.insert(name, Box::new(node));
    }

    /// Adds a container node.
    pub fn add_container_node(&mut self, name: String, node: ContainerNode) {
        self.container_nodes.insert(name, Box::new(node));
    }

    /// Runs initialization using the Luau builder if it exists.
    pub fn init(&mut self) -> Result<(), ComputeNodeError> {
        let builder_name = self.descriptor.builder_name.clone();
        if !builder_name.is_empty() {
            if let Some(interpreter_mutex) = self.interpreter.upgrade() {
                let interpreter = interpreter_mutex.lock().unwrap();
                let lua = &interpreter.lua;

                let chunk = r#"
                    local builderName, node = ...
                    local ll = require("@lib/ll.luau")
                    local builder = ll:get_container_node_builder(builderName)
                    if builder and builder.on_node_init then
                        builder:on_node_init(node)
                    end
                "#;

                let lua_node = unsafe {
                    crate::interpreter::wrappers::container_node::LuaContainerNode::new(self as *mut ContainerNode)
                };
                let lua_node_userdata = lua
                    .create_userdata(lua_node)
                    .map_err(|e| ComputeNodeError::DispatchFailed(e.to_string()))?;

                let run_fn: mlua::Function = lua
                    .load(chunk)
                    .into_function()
                    .map_err(|e| ComputeNodeError::DispatchFailed(e.to_string()))?;

                run_fn
                    .call::<()>((builder_name, lua_node_userdata))
                    .map_err(|e| ComputeNodeError::DispatchFailed(e.to_string()))?;
            }
        }
        Ok(())
    }
}

impl Node for ContainerNode {
    fn node_type(&self) -> NodeType {
        NodeType::Container
    }

    fn bind(&mut self, name: &str, obj: NodePort) -> Result<(), ComputeNodeError> {
        self.ports.insert(name.to_string(), obj);
        Ok(())
    }

    fn has_port(&self, name: &str) -> bool {
        self.ports.contains_key(name)
    }

    fn port(&self, name: &str) -> Option<&NodePort> {
        self.ports.get(name)
    }

    fn record(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) -> Result<(), ComputeNodeError> {
        let builder_name = self.descriptor.builder_name.clone();
        if !builder_name.is_empty() {
            if let Some(interpreter_mutex) = self.interpreter.upgrade() {
                let interpreter = interpreter_mutex.lock().unwrap();
                let lua = &interpreter.lua;

                let chunk = r#"
                    local builderName, node, cmdBuffer = ...
                    local ll = require("@lib/ll.luau")
                    local builder = ll:get_container_node_builder(builderName)
                    if builder and builder.on_node_record then
                        builder:on_node_record(node, cmdBuffer)
                    end
                "#;

                // FIXME: avoid unsafe, maybe using Arc<mut ContainerNode>
                let lua_node = unsafe { crate::interpreter::wrappers::container_node::LuaContainerNode::new(self) };
                let lua_node_userdata = lua
                    .create_userdata(lua_node)
                    .map_err(|e| ComputeNodeError::DispatchFailed(e.to_string()))?;

                let lua_cmd_buf = crate::interpreter::wrappers::container_node::LuaCommandBufferBuilder {
                    builder_ptr: builder as *mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
                };
                let lua_cmd_buf_userdata = lua
                    .create_userdata(lua_cmd_buf)
                    .map_err(|e| ComputeNodeError::DispatchFailed(e.to_string()))?;

                let run_fn: mlua::Function = lua
                    .load(chunk)
                    .into_function()
                    .map_err(|e| ComputeNodeError::DispatchFailed(e.to_string()))?;

                run_fn
                    .call::<()>((builder_name, lua_node_userdata, lua_cmd_buf_userdata))
                    .map_err(|e| ComputeNodeError::DispatchFailed(e.to_string()))?;
            }
        }
        Ok(())
    }
}
