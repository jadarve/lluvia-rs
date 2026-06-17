use thiserror::Error;

use super::*;

// ---------------------------------------------------------------------------
// ComputeNodeBuilderImpl trait
// ---------------------------------------------------------------------------

/// Error returned by [`ComputeNodeBuilderImpl`] operations.
#[derive(Error, Debug)]
pub enum ComputeNodeBuilderError {
    #[error("Runtime error: {msg}")]
    RuntimeError { msg: String },
}

/// Trait for compute node builders, mirroring C++ and Luau builders.
pub trait ComputeNodeBuilderImpl: Send {
    /// Returns the node descriptor configured by the builder.
    fn build_descriptor(
        &self,
        args: std::collections::HashMap<String, Argument>,
    ) -> Result<ComputeNodeDescriptor, ComputeNodeBuilderError>;

    /// Initializes the compute node (e.g., configures its grid shape or other state based on bound ports).
    fn init_node(&self, node: &mut ComputeNode) -> Result<(), ComputeNodeError>;
}

// ---------------------------------------------------------------------------
// ComputeNodeBuilder
// ---------------------------------------------------------------------------

#[derive(bon::Builder)]
#[builder(finish_fn(vis = "", name = build_internal))]
pub struct ComputeNodeBuilder {
    #[builder(field)]
    #[allow(dead_code)]
    descriptor: Option<ComputeNodeDescriptor>,

    #[builder(field)]
    #[allow(dead_code)]
    node: Option<ComputeNode>,

    #[builder(setters(name = set_builder))]
    builder: std::sync::Arc<dyn ComputeNodeBuilderImpl>,

    #[builder(setters(name = set_session))]
    session: std::sync::Arc<crate::session::Session>,
}

impl ComputeNodeBuilder {
    pub fn new(builder: Box<dyn ComputeNodeBuilderImpl>, session: std::sync::Arc<crate::session::Session>) -> Self {
        Self {
            descriptor: None,
            node: None,
            builder: std::sync::Arc::from(builder),
            session,
        }
    }

    pub fn build_descriptor(
        self,
        args: std::collections::HashMap<String, Argument>,
    ) -> Result<ComputeNodeBuilderBuilder<impl compute_node_builder_builder::IsComplete>, ComputeNodeBuilderError> {
        let descriptor = self.builder.build_descriptor(args)?;
        let node = self
            .session
            .create_compute_node(descriptor.clone())
            .map_err(|e| ComputeNodeBuilderError::RuntimeError { msg: e.to_string() })?;

        let mut builder = Self::builder().set_builder(self.builder).set_session(self.session);

        builder.descriptor = Some(descriptor);
        builder.node = Some(node);

        Ok(builder)
    }
}

impl<State: compute_node_builder_builder::State> ComputeNodeBuilderBuilder<State> {
    pub fn node_type(&self) -> NodeType {
        self.node.as_ref().expect("Node not initialized").node_type()
    }

    pub fn bind(mut self, name: &str, obj: NodePort) -> Result<Self, ComputeNodeError> {
        if let Some(ref mut node) = self.node {
            node.bind(name, obj)?;
        }
        Ok(self)
    }

    pub fn has_port(&self, name: &str) -> bool {
        self.node.as_ref().map(|n| n.has_port(name)).unwrap_or(false)
    }

    pub fn port(&self, name: &str) -> Option<&NodePort> {
        self.node.as_ref().and_then(|n| n.port(name))
    }

    pub fn set_constant(mut self, name: impl Into<String>, value: Constant) -> Self {
        if let Some(ref mut node) = self.node {
            node.set_constant(name, value);
        }
        self
    }
}

impl<State: compute_node_builder_builder::IsComplete> ComputeNodeBuilderBuilder<State> {
    pub fn build(self) -> Result<ComputeNode, ComputeNodeError> {
        let mut director = self.build_internal();
        let mut node = director
            .node
            .take()
            .ok_or_else(|| ComputeNodeError::CreationFailed("Node not initialized".to_string()))?;
        director.builder.init_node(&mut node)?;
        Ok(node)
    }
}
