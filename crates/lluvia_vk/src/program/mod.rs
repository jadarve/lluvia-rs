//! Program abstraction for loading SPIR-V shader modules.
//!
//! Mirrors the C++ `ll::Program` class, wrapping
//! `vulkano::shader::ShaderModule`.

use std::sync::Arc;

use thiserror::Error;
use vulkano::shader::ShaderModule;

#[derive(Error, Debug)]
pub enum ProgramError {
    #[error("Program creation failed: {0}")]
    CreationFailed(String),
}

/// A compiled shader program loaded from SPIR-V bytecode.
///
/// Mirrors C++ `ll::Program`.
#[derive(Clone, Debug)]
pub struct Program {
    module: Arc<ShaderModule>,
}

impl Program {
    pub(crate) fn new(module: Arc<ShaderModule>) -> Self {
        Self { module }
    }

    /// Returns a reference to the underlying shader module.
    pub fn shader_module(&self) -> &Arc<ShaderModule> {
        &self.module
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
