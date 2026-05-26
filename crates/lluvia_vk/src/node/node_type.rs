//! Node type enum.

use super::ComputeNodeError;

// ---------------------------------------------------------------------------
// NodeType
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum NodeType {
    Compute = 0,
    Container = 1,
}

impl From<NodeType> for u32 {
    fn from(node_type: NodeType) -> u32 {
        node_type as u32
    }
}

impl TryFrom<u32> for NodeType {
    type Error = ComputeNodeError;

    fn try_from(node_type: u32) -> Result<Self, Self::Error> {
        match node_type {
            0 => Ok(NodeType::Compute),
            1 => Ok(NodeType::Container),
            _ => Err(ComputeNodeError::InvalidNodeType(node_type)),
        }
    }
}
