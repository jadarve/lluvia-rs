use std::sync::Arc;

use thiserror::Error;
use vulkano::VulkanLibrary;
use vulkano::device::physical::PhysicalDeviceType;
use vulkano::device::{Device, DeviceCreateInfo, Queue, QueueCreateInfo, QueueFlags};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::memory::allocator::StandardMemoryAllocator;

#[derive(Error, Debug)]
pub enum SessionError {
    #[error("Failed to create Vulkan instance")]
    InstanceCreationError(#[from] vulkano::LoadingError),

    #[error("Runtime error: {0}")]
    RuntimeError(String),
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

#[derive(Debug, Clone, Default)]
pub struct SessionDescriptor {
    pub enable_debug: bool,
    pub device_descriptor: Option<DeviceDescriptor>,
}

impl SessionDescriptor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enable_debug(mut self, enable: bool) -> Self {
        self.enable_debug = enable;
        self
    }

    pub fn device_descriptor(mut self, descriptor: DeviceDescriptor) -> Self {
        self.device_descriptor = Some(descriptor);
        self
    }
}

pub struct Session {
    instance: Arc<Instance>,
    device: Arc<Device>,
    compute_queue: Arc<Queue>,
    allocator: Arc<StandardMemoryAllocator>,
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
                .ok_or_else(|| {
                    SessionError::RuntimeError("Requested device not found".to_string())
                })?
        } else {
            physical_devices
                .clone()
                .into_iter()
                .find(|dev| dev.properties().device_type == PhysicalDeviceType::DiscreteGpu)
                .or_else(|| physical_devices.into_iter().next())
                .ok_or_else(|| {
                    SessionError::RuntimeError("No physical devices found".to_string())
                })?
        };

        let queue_family_index = physical_device
            .queue_family_properties()
            .iter()
            .enumerate()
            .find(|(_index, properties)| properties.queue_flags.intersects(QueueFlags::COMPUTE))
            .map(|(index, _)| index as u32)
            .ok_or_else(|| {
                SessionError::RuntimeError("No compute queue family found".to_string())
            })?;

        let device_create_info = DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            ..Default::default()
        };

        let (device, mut queues) = Device::new(physical_device, device_create_info)
            .map_err(|e| SessionError::RuntimeError(format!("device creation: {e}")))?;

        let compute_queue = queues
            .next()
            .ok_or_else(|| SessionError::RuntimeError("Failed to get compute queue".to_string()))?;

        let allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        Ok(Arc::new(Self {
            instance,
            device,
            compute_queue,
            allocator,
        }))
    }

    pub fn instance(&self) -> Arc<Instance> {
        self.instance.clone()
    }

    pub fn device(&self) -> Arc<Device> {
        self.device.clone()
    }

    pub fn compute_queue(&self) -> Arc<Queue> {
        self.compute_queue.clone()
    }

    pub fn allocator(&self) -> Arc<StandardMemoryAllocator> {
        self.allocator.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_session() -> Result<()> {
        let descriptor = SessionDescriptor::new();
        let session = Session::new(descriptor)?;
        // Just verify we got a session.
        let _device = session.device();
        Ok(())
    }
}
