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
use vulkano::pipeline::Pipeline;
use vulkano::shader::ShaderModule;

use crate::buffer::Buffer;
use crate::image::ImageView;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::PrimaryAutoCommandBuffer;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{DescriptorSet, WriteDescriptorSet};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Compute,
    Container,
}

#[derive(Clone)]
pub enum NodePort {
    Buffer(Arc<Buffer>),
    ImageView(Arc<ImageView>),
}

#[derive(Clone, Default)]
pub struct PushConstants {
    data: Vec<u8>,
}

impl PushConstants {
    pub fn push_f32(&mut self, value: f32) {
        self.data.extend_from_slice(&value.to_ne_bytes());
    }

    pub fn push_i32(&mut self, value: i32) {
        self.data.extend_from_slice(&value.to_ne_bytes());
    }

    pub fn size(&self) -> u32 {
        self.data.len() as u32
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

pub trait Node {
    fn node_type(&self) -> NodeType;
    fn bind(&mut self, name: &str, obj: NodePort) -> Result<(), ComputeNodeError>;
    fn has_port(&self, name: &str) -> bool;
    fn port(&self, name: &str) -> Option<&NodePort>;
    fn set_parameter(&mut self, name: &str, value: f64);
    fn parameter(&self, name: &str) -> Option<f64>;
    fn record(
        &self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) -> Result<(), ComputeNodeError>;
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
    push_constants: PushConstants,
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
            push_constants: PushConstants::default(),
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

    /// Sets the push constants.
    pub fn push_constants(mut self, pc: PushConstants) -> Self {
        self.push_constants = pc;
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

    /// Returns the push constants.
    pub fn get_push_constants(&self) -> &PushConstants {
        &self.push_constants
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
    objects: HashMap<String, NodePort>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    descriptor_set: Option<Arc<DescriptorSet>>,
}

impl ComputeNode {
    /// Creates a new compute node from a descriptor.
    ///
    /// The descriptor must have a valid program and non-empty function name.
    pub fn new(
        device: Arc<Device>,
        descriptor: ComputeNodeDescriptor,
        descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
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
            objects: HashMap::new(),
            descriptor_set_allocator,
            descriptor_set: None,
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

    fn update_descriptor_set(&mut self) -> Result<(), ComputeNodeError> {
        if self.pipeline.layout().set_layouts().is_empty() {
            return Ok(());
        }
        let layout = &self.pipeline.layout().set_layouts()[0];
        let mut writes = Vec::new();

        for port in &self.descriptor.ports {
            if let Some(obj) = self.objects.get(&port.name) {
                let write = match obj {
                    NodePort::Buffer(b) => WriteDescriptorSet::buffer(port.binding, b.inner().clone()),
                    NodePort::ImageView(img) => {
                        WriteDescriptorSet::image_view(port.binding, img.view().clone())
                    }
                };
                writes.push(write);
            }
        }

        if writes.is_empty() {
            self.descriptor_set = None;
            return Ok(());
        }

        let set = DescriptorSet::new(
            self.descriptor_set_allocator.clone(),
            layout.clone(),
            writes,
            [],
        )
        .map_err(|e| ComputeNodeError::CreationFailed(e.to_string()))?;

        self.descriptor_set = Some(set);
        Ok(())
    }
}

impl Node for ComputeNode {
    fn node_type(&self) -> NodeType {
        NodeType::Compute
    }

    fn bind(&mut self, name: &str, obj: NodePort) -> Result<(), ComputeNodeError> {
        let _port_desc = self
            .descriptor
            .ports
            .iter()
            .find(|p| p.name == name)
            .ok_or_else(|| ComputeNodeError::PortNotFound(name.to_string()))?;

        self.objects.insert(name.to_string(), obj);
        self.update_descriptor_set()
    }

    fn has_port(&self, name: &str) -> bool {
        self.objects.contains_key(name)
    }

    fn port(&self, name: &str) -> Option<&NodePort> {
        self.objects.get(name)
    }

    fn set_parameter(&mut self, name: &str, value: f64) {
        self.descriptor.parameters.insert(name.to_string(), value);
    }

    fn parameter(&self, name: &str) -> Option<f64> {
        self.descriptor.parameters.get(name).copied()
    }

    fn record(
        &self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) -> Result<(), ComputeNodeError> {
        if self.descriptor.grid_shape.contains(&0) {
            return Err(ComputeNodeError::DispatchFailed(
                "Grid shape contains zero".to_string(),
            ));
        }

        builder
            .bind_pipeline_compute(self.pipeline.clone())
            .map_err(|e| ComputeNodeError::DispatchFailed(e.to_string()))?;

        if let Some(set) = &self.descriptor_set {
            builder
                .bind_descriptor_sets(
                    vulkano::pipeline::PipelineBindPoint::Compute,
                    self.pipeline.layout().clone(),
                    0,
                    vec![set.clone()],
                )
                .map_err(|e| ComputeNodeError::DispatchFailed(e.to_string()))?;
        }

        let pc_data = self.descriptor.push_constants.data();
        if !pc_data.is_empty() {
            // Push constants are not properly supported for dynamic slices in vulkano AutoCommandBufferBuilder
            // For now, we will fail if push constants are provided.
            return Err(ComputeNodeError::DispatchFailed(
                "Dynamic push constants are not supported via Vec<u8>".to_string(),
            ));
        }

        unsafe {
            builder
                .dispatch(self.descriptor.grid_shape)
                .map_err(|e| ComputeNodeError::DispatchFailed(e.to_string()))?;
        }

        Ok(())
    }
}
