//! Program abstraction for loading SPIR-V shader modules.
//!
//! Mirrors the C++ `ll::Program` class, wrapping
//! `vulkano::shader::ShaderModule`.

use std::sync::Arc;

use thiserror::Error;
use vulkano::device::Device;
use vulkano::shader::ShaderModule;

#[derive(Error, Debug)]
pub enum ProgramError {
    #[error("Program creation failed: {0}")]
    CreationFailed(String),

    #[error("Empty SPIR-V code")]
    EmptySpirv,
}

/// A compiled shader program loaded from SPIR-V bytecode.
///
/// Mirrors C++ `ll::Program`.
pub struct Program {
    module: Arc<ShaderModule>,
    spirv: Vec<u8>,
}

impl Program {
    /// Creates a new program from raw SPIR-V bytes.
    ///
    /// The SPIR-V code must be a valid SPIR-V module (aligned to 4 bytes).
    pub fn from_spirv(device: Arc<Device>, spirv: Vec<u8>) -> Result<Self, ProgramError> {
        if spirv.is_empty() {
            return Err(ProgramError::EmptySpirv);
        }

        // SAFETY: We trust the caller provides valid SPIR-V; vulkano will
        // validate it at module creation time via the Vulkan driver.
        let module = unsafe {
            ShaderModule::new(
                device,
                vulkano::shader::ShaderModuleCreateInfo::new(bytemuck::cast_slice(&spirv)),
            )
        }
        .map_err(|e| ProgramError::CreationFailed(e.to_string()))?;

        Ok(Self { module, spirv })
    }

    /// Creates a new program by reading a SPIR-V file from disk.
    pub fn from_file(device: Arc<Device>, path: &std::path::Path) -> Result<Self, ProgramError> {
        let spirv = std::fs::read(path)
            .map_err(|e| ProgramError::CreationFailed(format!("Failed to read {}: {e}", path.display())))?;
        Self::from_spirv(device, spirv)
    }

    /// Returns a reference to the underlying shader module.
    pub fn shader_module(&self) -> &Arc<ShaderModule> {
        &self.module
    }

    /// Returns the raw SPIR-V code.
    pub fn spirv(&self) -> &[u8] {
        &self.spirv
    }
}

#[cfg(test)]
mod test {
    // use super::*;

    // #[test]
    // fn test_empty_spirv_returns_error() {
    //     // We cannot construct a device without Vulkan, but we can at least
    //     // verify the empty-check path works without a device.
    //     let result = Program::from_spirv(
    //         // This will never be reached because the empty check fires first.
    //         // But we need a dummy — so we just check the error variant.
    //         panic_device(),
    //         vec![],
    //     );
    //     assert!(result.is_err());
    // }

    // /// Helper that would panic if called — used to prove that `from_spirv`
    // /// short-circuits before touching the device.
    // fn panic_device() -> Arc<vulkano::device::Device> {
    //     panic!("device should not be needed for empty-spirv check")
    // }
}
