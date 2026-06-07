//! Compute node abstractions.
//!
//! This module mirrors the C++ `ll::ComputeNodeDescriptor` and
//! `ll::ComputeNode` classes.

use thiserror::Error;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::PrimaryAutoCommandBuffer;

mod compute_node;
mod compute_node_descriptor;
mod constant;
mod node_port;
mod node_type;
mod port_descriptor;

pub use compute_node::*;
pub use compute_node_descriptor::*;
pub use constant::*;
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

    #[error("Invalid local shape: all components must be > 0")]
    InvalidLocalShape,

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
    fn record(&self, builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) -> Result<(), ComputeNodeError>;

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
    fn get_descriptor(&self) -> Result<ComputeNodeDescriptor, ComputeNodeBuilderError>;

    /// Initializes the compute node (e.g., configures its grid shape or other state based on bound ports).
    fn init_node(&self, node: &mut ComputeNode) -> Result<(), ComputeNodeError>;
}

pub trait ComputeNodeBuilder2: Send + Sized {
    // Build the descriptor, keep it internally
    fn build_descriptor(&mut self) -> Result<&mut Self, ComputeNodeBuilderError>;

    /// Returns the node descriptor, if not initialized, it calls init_descriptor() first.
    fn get_descriptor(&mut self) -> Result<ComputeNodeDescriptor, ComputeNodeBuilderError>;

    /// Initializes the compute node (e.g., configures its grid shape or other state based on bound ports).
    fn init_node(&self, node: &mut ComputeNode) -> Result<(), ComputeNodeError>;

    /// Sets a constant. It can be called after init_descriptor() and before build() is called.
    fn set_constant(&mut self, name: impl Into<String>, value: Constant) -> Result<&mut Self, ComputeNodeBuilderError>;

    fn bind(&mut self, name: &str, obj: NodePort) -> Result<&mut Self, ComputeNodeBuilderError>;

    fn build(&mut self) -> Result<ComputeNode, ComputeNodeBuilderError>;
}
