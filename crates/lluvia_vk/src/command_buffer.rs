use std::sync::Arc;
use thiserror::Error;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer};

use crate::buffer::Buffer;
use crate::node::{ComputeNode, ComputeNodeError, Node};

#[derive(Error, Debug)]
pub enum CommandBufferError {
    #[error("Vulkano command buffer error: {0}")]
    VulkanoError(#[from] vulkano::command_buffer::CommandBufferExecError),
    #[error("Compute node error: {0}")]
    NodeError(#[from] ComputeNodeError),
    #[error("Wait error: {0}")]
    WaitError(#[from] vulkano::sync::HostAccessError),
    #[error("Other Vulkan error: {0}")]
    VulkanError(#[from] vulkano::VulkanError),
    #[error("Validation error: {0}")]
    ValidationError(#[from] Box<vulkano::ValidationError>),
    #[error("Runtime error: {0}")]
    RuntimeError(String),
}

pub struct CommandBufferBuilder {
    builder: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
}

impl CommandBufferBuilder {
    pub(crate) fn new(
        allocator: Arc<StandardCommandBufferAllocator>,
        queue_family_index: u32,
    ) -> Result<Self, CommandBufferError> {
        let builder =
            AutoCommandBufferBuilder::primary(allocator, queue_family_index, CommandBufferUsage::OneTimeSubmit)
                .map_err(|e| CommandBufferError::RuntimeError(e.to_string()))?;

        Ok(Self { builder })
    }

    pub fn copy_buffer(&mut self, src: Arc<Buffer>, dst: Arc<Buffer>) -> Result<(), CommandBufferError> {
        self.builder
            .copy_buffer(vulkano::command_buffer::CopyBufferInfo::buffers(
                src.inner().clone(),
                dst.inner().clone(),
            ))
            .map_err(|e| CommandBufferError::RuntimeError(e.to_string()))?;
        Ok(())
    }

    pub fn copy_buffer_to_image(
        &mut self,
        src: Arc<Buffer>,
        dst: Arc<crate::image::Image>,
    ) -> Result<(), CommandBufferError> {
        self.builder
            .copy_buffer_to_image(vulkano::command_buffer::CopyBufferToImageInfo::buffer_image(
                src.inner().clone(),
                dst.inner().clone(),
            ))
            .map_err(|e| CommandBufferError::RuntimeError(e.to_string()))?;
        Ok(())
    }

    pub fn copy_image_to_buffer(
        &mut self,
        src: Arc<crate::image::Image>,
        dst: Arc<Buffer>,
    ) -> Result<(), CommandBufferError> {
        self.builder
            .copy_image_to_buffer(vulkano::command_buffer::CopyImageToBufferInfo::image_buffer(
                src.inner().clone(),
                dst.inner().clone(),
            ))
            .map_err(|e| CommandBufferError::RuntimeError(e.to_string()))?;
        Ok(())
    }

    pub fn record_compute_node(&mut self, node: &ComputeNode) -> Result<(), CommandBufferError> {
        node.record(&mut self.builder)?;
        Ok(())
    }

    pub(crate) fn build(self) -> Result<Arc<PrimaryAutoCommandBuffer>, CommandBufferError> {
        self.builder
            .build()
            .map_err(|e| CommandBufferError::RuntimeError(e.to_string()))
    }
}

#[derive(Clone)]
pub struct CommandBuffer {
    pub(crate) inner: Arc<PrimaryAutoCommandBuffer>,
}

impl CommandBufferBuilder {
    pub fn build_command_buffer(self) -> Result<CommandBuffer, CommandBufferError> {
        Ok(CommandBuffer { inner: self.build()? })
    }
}
