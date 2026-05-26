//! Port descriptor types for compute nodes.

use super::ComputeNodeError;

// ---------------------------------------------------------------------------
// PortDirection
// ---------------------------------------------------------------------------

/// Direction of a port (input or output).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum PortDirection {
    In = 0,
    Out = 1,
}

impl From<PortDirection> for u32 {
    fn from(direction: PortDirection) -> u32 {
        direction as u32
    }
}

impl TryFrom<u32> for PortDirection {
    type Error = ComputeNodeError;

    fn try_from(direction: u32) -> Result<Self, Self::Error> {
        match direction {
            0 => Ok(PortDirection::In),
            1 => Ok(PortDirection::Out),
            _ => Err(ComputeNodeError::InvalidPortDirection(direction)),
        }
    }
}

// ---------------------------------------------------------------------------
// PortType
// ---------------------------------------------------------------------------

/// Type of object a port accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum PortType {
    Buffer = 0,
    ImageView = 1,
}

impl From<PortType> for u32 {
    fn from(port_type: PortType) -> u32 {
        port_type as u32
    }
}

impl TryFrom<u32> for PortType {
    type Error = ComputeNodeError;

    fn try_from(port_type: u32) -> Result<Self, Self::Error> {
        match port_type {
            0 => Ok(PortType::Buffer),
            1 => Ok(PortType::ImageView),
            _ => Err(ComputeNodeError::InvalidPortType(port_type)),
        }
    }
}

// ---------------------------------------------------------------------------
// PortDescriptor
// ---------------------------------------------------------------------------

/// Descriptor for a node port, mirroring C++ `ll::PortDescriptor`.
#[derive(Debug, Clone)]
pub struct PortDescriptor {
    pub binding: u32,
    pub name: String,
    pub direction: PortDirection,
    pub port_type: PortType,
}

impl Default for PortDescriptor {
    fn default() -> Self {
        Self {
            binding: 0,
            name: String::new(),
            direction: PortDirection::In,
            port_type: PortType::Buffer,
        }
    }
}
