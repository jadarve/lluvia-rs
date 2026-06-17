use std::sync::Arc;

use thiserror::Error;
use vulkano::VulkanLibrary;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::device::physical::PhysicalDeviceType;
use vulkano::device::{Device, DeviceCreateInfo, Queue, QueueCreateInfo, QueueFlags};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::memory::allocator::StandardMemoryAllocator;

use crate::buffer::{Buffer, BufferError};
use crate::command_buffer::{CommandBuffer, CommandBufferBuilder, CommandBufferError};
use crate::node::{ComputeNode, ComputeNodeDescriptor, ComputeNodeError, ContainerNode, ContainerNodeDescriptor};
use crate::node_repository::Repository;
use crate::program::{Program, ProgramError};

#[derive(Error, Debug)]
pub enum SessionError {
    #[error("Failed to create Vulkan instance")]
    InstanceCreationError(#[from] vulkano::LoadingError),

    #[error("Runtime error: {0}")]
    RuntimeError(String),

    #[error("Compute Node error: {0}")]
    NodeError(#[from] ComputeNodeError),

    #[error("Command buffer error: {0}")]
    CommandBufferError(#[from] CommandBufferError),

    #[error("Vulkan future error: {0}")]
    FutureError(String),

    #[error("Builder not found: {0}")]
    BuilderNotFound(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Other,
    IntegratedGpu,
    DiscreteGpu,
    VirtualGpu,
    Cpu,
}

impl From<PhysicalDeviceType> for DeviceType {
    fn from(t: PhysicalDeviceType) -> Self {
        match t {
            PhysicalDeviceType::Other => DeviceType::Other,
            PhysicalDeviceType::IntegratedGpu => DeviceType::IntegratedGpu,
            PhysicalDeviceType::DiscreteGpu => DeviceType::DiscreteGpu,
            PhysicalDeviceType::VirtualGpu => DeviceType::VirtualGpu,
            PhysicalDeviceType::Cpu => DeviceType::Cpu,
            _ => DeviceType::Other,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceDescriptor {
    pub id: u32,
    pub device_type: DeviceType,
    pub name: String,
}

#[derive(Debug, Clone, Default, bon::Builder)]
pub struct SessionDescriptor {
    #[builder(default)]
    pub enable_debug: bool,
    pub device_descriptor: Option<DeviceDescriptor>,
}

pub struct Session {
    // instance: Arc<Instance>,
    device: Arc<Device>,
    compute_queue: Arc<Queue>,
    allocator: Arc<StandardMemoryAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,

    // TOTHINK: consider adding internal mutability to interpreter to contain Mutex<Lua>
    interpreter: Arc<std::sync::Mutex<crate::interpreter::Interpreter>>,

    pub(crate) repositories: Vec<Arc<Box<dyn Repository>>>,
}

impl Session {
    pub fn new(descriptor: SessionDescriptor) -> Result<Arc<Self>, SessionError> {
        let library = VulkanLibrary::new().map_err(SessionError::InstanceCreationError)?;

        // TODO: enable layers and debug extensions based on descriptor.enable_debug
        let instance_create_info = InstanceCreateInfo {
            application_name: Some("lluvia".to_string()),
            engine_name: Some("lluvia".to_string()),
            ..Default::default()
        };

        let instance = Instance::new(library, instance_create_info)
            .map_err(|e| SessionError::RuntimeError(format!("Instance creation failed: {e}")))?;

        let physical_devices: Vec<_> = instance
            .enumerate_physical_devices()
            .map_err(|e| SessionError::RuntimeError(e.to_string()))?
            .collect();

        let physical_device = if let Some(dev_desc) = descriptor.device_descriptor {
            physical_devices
                .into_iter()
                .find(|dev| {
                    let props = dev.properties();
                    props.device_id == dev_desc.id
                        && DeviceType::from(props.device_type) == dev_desc.device_type
                        && props.device_name == dev_desc.name
                })
                .ok_or_else(|| SessionError::RuntimeError("Requested device not found".to_string()))?
        } else {
            physical_devices
                .clone()
                .into_iter()
                .find(|dev| dev.properties().device_type == PhysicalDeviceType::DiscreteGpu)
                .or_else(|| physical_devices.into_iter().next())
                .ok_or_else(|| SessionError::RuntimeError("No physical devices found".to_string()))?
        };

        let queue_family_index = physical_device
            .queue_family_properties()
            .iter()
            .enumerate()
            .find(|(_index, properties)| properties.queue_flags.intersects(QueueFlags::COMPUTE))
            .map(|(index, _)| index as u32)
            .ok_or_else(|| SessionError::RuntimeError("No compute queue family found".to_string()))?;

        // Needed for Slang shaders.
        let mut enabled_features = vulkano::device::DeviceFeatures::default();
        if physical_device.supported_features().maintenance4 {
            enabled_features.maintenance4 = true;
        }

        let device_create_info = DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            enabled_features,
            ..Default::default()
        };

        let (device, mut queues) = Device::new(physical_device, device_create_info)
            .map_err(|e| SessionError::RuntimeError(format!("device creation: {e}")))?;

        let compute_queue = queues
            .next()
            .ok_or_else(|| SessionError::RuntimeError("Failed to get compute queue".to_string()))?;

        let allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let descriptor_set_allocator =
            Arc::new(StandardDescriptorSetAllocator::new(device.clone(), Default::default()));
        let command_buffer_allocator =
            Arc::new(StandardCommandBufferAllocator::new(device.clone(), Default::default()));
        let interpreter = Arc::new(std::sync::Mutex::new(
            crate::interpreter::Interpreter::new()
                .map_err(|e| SessionError::RuntimeError(format!("Failed to create Interpreter: {e:?}")))?,
        ));

        ///////////////////////////////////////////////////////////////////////
        // Default crate repository

        let repositories: Vec<Arc<Box<dyn Repository>>> =
            vec![Arc::new(Box::new(crate::node_repository::InternalRepository {}))];

        let session = Arc::new(Self {
            // instance,
            device,
            compute_queue,
            allocator,
            descriptor_set_allocator,
            command_buffer_allocator,
            interpreter,
            repositories,
        });

        // Register a weak back-reference in the interpreter so Luau globals
        // (e.g. `load_program`) can reach the session without creating a
        // strong reference cycle.
        session
            .interpreter
            .lock()
            .map_err(|_| SessionError::RuntimeError("Interpreter lock poisoned".to_string()))?
            .set_session(Arc::downgrade(&session))
            .map_err(|e| SessionError::RuntimeError(format!("Failed to register session globals: {e}")))?;

        Ok(session)
    }

    pub fn compute_queue(&self) -> Arc<Queue> {
        self.compute_queue.clone()
    }

    pub fn create_image(
        &self,
        descriptor: crate::image::ImageDescriptor,
    ) -> Result<Arc<crate::image::Image>, crate::image::ImageError> {
        crate::image::Image::new(self.allocator.clone(), descriptor).map(Arc::new)
    }

    pub fn create_buffer_device_local(&self, size: u64) -> Result<Arc<Buffer>, BufferError> {
        Buffer::new_device_local(self.allocator.clone(), size).map(Arc::new)
    }

    pub fn create_buffer_host_visible(&self, size: u64) -> Result<Arc<Buffer>, BufferError> {
        Buffer::new_host_visible(self.allocator.clone(), size).map(Arc::new)
    }

    pub fn create_compute_node(&self, descriptor: ComputeNodeDescriptor) -> Result<ComputeNode, SessionError> {
        ComputeNode::new(self.device.clone(), descriptor, self.descriptor_set_allocator.clone())
            .map_err(SessionError::NodeError)
    }

    pub fn create_container_node(&self, descriptor: ContainerNodeDescriptor) -> Result<ContainerNode, SessionError> {
        Ok(ContainerNode::new(Arc::downgrade(&self.interpreter), descriptor))
    }

    pub fn create_command_buffer_builder(&self) -> Result<CommandBufferBuilder, SessionError> {
        CommandBufferBuilder::new(
            self.command_buffer_allocator.clone(),
            self.compute_queue.queue_family_index(),
        )
        .map_err(SessionError::CommandBufferError)
    }

    pub fn run(&self, command_buffer: CommandBuffer) -> Result<(), SessionError> {
        use vulkano::sync::GpuFuture;
        let future = vulkano::sync::now(self.device.clone())
            .then_execute(self.compute_queue.clone(), command_buffer.inner)
            .map_err(|e| SessionError::FutureError(e.to_string()))?
            .then_signal_fence_and_flush()
            .map_err(|e| SessionError::FutureError(e.to_string()))?;

        future
            .wait(None)
            .map_err(|e| SessionError::FutureError(e.to_string()))?;

        Ok(())
    }

    pub fn create_program(&self, spirv: Vec<u8>) -> Result<Program, ProgramError> {
        if spirv.is_empty() {
            return Err(ProgramError::CreationFailed("Empty SPIR-V code".to_string()));
        }

        let module = unsafe {
            vulkano::shader::ShaderModule::new(
                self.device.clone(),
                vulkano::shader::ShaderModuleCreateInfo::new(bytemuck::cast_slice(&spirv)),
            )
        }
        .map_err(|e| ProgramError::CreationFailed(e.to_string()))?;

        Ok(Program::new(module))
    }

    pub fn create_program_from_shader_module(
        &self,
        module: Arc<vulkano::shader::ShaderModule>,
    ) -> Result<Program, ProgramError> {
        Ok(Program::new(module))
    }

    pub fn device(&self) -> Arc<Device> {
        self.device.clone()
    }

    /// Evaluates a Luau script in this session's interpreter.
    ///
    /// Session-level globals (e.g. `load_program`) are available because
    /// [`Session::new`] already wired the back-reference.
    ///
    /// # Errors
    ///
    /// Returns [`SessionError::RuntimeError`] if the interpreter lock is
    /// poisoned or if the script raises a Luau error.
    pub fn run_script(&self, script: &str) -> Result<(), SessionError> {
        self.interpreter
            .lock()
            .map_err(|_| SessionError::RuntimeError("Interpreter lock poisoned".to_string()))?
            .exec_script(script)
            .map_err(|e| SessionError::RuntimeError(e.to_string()))
    }

    pub fn load_program(&self, path: &str) -> Result<Program, ProgramError> {
        // validate if path contains .spv extension, if not, add it.
        let path = if path.ends_with(".spv") {
            path.to_string()
        } else {
            format!("{path}.spv")
        };

        for repo in self.repositories.iter() {
            if let Ok(spirv) = repo.load(&path) {
                return self.create_program(spirv);
            }
        }

        Err(ProgramError::CreationFailed(format!(
            "Failed to load program, not found: {path}"
        )))
    }

    pub fn load_compute_node_builder(
        self: &Arc<Self>,
        name: &str,
    ) -> Result<crate::node::ComputeNodeBuilder, SessionError> {
        let name_parts: Vec<&str> = name.split('/').collect();
        let last_part = name_parts
            .last()
            .ok_or_else(|| SessionError::RuntimeError(format!("Invalid compute node builder name: {name}")))?;

        let luau_path = format!("{name}/{last_part}.luau");

        let luau_bytes = self
            .repositories
            .iter()
            .find_map(|repo| repo.load(&luau_path).ok())
            .ok_or_else(|| SessionError::BuilderNotFound(name.to_string()))?;

        let script_content = std::str::from_utf8(&luau_bytes)
            .map_err(|e| SessionError::RuntimeError(format!("Invalid Luau script content: {e:?}")))?;

        let builder_table_key = {
            let interpreter = self.interpreter.lock().unwrap();
            interpreter
                .load_compute_node_builder(script_content, name)
                .map_err(|e| SessionError::RuntimeError(format!("Interpreter failed to load builder: {e:?}")))?
        };

        let builder = crate::interpreter::LuauComputeNodeBuilder {
            interpreter: self.interpreter.clone(),
            builder_table_key,
            name: name.to_string(),
        };

        Ok(crate::node::ComputeNodeBuilder::new(Box::new(builder), self.clone()))
    }

    pub fn load_container_node_builder(
        self: &Arc<Self>,
        name: &str,
    ) -> Result<Box<dyn crate::node::ContainerNodeBuilder>, SessionError> {
        let name_parts: Vec<&str> = name.split('/').collect();
        let last_part = name_parts
            .last()
            .ok_or_else(|| SessionError::RuntimeError(format!("Invalid container node builder name: {name}")))?;

        let luau_path = format!("{name}/{last_part}.luau");

        let luau_bytes = self
            .repositories
            .iter()
            .find_map(|repo| repo.load(&luau_path).ok())
            .ok_or_else(|| SessionError::BuilderNotFound(name.to_string()))?;

        let script_content = std::str::from_utf8(&luau_bytes)
            .map_err(|e| SessionError::RuntimeError(format!("Invalid Luau script content: {e:?}")))?;

        let builder_table_key = {
            let interpreter = self.interpreter.lock().unwrap();
            interpreter
                .load_container_node_builder(script_content, name)
                .map_err(|e| SessionError::RuntimeError(format!("Interpreter failed to load builder: {e:?}")))?
        };

        let builder = crate::interpreter::LuauContainerNodeBuilder {
            interpreter: self.interpreter.clone(),
            builder_table_key,
        };

        Ok(Box::new(builder))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_session() -> Result<()> {
        let descriptor = SessionDescriptor::builder().build();
        let _ = Session::new(descriptor)?;
        Ok(())
    }
}
