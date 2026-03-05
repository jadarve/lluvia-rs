use thiserror::Error;

#[derive(Error, Debug)]
pub enum LluviaGpuError {
    #[error("Failed to create a buffer: {0}")]
    BufferCreationError(String),

    #[error("Failed to map buffer: {0}")]
    BufferMapError(String),
}
