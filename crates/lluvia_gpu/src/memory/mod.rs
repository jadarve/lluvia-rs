//! Memory abstractions for Vulkan memory management.
//!
//! This module provides types mirroring the C++ `ll::Memory` and related
//! memory-property concepts, backed by `vulkano`'s memory allocator.

use std::sync::Arc;

use vulkano::memory::MemoryPropertyFlags;
use vulkano::memory::allocator::StandardMemoryAllocator;

/// Flags describing desired memory properties.
///
/// Maps to the C++ `ll::MemoryPropertyFlagBits` / `ll::MemoryPropertyFlags`.
/// In Rust we re-export `vulkano::memory::MemoryPropertyFlags` directly as it
/// already provides the same bitflags (DEVICE_LOCAL, HOST_VISIBLE, etc.).
pub type LlMemoryPropertyFlags = MemoryPropertyFlags;

/// Information about a Vulkan memory heap, mirroring C++ `ll::VkHeapInfo`.
#[derive(Debug, Clone)]
pub struct VkHeapInfo {
    /// Index into the physical-device memory type array.
    pub type_index: u32,
    /// Heap total size in bytes.
    pub size: u64,
    /// Memory property flags for this type.
    pub flags: MemoryPropertyFlags,
    /// Family queue indices this memory will be used on.
    pub family_queue_indices: Vec<u32>,
}

/// High-level memory handle used to create buffers and images.
///
/// Unlike the C++ implementation which manually pages `VkDeviceMemory`,
/// the Rust version delegates allocation to `vulkano`'s
/// [`StandardMemoryAllocator`].  The allocator is shared with
/// [`Session`](crate::session::Session) and passed into buffer/image
/// creation calls.
pub struct Memory {
    allocator: Arc<StandardMemoryAllocator>,
    heap_info: VkHeapInfo,
}

impl Memory {
    /// Creates a new `Memory` handle.
    pub fn new(allocator: Arc<StandardMemoryAllocator>, heap_info: VkHeapInfo) -> Self {
        Self {
            allocator,
            heap_info,
        }
    }

    /// Returns a reference to the underlying allocator.
    pub fn allocator(&self) -> &Arc<StandardMemoryAllocator> {
        &self.allocator
    }

    /// Returns the heap info associated with this memory.
    pub fn heap_info(&self) -> &VkHeapInfo {
        &self.heap_info
    }

    /// Returns the memory property flags.
    pub fn memory_property_flags(&self) -> MemoryPropertyFlags {
        self.heap_info.flags
    }

    /// Checks whether this memory is host-visible (mappable).
    pub fn is_mappable(&self) -> bool {
        self.heap_info
            .flags
            .intersects(MemoryPropertyFlags::HOST_VISIBLE)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::session::{Session, SessionDescriptor};
    use anyhow::Result;

    #[test]
    fn test_memory_creation() -> Result<()> {
        let session = Session::new(SessionDescriptor::new())?;

        let heap_info = VkHeapInfo {
            type_index: 0,
            size: 1024,
            flags: MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
            family_queue_indices: vec![0],
        };

        let memory = Memory::new(session.allocator(), heap_info);
        assert!(memory.is_mappable());

        Ok(())
    }
}
