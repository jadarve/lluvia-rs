use anyhow::Result;

use super::{PacketView, PACKET_SIZE};

pub struct TsMemoryReader {
    buffer: bytes::Bytes,
}

impl TsMemoryReader {
    pub fn new(filename: &str) -> Result<Self> {
        let buffer = {
            let bytes_vec = std::fs::read(filename)?;
            bytes::Bytes::from_owner(bytes_vec)
        };

        Ok(Self { buffer })
    }

    pub fn at(&self, i: usize) -> Result<PacketView> {
        // TODO: Add bounds test
        // TODO: Find sync byte alignment offset with respect to first packet
        let packet_slice = self.buffer.slice(i * PACKET_SIZE..(i + 1) * PACKET_SIZE);

        Ok(PacketView::new(packet_slice))
    }

    pub fn len(&self) -> usize {
        self.buffer.len() / PACKET_SIZE
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_read_file() -> Result<()> {
        let reader = TsMemoryReader::new("../../local/sample.ts")?;
        assert!(!reader.is_empty(), "There should be packets in the file");

        for i in 0..reader.len() {
            let pkt = reader.at(i)?;
            assert!(pkt.is_valid().is_ok());
            println!("Packet: {pkt:?}");

            // TODO: should assert that the packet is well formed
        }

        Ok(())
    }
}
