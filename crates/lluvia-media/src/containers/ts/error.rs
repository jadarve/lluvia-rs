use thiserror::Error;

use super::AdaptationFieldControl;

#[derive(Error, Debug)]
pub enum TsError {
    /// The sync byte is not equal to TS_PACKET_SYNC_BYTE
    /// The error contains the invalid sync byte detected
    #[error("Invalid sync byte: {0}")]
    InvalidSyncByte(u8),

    /// The adaptation field length is invalid
    #[error(
        "Invalid adaptation field length. Possible values are: \
        (AdaptationFieldOnly, 183) \
        or (AdaptationFieldAndPayload, [0, 182]), \
        got ({0:?}, {1})"
    )]
    InvalidAdaptationFieldLength(AdaptationFieldControl, u8),

    /// The adaptation field is empty, but the caller expected it to be present
    #[error("Empty adaptation field: {0}")]
    EmptyAdaptationField(String),

    /// Invalid payload start offset
    #[error("Invalid payload start offset. Maximum allowed value is 187, got {0}")]
    InvalidPayloadStartOffset(usize),

    /// Attempted to read adaptation field when there is none
    #[error("Attempted to read adaptation field when there is none")]
    NoAdaptationField,
}
