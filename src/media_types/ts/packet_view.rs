use crate::media_types::common::to_hex_string;

use super::{AdaptationFieldControl, TransportScramblingControl, TsError};

pub const PACKET_SIZE: usize = 188;
pub const PACKET_SYNC_BYTE: u8 = 0x47;

/// A view of a Transport Stream packet from a slice of bytes.
/// See ISO/IEC 13818-1:2023, section 2.4.3
pub struct PacketView {
    // data: &'a [u8; PACKET_SIZE],
    data: bytes::Bytes,
}

impl PacketView {
    pub fn new(data: bytes::Bytes) -> Self {
        PacketView { data: data }
    }

    pub fn sync_byte(&self) -> u8 {
        self.data[0]
    }

    pub fn transport_error_indicator(&self) -> bool {
        (self.data[1] & 0b1000_0000) != 0
    }

    pub fn payload_unit_start_indicator(&self) -> bool {
        (self.data[1] & 0b0100_0000) != 0
    }

    pub fn transport_priority(&self) -> bool {
        (self.data[1] & 0b0010_0000) != 0
    }

    pub fn pid(&self) -> u16 {
        ((self.data[1] as u16 & 0b0001_1111) << 8) | self.data[2] as u16
    }

    pub fn transport_scrambling_control(&self) -> TransportScramblingControl {
        TransportScramblingControl::from((self.data[3] & 0b1100_0000) >> 6)
    }

    pub fn adaptation_field_control(&self) -> AdaptationFieldControl {
        AdaptationFieldControl::from((self.data[3] & 0b0011_0000) >> 4)
    }

    pub fn continuity_counter(&self) -> u8 {
        self.data[3] & 0b0000_1111
    }

    pub fn adaptation_field(&self) -> Result<AdaptationFieldView, TsError> {
        let adaptation_field_control = self.adaptation_field_control();
        // read the lenght of the adaptation field, even if there is not adaptation field
        // this is useful for the pattern matching below
        let adaptation_field_length = self.data[4] as usize;

        match (adaptation_field_control, adaptation_field_length) {
            // the only allowed length for AdaptationFieldOnly is 183
            (AdaptationFieldControl::AdaptationFieldOnly, 183) => {
                Ok(AdaptationFieldView::new(self.data.slice(4..)))
            }
            // any other length is invalid
            (AdaptationFieldControl::AdaptationFieldOnly, _) => {
                Err(TsError::InvalidAdaptationFieldLength(
                    adaptation_field_control,
                    adaptation_field_length as u8,
                ))
            }
            // if there is adaptation field and payload, the length must be between 0 and 182 inclusive
            (AdaptationFieldControl::AdaptationFieldAndPayload, 0..=182) => Ok(
                AdaptationFieldView::new(self.data.slice(4..4 + adaptation_field_length + 1)),
            ),
            _ => Err(TsError::NoAdaptationField),
        }
    }

    pub fn payload(&self) -> Result<PayloadView, TsError> {
        let adaptation_field_control = self.adaptation_field_control();
        let adaptation_field_length = self.data[4] as usize;

        match adaptation_field_control {
            AdaptationFieldControl::AdaptationFieldAndPayload => {
                let payload_start = 4 + adaptation_field_length + 1;
                if payload_start < 187 {
                    Ok(PayloadView::new(&self.data[payload_start..]))
                } else {
                    Err(TsError::InvalidPayloadStartOffset(payload_start))
                }
            }
            AdaptationFieldControl::PayloadOnly => {
                let payload_start = 4;
                Ok(PayloadView::new(&self.data[payload_start..]))
            }
            _ => Err(TsError::NoAdaptationField),
        }
    }
}

impl std::fmt::Debug for PacketView {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut w = f.debug_struct("PacketView");
        w.field("sync_byte", &self.sync_byte())
            .field(
                "transport_error_indicator",
                &self.transport_error_indicator(),
            )
            .field(
                "payload_unit_start_indicator",
                &self.payload_unit_start_indicator(),
            )
            .field("transport_priority", &self.transport_priority())
            .field("pid", &self.pid())
            .field(
                "transport_scrambling_control",
                &self.transport_scrambling_control(),
            );

        let adaptation_field_control = self.adaptation_field_control();
        w.field("adaptation_field_control", &adaptation_field_control)
            .field("continuity_counter", &self.continuity_counter());

        match adaptation_field_control {
            AdaptationFieldControl::AdaptationFieldOnly => {
                w.field("adaptation_field", &self.adaptation_field());
            }
            AdaptationFieldControl::AdaptationFieldAndPayload => {
                w.field("adaptation_field", &self.adaptation_field());
                w.field("payload", &self.payload());
            }
            AdaptationFieldControl::PayloadOnly => {
                w.field("payload", &self.payload());
            }
            _ => {}
        }

