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

#[derive(Clone, Copy, Debug)]
#[repr(u32)]
pub enum ComputeDimensions {
    ONE,
    TWO,
    THREE,
}

/// Descriptor used to build a [`ComputeNode`](super::ComputeNode).
///
/// Mirrors C++ `ll::ComputeNodeDescriptor`.
#[derive(Clone, Builder)]
pub struct ComputeNodeDescriptor {
    #[builder(field)]
    pub ports: Vec<PortDescriptor>,

    #[builder(field)]
    pub constants: HashMap<String, Constant>,

    #[builder(field = math::UVec3::ONE)]
    pub local_shape: math::UVec3,

    #[builder(field = math::UVec3::ONE)]
    pub grid_shape: math::UVec3,

    #[builder(default = ComputeDimensions::ONE)]
    pub dimensions: ComputeDimensions,

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
            local_shape: math::UVec3::ONE,
            grid_shape: math::UVec3::ONE,
            dimensions: ComputeDimensions::ONE,
            ports: Vec::new(),
            constants: HashMap::new(),
        }
    }
}

impl<State: compute_node_descriptor_builder::State> ComputeNodeDescriptorBuilder<State> {
    /// Sets the local workgroup shape `[x, y, z]`.
    pub fn local_shape(mut self, shape: &math::UVec3) -> Self {
        self.local_shape = *shape;
        self
    }

    /// Sets the dispatch grid shape `[x, y, z]`.
    pub fn grid_shape(mut self, shape: &math::UVec3) -> Self {
        self.grid_shape = *shape;
        self
    }

    /// Computes the grid shape from a global shape: `grid = ceil(global / local)`.
    pub fn configure_grid_shape(mut self, global_shape: &math::UVec3) -> Self {
        self.grid_shape.inner.x = global_shape.inner.x.div_ceil(self.local_shape.inner.x);
        self.grid_shape.inner.y = global_shape.inner.y.div_ceil(self.local_shape.inner.y);
        self.grid_shape.inner.z = global_shape.inner.z.div_ceil(self.local_shape.inner.z);
        self
    }

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
        if self.local_shape.inner.x == 0 || self.local_shape.inner.y == 0 || self.local_shape.inner.z == 0 {
            return Err(ComputeNodeError::InvalidLocalShape);
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
        let desc = ComputeNodeDescriptor::builder().build();
        assert!(desc.program.is_none());
        assert_eq!(desc.function_name, "main");
        assert_eq!(desc.local_shape.inner.x, 1);
        assert_eq!(desc.grid_shape.inner.x, 1);
        assert!(desc.ports.is_empty());
        assert!(desc.constants.is_empty());
    }

    #[test]
    fn builder_adds_port_and_constant() {
        let port = PortDescriptor::default();
        let val = Constant::Int(42);

        let desc = ComputeNodeDescriptor::builder()
            .dimensions(ComputeDimensions::ONE)
            .add_port(port)
            .add_constant("my_const", val)
            .build();

        assert_eq!(desc.ports.len(), 1);
        assert_eq!(desc.constants.len(), 1);
        assert!(desc.get_constant("my_const").is_ok());
    }

    #[test]
    fn configure_grid_shape_ceiling_division() {
        let local = math::UVec3::new(8, 4, 2);
        let global = math::UVec3::new(17, 9, 5);

        let desc = ComputeNodeDescriptor::builder()
            .dimensions(ComputeDimensions::ONE)
            .local_shape(&local)
            .configure_grid_shape(&global)
            .build();

        // ceil(17/8) = 3, ceil(9/4) = 3, ceil(5/2) = 3
        assert_eq!(desc.grid_shape.inner.x, 3);
        assert_eq!(desc.grid_shape.inner.y, 3);
        assert_eq!(desc.grid_shape.inner.z, 3);
    }

    #[test]
    fn configure_grid_shape_exact_division() {
        let local = math::UVec3::new(4, 4, 4);
        let global = math::UVec3::new(8, 8, 8);

        let desc = ComputeNodeDescriptor::builder()
            .dimensions(ComputeDimensions::ONE)
            .local_shape(&local)
            .configure_grid_shape(&global)
            .build();

        assert_eq!(desc.grid_shape.inner.x, 2);
        assert_eq!(desc.grid_shape.inner.y, 2);
        assert_eq!(desc.grid_shape.inner.z, 2);
    }
}
