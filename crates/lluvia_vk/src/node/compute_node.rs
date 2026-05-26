//! Compute node wrapping a Vulkan compute pipeline.

use std::collections::HashMap;
use std::sync::Arc;

use foldhash::HashMapExt;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::PrimaryAutoCommandBuffer;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{DescriptorSet, WriteDescriptorSet};
use vulkano::device::Device;
use vulkano::pipeline::Pipeline;
use vulkano::pipeline::compute::ComputePipelineCreateInfo;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::{PipelineLayout, PipelineShaderStageCreateInfo, compute::ComputePipeline};
use vulkano::shader::SpecializedShaderModule;

use crate::math;

use super::compute_node_descriptor::ComputeNodeDescriptor;
use super::node_port::NodePort;
use super::node_type::NodeType;
use super::{ComputeNodeError, Node};

// ---------------------------------------------------------------------------
// ComputeNode
// ---------------------------------------------------------------------------

/// A compute node wrapping a `vulkano::pipeline::ComputePipeline`.
///
/// Mirrors C++ `ll::ComputeNode`.
pub struct ComputeNode {
    pipeline: Arc<ComputePipeline>,
    descriptor: ComputeNodeDescriptor,
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

        let program = descriptor.program.as_ref().ok_or(ComputeNodeError::InvalidProgram)?;

        ///////////////////////////////////////////////////////////////////////
        // Specialization constants to set local grid shape
        let mut specialization_constants = foldhash::HashMap::<u32, vulkano::shader::SpecializationConstant>::new();

        // TODO: should use some linear algebra to represent this.
        let local_shape = &descriptor.local_shape;

        specialization_constants.insert(1, (local_shape.inner.x).into());
        specialization_constants.insert(2, (local_shape.inner.y).into());
        specialization_constants.insert(3, (local_shape.inner.z).into());

        let shader_module: Arc<SpecializedShaderModule> = program
            .shader_module()
            .specialize(specialization_constants)
            .map_err(|e| ComputeNodeError::CreationFailed(e.to_string()))?;

        let entry_point = shader_module.entry_point(&descriptor.function_name).ok_or_else(|| {
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
            objects: HashMap::new(),
            descriptor_set_allocator,
            descriptor_set: None,
        })
    }

    /// Returns the node descriptor.
    pub fn descriptor(&self) -> &ComputeNodeDescriptor {
        &self.descriptor
    }

    /// Returns the grid shape `[x, y, z]`.
    pub fn grid_shape(&self) -> math::UVec3 {
        self.descriptor.grid_shape
    }

    /// Sets the grid shape.
    pub fn set_grid_shape(&mut self, shape: &math::UVec3) {
        self.descriptor.grid_shape = *shape;
    }

    /// Returns the local workgroup shape.
    pub fn local_shape(&self) -> math::UVec3 {
        self.descriptor.local_shape
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
                    NodePort::ImageView(img) => WriteDescriptorSet::image_view(port.binding, img.view().clone()),
                };
                writes.push(write);
            }
        }

        if writes.is_empty() {
            self.descriptor_set = None;
            return Ok(());
        }

        let set = DescriptorSet::new(self.descriptor_set_allocator.clone(), layout.clone(), writes, [])
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

    fn record(&self, builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) -> Result<(), ComputeNodeError> {
        if self.descriptor.grid_shape.inner.x == 0
            || self.descriptor.grid_shape.inner.y == 0
            || self.descriptor.grid_shape.inner.z == 0
        {
            return Err(ComputeNodeError::DispatchFailed("Grid shape contains zero".to_string()));
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

        unsafe {
            builder
                .dispatch(self.descriptor.grid_shape.inner.to_array())
                .map_err(|e| ComputeNodeError::DispatchFailed(e.to_string()))?;
        }

        Ok(())
    }
}
