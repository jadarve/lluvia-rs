//! Descriptor used to build a [`ComputeNode`](super::ComputeNode).

use bon::Builder;
use std::collections::HashMap;

use crate::math;
use crate::program::Program;

use super::ComputeNodeError;
use super::constant::Constant;
use super::port_descriptor::PortDescriptor;

// ---------------------------------------------------------------------------
// ComputeNodeDescriptor
// ---------------------------------------------------------------------------

/// Descriptor used to build a [`ComputeNode`](super::ComputeNode).
///
/// Mirrors C++ `ll::ComputeNodeDescriptor`.
#[derive(Clone, Builder)]
pub struct ComputeNodeDescriptor {
    #[builder(field)]
    pub ports: Vec<PortDescriptor>,

    #[builder(field)]
    pub constants: HashMap<String, Constant>,

    pub global_shape: math::UVec3,

    #[builder(into)]
    pub program: Option<Program>,

    #[builder(default = "main".to_string(), into)]
    pub function_name: String,
}

impl Default for ComputeNodeDescriptor {
    fn default() -> Self {
        Self {
            program: None,
            function_name: "main".to_string(),
            global_shape: math::UVec3::ONE,
            ports: Vec::new(),
            constants: HashMap::new(),
        }
    }
}

impl<State: compute_node_descriptor_builder::State> ComputeNodeDescriptorBuilder<State> {
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

impl ComputeNodeDescriptor {
    /// Adds a port descriptor.
    pub fn add_port(mut self, port: PortDescriptor) -> Self {
        self.ports.push(port);
        self
    }

    /// Sets a constant.
    pub fn set_constant(&mut self, name: impl Into<String>, value: Constant) {
        self.constants.insert(name.into(), value);
    }

    /// Gets a constant reference by name.
    pub fn get_constant(&self, name: &str) -> Result<&Constant, ComputeNodeError> {
        self.constants
            .get(name)
            .ok_or_else(|| ComputeNodeError::ConstantNotFound(name.to_string()))
    }

    pub(super) fn validate(&self) -> Result<(), ComputeNodeError> {
        if self.program.is_none() {
            return Err(ComputeNodeError::InvalidProgram);
        }
        if self.function_name.is_empty() {
            return Err(ComputeNodeError::InvalidFunctionName);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn descriptor_without_program() -> ComputeNodeDescriptor {
        ComputeNodeDescriptor::default()
    }

    #[test]
    fn validate_requires_program() {
        let desc = descriptor_without_program();
        assert!(matches!(desc.validate(), Err(ComputeNodeError::InvalidProgram)));
    }

    #[test]
    fn builder_sets_default_values() {
        let desc = ComputeNodeDescriptor::builder().global_shape(math::UVec3::ONE).build();
        assert!(desc.program.is_none());
        assert_eq!(desc.function_name, "main");
        assert!(desc.ports.is_empty());
        assert!(desc.constants.is_empty());
    }

    #[test]
    fn builder_adds_port_and_constant() {
        let port = PortDescriptor::default();
        let val = Constant::Int(42);

        let desc = ComputeNodeDescriptor::builder()
            .global_shape(math::UVec3::ONE)
            .add_port(port)
            .add_constant("my_const", val)
            .build();

        assert_eq!(desc.ports.len(), 1);
        assert_eq!(desc.constants.len(), 1);
        assert!(desc.get_constant("my_const").is_ok());
    }
}
