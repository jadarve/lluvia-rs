//! Compute node abstractions.
//!
//! This module mirrors the C++ `ll::ComputeNodeDescriptor` and
//! `ll::ComputeNode` classes.

use std::collections::HashMap;
use std::sync::Arc;

use thiserror::Error;
use vulkano::device::Device;
use vulkano::pipeline::compute::ComputePipelineCreateInfo;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::{PipelineLayout, PipelineShaderStageCreateInfo, compute::ComputePipeline};
use vulkano::shader::ShaderModule;

use crate::program::Program;

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

    #[error("Dispatch failed: {0}")]
    DispatchFailed(String),

    #[error("Port not found: {0}")]
    PortNotFound(String),
}

// ---------------------------------------------------------------------------
// PortDescriptor (simplified mirror of C++ ll::PortDescriptor)
// ---------------------------------------------------------------------------

/// Direction of a port (input or output).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortDirection {
    In,
    Out,
}

/// Type of object a port accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortType {
    Buffer,
    ImageView,
}

/// Descriptor for a node port, mirroring C++ `ll::PortDescriptor`.
#[derive(Debug, Clone)]
pub struct PortDescriptor {
    pub binding: u32,
    pub name: String,
    pub direction: PortDirection,
    pub port_type: PortType,
}

// ---------------------------------------------------------------------------
// ComputeNodeDescriptor
// ---------------------------------------------------------------------------

/// Descriptor used to build a [`ComputeNode`].
///
/// Mirrors C++ `ll::ComputeNodeDescriptor`.
#[derive(Clone)]
pub struct ComputeNodeDescriptor {
    program: Option<Arc<Program>>,
    function_name: String,
    local_shape: [u32; 3],
    grid_shape: [u32; 3],
    ports: Vec<PortDescriptor>,
    parameters: HashMap<String, f64>,
}

impl Default for ComputeNodeDescriptor {
    fn default() -> Self {
        Self {
            program: None,
            function_name: "main".to_string(),
            local_shape: [1, 1, 1],
            grid_shape: [1, 1, 1],
            ports: Vec::new(),
            parameters: HashMap::new(),
        }
    }
}

impl ComputeNodeDescriptor {
    /// Sets the shader program.
    pub fn program(mut self, program: Arc<Program>) -> Self {
        self.program = Some(program);
        self
    }

    /// Sets the entry-point function name (default: `"main"`).
    pub fn function_name(mut self, name: impl Into<String>) -> Self {
        self.function_name = name.into();
        self
    }

    /// Sets the local workgroup shape `[x, y, z]`.
    pub fn local_shape(mut self, shape: [u32; 3]) -> Self {
        self.local_shape = shape;
        self
    }

    /// Sets the dispatch grid shape `[x, y, z]`.
    pub fn grid_shape(mut self, shape: [u32; 3]) -> Self {
        self.grid_shape = shape;
        self
    }

    /// Computes the grid shape from a global shape: `grid = ceil(global / local)`.
    pub fn configure_grid_shape(mut self, global_shape: [u32; 3]) -> Self {
        for (grid, (global, local)) in self
            .grid_shape
            .iter_mut()
            .zip(global_shape.iter().zip(self.local_shape.iter()))
        {
            *grid = global.div_ceil(*local);
        }
        self
    }

    /// Adds a port descriptor.
    pub fn add_port(mut self, port: PortDescriptor) -> Self {
        self.ports.push(port);
        self
    }

    /// Sets a named parameter.
    pub fn set_parameter(mut self, name: impl Into<String>, value: f64) -> Self {
        self.parameters.insert(name.into(), value);
        self
    }

    /// Returns the program, if set.
    pub fn get_program(&self) -> Option<&Arc<Program>> {
        self.program.as_ref()
    }

    /// Returns the function name.
    pub fn get_function_name(&self) -> &str {
        &self.function_name
    }

    /// Returns the local workgroup shape.
    pub fn get_local_shape(&self) -> [u32; 3] {
        self.local_shape
    }

    /// Returns the dispatch grid shape.
    pub fn get_grid_shape(&self) -> [u32; 3] {
        self.grid_shape
    }

    fn validate(&self) -> Result<(), ComputeNodeError> {
        if self.program.is_none() {
            return Err(ComputeNodeError::InvalidProgram);
        }
        if self.function_name.is_empty() {
            return Err(ComputeNodeError::InvalidFunctionName);
        }
        if self.local_shape.contains(&0) {
            return Err(ComputeNodeError::InvalidLocalShape);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// ComputeNode
// ---------------------------------------------------------------------------

/// A compute node wrapping a `vulkano::pipeline::ComputePipeline`.
///
/// Mirrors C++ `ll::ComputeNode`.
pub struct ComputeNode {
    pipeline: Arc<ComputePipeline>,
    descriptor: ComputeNodeDescriptor,
    device: Arc<Device>,
}

impl ComputeNode {
    /// Creates a new compute node from a descriptor.
    ///
    /// The descriptor must have a valid program and non-empty function name.
    pub fn new(
        device: Arc<Device>,
        descriptor: ComputeNodeDescriptor,
    ) -> Result<Self, ComputeNodeError> {
        descriptor.validate()?;

        let program = descriptor
            .program
            .as_ref()
            .ok_or(ComputeNodeError::InvalidProgram)?;
        let shader_module: &Arc<ShaderModule> = program.shader_module();

        let entry_point = shader_module
            .entry_point(&descriptor.function_name)
            .ok_or_else(|| {
                ComputeNodeError::CreationFailed(format!(
                    "Entry point '{}' not found in shader module",
                    descriptor.function_name
                ))
            })?;

        let stage = PipelineShaderStageCreateInfo::new(entry_point);
        let layout = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages([&stage])
                .into_pipeline_layout_create_info(device.clone())
                .map_err(|e| ComputeNodeError::CreationFailed(e.to_string()))?,
        )
        .map_err(|e| ComputeNodeError::CreationFailed(e.to_string()))?;

        let pipeline = ComputePipeline::new(
            device.clone(),
            None,
            ComputePipelineCreateInfo::stage_layout(stage, layout),
        )
        .map_err(|e| ComputeNodeError::CreationFailed(e.to_string()))?;

        Ok(Self {
            pipeline,
            descriptor,
            device,
        })
    }

    /// Returns the underlying compute pipeline.
    pub fn pipeline(&self) -> &Arc<ComputePipeline> {
        &self.pipeline
    }

    /// Returns the node descriptor.
    pub fn descriptor(&self) -> &ComputeNodeDescriptor {
        &self.descriptor
    }

    /// Returns the grid shape `[x, y, z]`.
    pub fn grid_shape(&self) -> [u32; 3] {
        self.descriptor.grid_shape
    }

    /// Sets the grid shape.
    pub fn set_grid_shape(&mut self, shape: [u32; 3]) {
        self.descriptor.grid_shape = shape;
    }

    /// Returns the local workgroup shape.
    pub fn local_shape(&self) -> [u32; 3] {
        self.descriptor.local_shape
    }

    /// Returns the device.
    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }
}
