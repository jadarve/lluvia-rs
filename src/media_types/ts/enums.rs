#[derive(Debug)]
#[repr(u8)]
pub enum AdaptationFieldControl {
    Reserved = 0b00,
    PayloadOnly = 0b01,
    AdaptationFieldOnly = 0b10,
    AdaptationFieldAndPayload = 0b11,
}

impl From<u8> for AdaptationFieldControl {
    /// Convert a u8 value into an AdaptationFieldControl enum.
    ///
    /// Internally, the value parameter is masked to only consider the two least significant bits,
    /// and hence this method will alawys return an AdaptationFieldControl enum.
    fn from(value: u8) -> Self {
        match value & 0b0000_0011 {
            0 => AdaptationFieldControl::Reserved,
            1 => AdaptationFieldControl::PayloadOnly,
            2 => AdaptationFieldControl::AdaptationFieldOnly,
            3 => AdaptationFieldControl::AdaptationFieldAndPayload,
            _ => unreachable!("Unexpected adaptation field control value: {value:0X}"),
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

impl From<u8> for TransportScramblingControl {
    /// Convert a u8 value into a TransportScramblingControl enum.
    ///
    /// Internally, the value parameter is masked to only consider the two least significant bits,
    /// and hence this method will always return a TransportScramblingControl enum.
    fn from(value: u8) -> Self {
        match value & 0b0000_0011 {
            0 => TransportScramblingControl::NotScrambled,
            1 => TransportScramblingControl::UserDefined1,
            2 => TransportScramblingControl::UserDefined2,
            3 => TransportScramblingControl::UserDefined3,
            _ => unreachable!("Unexepected transport scrambling control value: {value:0X}"),
        }
    }
}
