// TODO: Convert enums to and from their binary values

pub enum TsError {
    /// The sync byte is not equal to TS_PACKET_SYNC_BYTE
    /// The error contains the invalid sync byte detected
    InvalidSyncByte(u8),

    InvalidAdaptationFieldControl(u8),
    InvalidScramblingControl(u8),
}

pub trait FromBytes<E> {
    fn from_bytes(bytes: &[u8]) -> Result<Self, E>
    where
        Self: Sized;
}

pub const TS_PACKET_SIZE: usize = 188;
pub const TS_PACKET_SYNC_BYTE: u8 = 0x47;

#[derive(Debug)]
#[repr(u8)]
pub enum AdaptationFieldControl {
    Reserved = 0b00,
    PayloadOnly = 0b01,
    AdaptationFieldOnly = 0b10,
    AdaptationFieldAndPayload = 0b11,
}

impl TryFrom<u8> for AdaptationFieldControl {
    type Error = TsError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(AdaptationFieldControl::Reserved),
            1 => Ok(AdaptationFieldControl::PayloadOnly),
            2 => Ok(AdaptationFieldControl::AdaptationFieldOnly),
            3 => Ok(AdaptationFieldControl::AdaptationFieldAndPayload),
            _ => Err(TsError::InvalidAdaptationFieldControl(value)),
        }
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum TransportScramblingControl {
    NotScrambled = 0b00,
    UserDefined1 = 0b01,
    UserDefined2 = 0b10,
    UserDefined3 = 0b11,
}

impl TryFrom<u8> for TransportScramblingControl {
    type Error = TsError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(TransportScramblingControl::NotScrambled),
            1 => Ok(TransportScramblingControl::UserDefined1),
            2 => Ok(TransportScramblingControl::UserDefined2),
            3 => Ok(TransportScramblingControl::UserDefined3),
            _ => Err(TsError::InvalidScramblingControl(value)),
        }
    }
}

#[derive(Debug)]
pub struct TsPacket {
    transport_error_indicator: bool,
    payload_unit_start_indicator: bool,
    transport_priority: bool,
    pid: u16,
    transport_scrambling_control: TransportScramblingControl,
    adaptation_field_control: AdaptationFieldControl,
    continuity_counter: u8,
    adaptation_field: Option<AdaptationField>,
    payload: Option<Vec<u8>>,
}

impl FromBytes<TsError> for TsPacket {
    fn from_bytes(bytes: &[u8]) -> Result<Self, TsError> {
        if bytes[0] != TS_PACKET_SYNC_BYTE {
            return Err(TsError::InvalidSyncByte(bytes[0]));
        }

        let transport_error_indicator = (bytes[1] & 0b1000_0000) != 0;
        let payload_unit_start_indicator = (bytes[1] & 0b0100_0000) != 0;
        let transport_priority = (bytes[1] & 0b0010_0000) != 0;
        let pid = ((bytes[1] as u16 & 0b0001_1111) << 8) | bytes[2] as u16;

        let transport_scrambling_control =
            TransportScramblingControl::try_from((bytes[3] & 0b1100_0000) >> 6)?;

        let adaptation_field_control =
            AdaptationFieldControl::try_from((bytes[3] & 0b0011_0000) >> 4)?;

        let continuity_counter = bytes[3] & 0b0000_1111;

        let adaptation_field = match adaptation_field_control {
            AdaptationFieldControl::AdaptationFieldOnly
            | AdaptationFieldControl::AdaptationFieldAndPayload => {
                Some(AdaptationField::from_bytes(&bytes[4..])?)
            }
            _ => None,
        };

        Ok(TsPacket {
            transport_error_indicator,
            payload_unit_start_indicator,
            transport_priority,
            pid,
            transport_scrambling_control,
            adaptation_field_control,
            continuity_counter,
            adaptation_field: adaptation_field,
            payload: None,
        })
    }
}

#[derive(Debug)]
pub struct AdaptationField {
    length: u8,
    payload: Option<AdaptationFieldPayload>,
}

impl FromBytes<TsError> for AdaptationField {
    fn from_bytes(bytes: &[u8]) -> Result<Self, TsError> {
        let length = bytes[0];

        let payload = if length > 0 {
            Some(AdaptationFieldPayload::from_bytes(&bytes[1..])?)
        } else {
            None
        };

        Ok(AdaptationField { length, payload })
    }
}

