use super::{AdaptationFieldControl, TransportScramblingControl, TsError};

pub const PACKET_SIZE: usize = 188;
pub const PACKET_SYNC_BYTE: u8 = 0x47;

/// A view of a Transport Stream packet from a slice of bytes.
/// See ISO/IEC 13818-1:2023, section 2.4.3
pub struct PacketView<'a> {
    data: &'a [u8; PACKET_SIZE],
}

impl<'a> PacketView<'a> {
    pub fn new(data: &'a [u8; PACKET_SIZE]) -> Self {
        PacketView { data }
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

    // fn adaptation_field(&self) -> Option<AdaptationField> {
    //     match self.adaptation_field_control() {
    //         AdaptationFieldControl::AdaptationFieldOnly
    //         | AdaptationFieldControl::AdaptationFieldAndPayload => {
    //             Some(AdaptationField::from_bytes(&self.data[4..]).unwrap())
    //         }
    //         _ => None,
    //     }
    // }

    // fn payload(&self) -> Option<&[u8]> {
    //     None
    // }
}

impl std::fmt::Debug for PacketView<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PacketView")
            .field("sync_byte", &self.sync_byte())
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
            )
            .field("adaptation_field_control", &self.adaptation_field_control())
            .field("continuity_counter", &self.continuity_counter())
            .finish()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_file() -> Result<(), String> {
        let data = std::fs::read("local/sample.ts").map_err(|e| e.to_string())?;

        print!("data len: {}", data.len());

        for i in 0..data.len() / PACKET_SIZE {
            let packet_slice: &[u8; 188] = &data[i * PACKET_SIZE..(i + 1) * PACKET_SIZE]
                .try_into()
                .unwrap();

            let packet_view = PacketView::new(packet_slice);
            println!("{:?}", packet_view);

            // given the packet slice, we can create a TsPacket

            // let packet = TsPacket::from_bytes(packet);
            // println!("{:?}", packet_slice);
        }

        Ok(())
    }
}
