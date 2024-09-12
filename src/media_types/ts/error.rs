use thiserror::Error;

#[derive(Error, Debug)]
pub enum TsError {
    /// The sync byte is not equal to TS_PACKET_SYNC_BYTE
    /// The error contains the invalid sync byte detected
    #[error("Invalid sync byte: {0}")]
    InvalidSyncByte(u8),
}
