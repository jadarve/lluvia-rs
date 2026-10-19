//! Node port and push-constants types.

use std::sync::Arc;

use crate::buffer::Buffer;
use crate::image::ImageView;

// ---------------------------------------------------------------------------
// NodePort
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub enum NodePort {
    Buffer(Arc<Buffer>),
    ImageView(Arc<ImageView>),
}

// ---------------------------------------------------------------------------
// PushConstants
// ---------------------------------------------------------------------------

#[derive(Clone, Default)]
pub struct PushConstants {
    data: Vec<u8>,
}

impl PushConstants {
    pub fn push_f32(&mut self, value: f32) {
        self.data.extend_from_slice(&value.to_ne_bytes());
    }

    pub fn push_i32(&mut self, value: i32) {
        self.data.extend_from_slice(&value.to_ne_bytes());
    }

    pub fn size(&self) -> u32 {
        self.data.len() as u32
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}
