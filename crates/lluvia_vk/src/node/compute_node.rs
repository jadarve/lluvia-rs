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
use crate::node::ComputeDimensions;

use super::compute_node_descriptor::ComputeNodeDescriptor;
use super::node_port::{NodePort, PushConstants};
use super::node_type::NodeType;
use super::{ComputeNodeError, Constant, Node};

fn get_groups_shape(workgroup_shape: &math::UVec3, global_shape: &math::UVec3) -> math::UVec3 {
    math::UVec3::new(
        global_shape.inner.x.div_ceil(workgroup_shape.inner.x),
        global_shape.inner.y.div_ceil(workgroup_shape.inner.y),
        global_shape.inner.z.div_ceil(workgroup_shape.inner.z),
    )
}

/// Computes an optimal local workgroup shape (local size) for a given dimension layout.
///
/// The computed shape ensures:
/// 1. Maximum utilization of hardware subgroups by building up power-of-two dimensions.
/// 2. Total invocations (X * Y * Z) do not exceed the device's `max_compute_work_group_invocations`.
/// 3. Individual dimensions do not exceed the device's `max_compute_work_group_size` limits.
fn get_workgroup_shape(device: &Arc<Device>, dimensions: ComputeDimensions) -> math::UVec3 {
    // Get properties of the physical device
    let properties = device.physical_device().properties();

    // Get subgroup size (typically 32 or 64; default to 32 if None)
    let _subgroup_size = properties.subgroup_size.unwrap_or(32);

    // Get the maximum allowed workgroup size dimensions and invocations
    let max_invocations = properties.max_compute_work_group_invocations;
    let max_workgroup_size = properties.max_compute_work_group_size; // [u32; 3]

    match dimensions {
        ComputeDimensions::ONE => {
            let x = max_invocations.min(max_workgroup_size[0]);
            math::UVec3::new(x, 1, 1)
        }
        ComputeDimensions::TWO => {
            let mut x = 1;
            let mut y = 1;

            // The `* 2` acts as a look-ahead guard checking if doubling one of the dimensions
            // will exceed the max_invocations limit in the next step.
            while x * y * 2 <= max_invocations && (x < max_workgroup_size[0] || y < max_workgroup_size[1]) {
                // Grow the smallest dimension first to maintain a balanced, square-ish shape.
                // This maximizes spatial cache locality for 2D data (e.g. image processing).
                if x <= y && x < max_workgroup_size[0] {
                    x *= 2;
                } else if y < max_workgroup_size[1] {
                    y *= 2;
                } else if x < max_workgroup_size[0] {
                    // Fallback: if the preferred dimension hits its limit, grow the other dimension.
                    x *= 2;
                } else {
                    break;
                }
            }
            math::UVec3::new(x, y, 1)
        }
        ComputeDimensions::THREE => {
            let mut x = 1;
            let mut y = 1;
            let mut z = 1;

            // The `* 2` acts as a look-ahead guard checking if doubling one of the dimensions
            // will exceed the max_invocations limit in the next step.
            while x * y * z * 2 <= max_invocations
                && (x < max_workgroup_size[0] || y < max_workgroup_size[1] || z < max_workgroup_size[2])
            {
                // Grow the smallest dimension first to maintain a balanced, cubic-ish shape.
                // This maximizes spatial cache locality for 3D data (e.g. volume grids).
                if x <= y && x <= z && x < max_workgroup_size[0] {
                    x *= 2;
                } else if y <= z && y < max_workgroup_size[1] {
                    y *= 2;
                } else if z < max_workgroup_size[2] {
                    z *= 2;
                } else if x < max_workgroup_size[0] {
                    // Fallback: if the preferred dimension hits its limit, grow the other dimensions.
                    x *= 2;
                } else if y < max_workgroup_size[1] {
                    y *= 2;
                } else {
                    break;
                }
            }
            math::UVec3::new(x, y, z)
        }
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
    objects: HashMap<String, NodePort>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    descriptor_set: Option<Arc<DescriptorSet>>,
    workgroup_shape: math::UVec3,
    global_shape: Option<math::UVec3>,
    pub push_constants: Option<PushConstants>,
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

        let workgroup_shape = get_workgroup_shape(&device, descriptor.dimensions);

        specialization_constants.insert(1, (workgroup_shape.inner.x).into());
        specialization_constants.insert(2, (workgroup_shape.inner.y).into());
        specialization_constants.insert(3, (workgroup_shape.inner.z).into());

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
            workgroup_shape,
            global_shape: None,
            push_constants: None,
        })
    }

    /// Returns the node descriptor.
    pub fn descriptor(&self) -> &ComputeNodeDescriptor {
        &self.descriptor
    }

    /// Sets a constant on the descriptor.
    /// FIXME: this should not be here
    pub fn set_constant(&mut self, name: impl Into<String>, value: Constant) {
        self.descriptor.set_constant(name, value);
    }

    /// Gets a constant reference by name.
    /// FIXME: this should not be here
    pub fn get_constant(&self, name: &str) -> Result<&Constant, ComputeNodeError> {
        self.descriptor.get_constant(name)
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
        let global_shape = if let Some(global_shape) = self.global_shape {
            global_shape
        } else {
            // return Err(ComputeNodeError::DispatchFailed("Global shape not set".to_string()));
            math::UVec3::new(128, 0, 0)
        };

        let groups = get_groups_shape(&self.workgroup_shape, &global_shape);
        println!("groups: {:?}", groups.inner);

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

        // FIXME: need to figure out a better way.
        if let Some(push_constants) = &self.push_constants {
            let size = push_constants.size();
            if size > 0 {
                let layout = self.pipeline.layout().clone();
                match size {
                    4 => {
                        let mut data = [0u8; 4];
                        data.copy_from_slice(&push_constants.data()[..4]);
                        builder
                            .push_constants(layout.clone(), 0, data)
                            .map_err(|e| ComputeNodeError::DispatchFailed(e.to_string()))?;
                    }
                    8 => {
                        let mut data = [0u8; 8];
                        data.copy_from_slice(&push_constants.data()[..8]);
                        builder
                            .push_constants(layout, 0, data)
                            .map_err(|e| ComputeNodeError::DispatchFailed(e.to_string()))?;
                    }
                    12 => {
                        let mut data = [0u8; 12];
                        data.copy_from_slice(&push_constants.data()[..12]);
                        builder
                            .push_constants(layout, 0, data)
                            .map_err(|e| ComputeNodeError::DispatchFailed(e.to_string()))?;
                    }
                    16 => {
                        let mut data = [0u8; 16];
                        data.copy_from_slice(&push_constants.data()[..16]);
                        builder
                            .push_constants(layout, 0, data)
                            .map_err(|e| ComputeNodeError::DispatchFailed(e.to_string()))?;
                    }
                    20 => {
                        let mut data = [0u8; 20];
                        data.copy_from_slice(&push_constants.data()[..20]);
                        builder
                            .push_constants(layout, 0, data)
                            .map_err(|e| ComputeNodeError::DispatchFailed(e.to_string()))?;
                    }
                    24 => {
                        let mut data = [0u8; 24];
                        data.copy_from_slice(&push_constants.data()[..24]);
                        builder
                            .push_constants(layout, 0, data)
                            .map_err(|e| ComputeNodeError::DispatchFailed(e.to_string()))?;
                    }
                    28 => {
                        let mut data = [0u8; 28];
                        data.copy_from_slice(&push_constants.data()[..28]);
                        builder
                            .push_constants(layout, 0, data)
                            .map_err(|e| ComputeNodeError::DispatchFailed(e.to_string()))?;
                    }
                    32 => {
                        let mut data = [0u8; 32];
                        data.copy_from_slice(&push_constants.data()[..32]);
                        builder
                            .push_constants(layout, 0, data)
                            .map_err(|e| ComputeNodeError::DispatchFailed(e.to_string()))?;
                    }
                    36 => {
                        let mut data = [0u8; 36];
                        data.copy_from_slice(&push_constants.data()[..36]);
                        builder
                            .push_constants(layout, 0, data)
                            .map_err(|e| ComputeNodeError::DispatchFailed(e.to_string()))?;
                    }
                    40 => {
                        let mut data = [0u8; 40];
                        data.copy_from_slice(&push_constants.data()[..40]);
                        builder
                            .push_constants(layout, 0, data)
                            .map_err(|e| ComputeNodeError::DispatchFailed(e.to_string()))?;
                    }
                    44 => {
                        let mut data = [0u8; 44];
                        data.copy_from_slice(&push_constants.data()[..44]);
                        builder
                            .push_constants(layout, 0, data)
                            .map_err(|e| ComputeNodeError::DispatchFailed(e.to_string()))?;
                    }
                    48 => {
                        let mut data = [0u8; 48];
                        data.copy_from_slice(&push_constants.data()[..48]);
                        builder
                            .push_constants(layout, 0, data)
                            .map_err(|e| ComputeNodeError::DispatchFailed(e.to_string()))?;
                    }
                    52 => {
                        let mut data = [0u8; 52];
                        data.copy_from_slice(&push_constants.data()[..52]);
                        builder
                            .push_constants(layout, 0, data)
                            .map_err(|e| ComputeNodeError::DispatchFailed(e.to_string()))?;
                    }
                    56 => {
                        let mut data = [0u8; 56];
                        data.copy_from_slice(&push_constants.data()[..56]);
                        builder
                            .push_constants(layout, 0, data)
                            .map_err(|e| ComputeNodeError::DispatchFailed(e.to_string()))?;
                    }
                    60 => {
                        let mut data = [0u8; 60];
                        data.copy_from_slice(&push_constants.data()[..60]);
                        builder
                            .push_constants(layout, 0, data)
                            .map_err(|e| ComputeNodeError::DispatchFailed(e.to_string()))?;
                    }
                    64 => {
                        let mut data = [0u8; 64];
                        data.copy_from_slice(&push_constants.data()[..64]);
                        builder
                            .push_constants(layout, 0, data)
                            .map_err(|e| ComputeNodeError::DispatchFailed(e.to_string()))?;
                    }
                    _ => {
                        return Err(ComputeNodeError::DispatchFailed(format!(
                            "Unsupported push constants size: {}",
                            size
                        )));
                    }
                }
            }
        }

        unsafe {
            builder
                .dispatch(groups.inner.to_array())
                .map_err(|e| ComputeNodeError::DispatchFailed(e.to_string()))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{Session, SessionDescriptor};

    #[test]
    fn test_get_workgroup_shape() {
        let session = Session::new(SessionDescriptor::default()).unwrap();
        let device = session.device();

        let max_invocations = device.physical_device().properties().max_compute_work_group_invocations;
        let max_workgroup_size = device.physical_device().properties().max_compute_work_group_size;

        let shape_1d = get_workgroup_shape(&device, ComputeDimensions::ONE);
        assert!(shape_1d.inner.x >= 1);
        assert!(shape_1d.inner.x <= max_workgroup_size[0]);
        assert_eq!(shape_1d.inner.y, 1);
        assert_eq!(shape_1d.inner.z, 1);
        assert!(shape_1d.inner.x * shape_1d.inner.y * shape_1d.inner.z <= max_invocations);

        let shape_2d = get_workgroup_shape(&device, ComputeDimensions::TWO);
        assert!(shape_2d.inner.x >= 1);
        assert!(shape_2d.inner.x <= max_workgroup_size[0]);
        assert!(shape_2d.inner.y >= 1);
        assert!(shape_2d.inner.y <= max_workgroup_size[1]);
        assert_eq!(shape_2d.inner.z, 1);
        assert!(shape_2d.inner.x * shape_2d.inner.y <= max_invocations);

        let shape_3d = get_workgroup_shape(&device, ComputeDimensions::THREE);
        assert!(shape_3d.inner.x >= 1);
        assert!(shape_3d.inner.x <= max_workgroup_size[0]);
        assert!(shape_3d.inner.y >= 1);
        assert!(shape_3d.inner.y <= max_workgroup_size[1]);
        assert!(shape_3d.inner.z >= 1);
        assert!(shape_3d.inner.z <= max_workgroup_size[2]);
        assert!(shape_3d.inner.x * shape_3d.inner.y * shape_3d.inner.z <= max_invocations);
    }
}
