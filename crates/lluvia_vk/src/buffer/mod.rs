//! Buffer abstractions for Vulkan buffer management.
//!
//! This module mirrors the C++ `ll::Buffer` class, wrapping
//! `vulkano::buffer::Buffer` / `Subbuffer`.

use std::sync::Arc;

use thiserror::Error;
use vulkano::buffer::{Buffer as VkBuffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};

#[derive(Error, Debug)]
pub enum BufferError {
    #[error("Buffer creation failed: {0}")]
    CreationFailed(String),
}

/// Usage flags for buffers, mirroring C++ `ll::BufferUsageFlagBits`.
///
/// We re-export `vulkano::buffer::BufferUsage` directly since it provides the
/// same bitflags (`STORAGE_BUFFER`, `TRANSFER_SRC`, `TRANSFER_DST`,
/// `UNIFORM_BUFFER`).
pub type LlBufferUsage = BufferUsage;

/// A GPU buffer backed by `vulkano::buffer::Subbuffer<[u8]>`.
pub struct Buffer {
    inner: Subbuffer<[u8]>,
    size: u64,
    // usage: BufferUsage,
}

impl Buffer {
    /// Creates a new buffer using the given allocator.
    ///
    /// The default usage flags match the C++ default:
    /// `StorageBuffer | TransferSrc | TransferDst`.
    pub(crate) fn new(
        allocator: Arc<StandardMemoryAllocator>,
        size: u64,
        usage: BufferUsage,
        memory_type_filter: MemoryTypeFilter,
    ) -> Result<Self, BufferError> {
        let create_info = BufferCreateInfo {
            usage,
            ..Default::default()
        };

        let alloc_info = AllocationCreateInfo {
            memory_type_filter,
            ..Default::default()
        };

        let inner = VkBuffer::new_slice::<u8>(allocator, create_info, alloc_info, size)
            .map_err(|e| BufferError::CreationFailed(e.to_string()))?;

        Ok(Self { inner, size })
    }

    /// Creates a device-local storage buffer (the most common case).
    pub(crate) fn new_device_local(allocator: Arc<StandardMemoryAllocator>, size: u64) -> Result<Self, BufferError> {
        let usage = BufferUsage::STORAGE_BUFFER | BufferUsage::TRANSFER_SRC | BufferUsage::TRANSFER_DST;
        Self::new(allocator, size, usage, MemoryTypeFilter::PREFER_DEVICE)
    }

    /// Creates a host-visible storage buffer.
    pub(crate) fn new_host_visible(allocator: Arc<StandardMemoryAllocator>, size: u64) -> Result<Self, BufferError> {
        let usage = BufferUsage::STORAGE_BUFFER | BufferUsage::TRANSFER_SRC | BufferUsage::TRANSFER_DST;
        Self::new(
            allocator,
            size,
            usage,
            MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
        )
    }

    /// Returns the requested buffer size in bytes.
    pub fn size(&self) -> u64 {
        self.size
    }

    // /// Returns the buffer usage flags.
    // pub fn usage(&self) -> BufferUsage {
    //     self.usage
    // }

    pub fn write(&self, data: &[u8]) {
        if self.size < data.len() as u64 {
            panic!("Buffer is too small to hold the data");
        }

        let mut write_guard = self.inner.write().expect("Failed to lock buffer for writing");
        write_guard.copy_from_slice(data);
    }

    pub fn read(&self) -> Vec<u8> {
        let read_guard = self.inner.read().expect("Failed to lock buffer for reading");
        read_guard.to_vec()
    }
}

#[cfg(test)]
mod test {
    use crate::session::{Session, SessionDescriptor};
    use anyhow::Result;

    #[test]
    fn test_buffer_device_local() -> Result<()> {
        let session = Session::new(SessionDescriptor::builder().build())?;
        let buffer = session.create_buffer_device_local(256)?;
        assert_eq!(buffer.size(), 256);
        Ok(())
    }

    #[test]
    fn test_buffer_host_visible() -> Result<()> {
        let session = Session::new(SessionDescriptor::builder().build())?;
        let buffer = session.create_buffer_host_visible(128)?;
        assert_eq!(buffer.size(), 128);
        Ok(())
    }
}
