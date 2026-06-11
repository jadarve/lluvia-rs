//! Compute node abstractions.
//!
//! This module mirrors the C++ `ll::ComputeNodeDescriptor` and
//! `ll::ComputeNode` classes.

use thiserror::Error;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::PrimaryAutoCommandBuffer;

mod argument;
mod compute_node;
mod compute_node_descriptor;
mod constant;
mod container_node;
mod container_node_descriptor;
mod node_port;
mod node_type;
mod port_descriptor;

pub use argument::*;
pub use compute_node::*;
pub use compute_node_descriptor::*;
pub use constant::*;
pub use container_node::*;
pub use container_node_descriptor::*;
pub use node_port::*;
pub use node_type::*;
pub use port_descriptor::*;

use crate::math;

// ---------------------------------------------------------------------------
// ComputeNodeError
// ---------------------------------------------------------------------------

#[derive(Error, Debug)]
pub enum ComputeNodeError {
    #[error("Compute node creation failed: {0}")]
    CreationFailed(String),

    #[error("Invalid shader program")]
    InvalidProgram,

    #[error("Invalid function name")]
    InvalidFunctionName,

    #[error("Invalid port direction: {0}")]
    InvalidPortDirection(u32),

    #[error("Invalid port type: {0}")]
    InvalidPortType(u32),

    #[error("Invalid node type: {0}")]
    InvalidNodeType(u32),

    #[error("Dispatch failed: {0}")]
    DispatchFailed(String),

    #[error("Port not found: {0}")]
    PortNotFound(String),

    #[error("Constant not found: {0}")]
    ConstantNotFound(String),

    #[error("Invalid global shape: {0}")]
    InvalidGlobalShape(math::UVec3),
}

// ---------------------------------------------------------------------------
// Node trait
// ---------------------------------------------------------------------------

pub trait Node {
    fn node_type(&self) -> NodeType;
    fn bind(&mut self, name: &str, obj: NodePort) -> Result<(), ComputeNodeError>;
    fn has_port(&self, name: &str) -> bool;
    fn port(&self, name: &str) -> Option<&NodePort>;
    fn record(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) -> Result<(), ComputeNodeError>;

    // FIXME: constant related methods should be declared here
}

// ---------------------------------------------------------------------------
// ComputeNodeBuilder trait
// ---------------------------------------------------------------------------

/// Error returned by [`ComputeNodeBuilder`] operations.
#[derive(Error, Debug)]
pub enum ComputeNodeBuilderError {
    #[error("Runtime error: {msg}")]
    RuntimeError { msg: String },
}

/// Trait for compute node builders, mirroring C++ and Luau builders.
pub trait ComputeNodeBuilder: Send {
    /// Returns the node descriptor configured by the builder.
    fn build_descriptor(
        &self,
        args: std::collections::HashMap<String, Argument>,
    ) -> Result<ComputeNodeDescriptor, ComputeNodeBuilderError>;

    /// Initializes the compute node (e.g., configures its grid shape or other state based on bound ports).
    fn init_node(&self, node: &mut ComputeNode) -> Result<(), ComputeNodeError>;
}

/// Trait for container node builders, mirroring C++ and Luau builders.
pub trait ContainerNodeBuilder: Send {
    /// Returns the node descriptor configured by the builder.
    fn build_descriptor(
        &self,
        args: std::collections::HashMap<String, Argument>,
    ) -> Result<ContainerNodeDescriptor, ComputeNodeBuilderError>;

    /// Initializes the container node (e.g., creates sub-nodes, binds internal ports, etc.).
    fn init_node(&self, node: &mut ContainerNode) -> Result<(), ComputeNodeError>;
}