#[derive(Debug)]
pub struct AdaptationFieldPayload {
    discontinuity_indicator: bool,
    random_access_indicator: bool,
    elementary_stream_priority_indicator: bool,
    pcr_flag: bool,
    opcr_flag: bool,
    splicing_point_flag: bool,
    transport_private_data_flag: bool,
    adaptation_field_extension_flag: bool,

    // PCR
    pcr: Option<AdaptationFieldPCRInfo>,
    opcr: Option<AdaptationFieldPCRInfo>,
    splice_countdown: Option<u8>,
    transport_private_data: Option<AdaptationFieldPrivateData>,
    adaptation_field_extension: Option<AdaptationFieldExtension>,
}

impl FromBytes<TsError> for AdaptationFieldPayload {
    fn from_bytes(bytes: &[u8]) -> Result<Self, TsError> {
        let discontinuity_indicator = (bytes[0] & 0b1000_0000) != 0;
        let random_access_indicator = (bytes[0] & 0b0100_0000) != 0;
        let elementary_stream_priority_indicator = (bytes[0] & 0b0010_0000) != 0;
        let pcr_flag = (bytes[0] & 0b0001_0000) != 0;
        let opcr_flag = (bytes[0] & 0b0000_1000) != 0;
        let splicing_point_flag = (bytes[0] & 0b0000_0100) != 0;
        let transport_private_data_flag = (bytes[0] & 0b0000_0010) != 0;
        let adaptation_field_extension_flag = (bytes[0] & 0b0000_0001) != 0;

        let mut offset: usize = 2;
        let pcr_info = if pcr_flag {
            let info = AdaptationFieldPCRInfo::from_bytes(&bytes[offset..offset + 6])?;
            offset += 6;
            Some(info)
        } else {
            None
        };

        let opcr_info = if opcr_flag {
            let info = AdaptationFieldPCRInfo::from_bytes(&bytes[offset..offset + 6])?;
            offset += 6;
            Some(info)
        } else {
            None
        };

        // aqui voy
        if splicing_point_flag {}

        if transport_private_data_flag {}

        if adaptation_field_extension_flag {}

        Ok(AdaptationFieldPayload {
            discontinuity_indicator,
            random_access_indicator,
            elementary_stream_priority_indicator,
            pcr_flag,
            opcr_flag,
            splicing_point_flag,
            transport_private_data_flag,
            adaptation_field_extension_flag,
            pcr: pcr_info,
            opcr: opcr_info,
            splice_countdown: None,
            transport_private_data: None,
            adaptation_field_extension: None,
        })
    }
}

#[derive(Debug)]
pub struct AdaptationFieldPCRInfo {
    base: u64,
    extension: u16,
}

impl FromBytes<TsError> for AdaptationFieldPCRInfo {
    fn from_bytes(bytes: &[u8]) -> Result<Self, TsError> {
        // TODO: check, AI generated this code
        let base = ((bytes[0] as u64) << 25)
            | ((bytes[1] as u64) << 17)
            | ((bytes[2] as u64) << 9)
            | ((bytes[3] as u64) << 1)
            | ((bytes[4] as u64) >> 7);

        let extension = ((bytes[4] as u16 & 0b0000_0001) << 8) | bytes[5] as u16;

        Ok(AdaptationFieldPCRInfo { base, extension })
    }
}

// quite a generic type
#[derive(Debug)]
pub struct AdaptationFieldPrivateData {
    length: u8, // redundant
    data: Vec<u8>,
}

#[derive(Debug)]
pub struct AdaptationFieldExtension {
    length: u8,
    ltw_flag: bool,
    piecewise_rate_flag: bool,
    seamless_splice_flag: bool,
    // ltw: Option<AdaptationFieldLTW>,
    // piecewise_rate: Option<AdaptationFieldPiecewiseRate>,
    // seamless_splice: Option<AdaptationFieldSeamlessSplice>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_file() -> Result<(), String> {
        let data = std::fs::read("local/sample.ts").map_err(|e| e.to_string())?;

        print!("data len: {}", data.len());

        for i in 0..data.len() / TS_PACKET_SIZE {
            let packet_slice = &data[i * TS_PACKET_SIZE..(i + 1) * TS_PACKET_SIZE];
            // packet_slice.iter().for_each(|b| print!("{:02X}", b));
            // println!("");

            let packet = TsPacket::from_bytes(packet_slice)
                .map_err(|_| String::from("Error mapping bytes to TS packet"))?;

            println!("{:?}", packet);

            // given the packet slice, we can create a TsPacket

            // let packet = TsPacket::from_bytes(packet);
            // println!("{:?}", packet_slice);
        }

        Ok(())
    }
}