        w.finish()
    }
}

pub struct AdaptationFieldView {
    data: bytes::Bytes,
}

impl AdaptationFieldView {
    const DISCONTINUITY_INDICATOR_MASK: u8 = 0b1000_0000;

    pub fn new(data: bytes::Bytes) -> Self {
        AdaptationFieldView { data }
    }

    pub fn length(&self) -> u8 {
        self.data[0]
    }

    pub fn discontinuity_indicator(&self) -> Result<bool, TsError> {
        // TODO: could extract all the flags in a single
        if self.length() > 0 {
            Ok((self.data[1] & Self::DISCONTINUITY_INDICATOR_MASK) != 0)
        } else {
            Err(TsError::EmptyAdaptationField(
                "attempting to read discontinuity indicator".to_string(),
            ))
        }
    }

    pub fn random_access_indicator(&self) -> Result<bool, TsError> {
        if self.length() > 0 {
            Ok((self.data[1] & 0b0100_0000) != 0)
        } else {
            Err(TsError::EmptyAdaptationField(
                "attempting to read random access indicator".to_string(),
            ))
        }
    }

    pub fn elementary_stream_priority_indicator(&self) -> Result<bool, TsError> {
        if self.length() > 0 {
            Ok((self.data[1] & 0b0010_0000) != 0)
        } else {
            Err(TsError::EmptyAdaptationField(
                "attempting to read elementary stream priority indicator".to_string(),
            ))
        }
    }

    pub fn pcr_flag(&self) -> Result<bool, TsError> {
        if self.length() > 0 {
            Ok((self.data[1] & 0b0001_0000) != 0)
        } else {
            Err(TsError::EmptyAdaptationField(
                "attempting to read PCR flag".to_string(),
            ))
        }
    }

    pub fn opcr_flag(&self) -> Result<bool, TsError> {
        if self.length() > 0 {
            Ok((self.data[1] & 0b0000_1000) != 0)
        } else {
            Err(TsError::EmptyAdaptationField(
                "attempting to read OPCR flag".to_string(),
            ))
        }
    }

    pub fn splicing_point_flag(&self) -> Result<bool, TsError> {
        if self.length() > 0 {
            Ok((self.data[1] & 0b0000_0100) != 0)
        } else {
            Err(TsError::EmptyAdaptationField(
                "attempting to read splicing point flag".to_string(),
            ))
        }
    }

    pub fn transport_private_data_flag(&self) -> Result<bool, TsError> {
        if self.length() > 0 {
            Ok((self.data[1] & 0b0000_0010) != 0)
        } else {
            Err(TsError::EmptyAdaptationField(
                "attempting to read transport private data flag".to_string(),
            ))
        }
    }

    pub fn adaptation_field_extension_flag(&self) -> Result<bool, TsError> {
        if self.length() > 0 {
            Ok((self.data[1] & 0b0000_0001) != 0)
        } else {
            Err(TsError::EmptyAdaptationField(
                "attempting to read adaptation field extension flag".to_string(),
            ))
        }
    }
}

impl std::fmt::Debug for AdaptationFieldView {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("AdaptationFieldView")
            .field("length", &self.length())
            .field("discontinuity_indicator", &self.discontinuity_indicator())
            .field("random_access_indicator", &self.random_access_indicator())
            .field(
                "elementary_stream_priority_indicator",
                &self.elementary_stream_priority_indicator(),
            )
            .field("pcr_flag", &self.pcr_flag())
            .field("opcr_flag", &self.opcr_flag())
            .field("splicing_point_flag", &self.splicing_point_flag())
            .field(
                "transport_private_data_flag",
                &self.transport_private_data_flag(),
            )
            .field(
                "adaptation_field_extension_flag",
                &self.adaptation_field_extension_flag(),
            )
            .finish()
    }
}

pub struct PayloadView<'a> {
    data: &'a [u8],
}

impl<'a> PayloadView<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        PayloadView { data }
    }

    pub fn length(&self) -> usize {
        self.data.len()
    }
}

impl std::fmt::Debug for PayloadView<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PayloadView")
            .field("length", &self.data.len())
            .field("data", &to_hex_string(&self.data))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::media_types::ts::packet_view;

    use super::*;

    #[test]
    fn test_read_file() -> Result<(), String> {
        let data = {
            let bytes_vec = std::fs::read("local/sample.ts").map_err(|e| e.to_string())?;
            bytes::Bytes::from_owner(bytes_vec)
        };

        print!("data len: {}", data.len());

        for i in 0..data.len() / PACKET_SIZE {
            let packet_slice = data.slice(i * PACKET_SIZE..(i + 1) * PACKET_SIZE);
            let packet_view = PacketView::new(packet_slice);
            println!("{:?}", packet_view);
        }

        Ok(())
    }
}
