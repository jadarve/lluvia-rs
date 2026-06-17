//! Descriptor used to build a [`ContainerNode`](super::ContainerNode).

use bon::Builder;
use std::collections::HashMap;

use super::constant::Constant;
use crate::node::PortDescriptor;

/// Descriptor used to build a [`ContainerNode`](super::ContainerNode).
///
/// Mirrors C++ `ll::ContainerNodeDescriptor`.
#[derive(Clone, Builder, Default)]
#[builder(derive(Clone, Debug))]
pub struct ContainerNodeDescriptor {
    #[builder(field)]
    pub ports: Vec<PortDescriptor>,

    #[builder(field)]
    pub constants: HashMap<String, Constant>,

    #[builder(into)]
    pub builder_name: String,
}

impl<State: container_node_descriptor_builder::State> ContainerNodeDescriptorBuilder<State> {
    /// Adds a port descriptor.
    pub fn add_port(mut self, port: PortDescriptor) -> Self {
        self.ports.push(port);
        self
    }

    /// Adds a constant.
    pub fn add_constant(mut self, name: impl Into<String>, value: Constant) -> Self {
        self.constants.insert(name.into(), value);
        self
    }
}

impl ContainerNodeDescriptor {
    /// Gets a port reference by name.
    pub fn get_port(&self, name: &str) -> Option<&PortDescriptor> {
        self.ports.iter().find(|p| p.name == name)
    }

    /// Gets a constant reference by name.
    pub fn get_constant(&self, name: &str) -> Result<&Constant, super::ComputeNodeError> {
        self.constants
            .get(name)
            .ok_or_else(|| super::ComputeNodeError::ConstantNotFound(name.to_string()))
    }
}
