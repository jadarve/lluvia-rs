use std::usize;

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

        Ok(Self { buffer: buffer })
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
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_read_file() -> Result<()> {
        let reader = TsMemoryReader::new("local/sample.ts")?;
        assert!(reader.len() > 0, "There should be packets in the file");

        for i in 0..reader.len() {
            let pkt = reader.at(i)?;
            println!("Packet: {pkt:?}");

            // TODO: should assert that the packet is well formed
        }

        Ok(())
    }
}
