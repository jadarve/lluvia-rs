use std::usize;

use anyhow::Result;

use super::{PacketView, PACKET_SIZE};

pub struct TsMemoryReader {
    buffer: Vec<u8>,
}

impl<'a> TsMemoryReader {
    pub fn new(filename: &str) -> Result<Self> {
        let buffer = std::fs::read(filename)?;

        Ok(Self { buffer: buffer })

        // for i in 0..data.len() / PACKET_SIZE {
        //     let packet_slice: &[u8; 188] = &data[i * PACKET_SIZE..(i + 1) * PACKET_SIZE]
        //         .try_into()
        //         .unwrap();

        //     let packet_view = PacketView::new(packet_slice);
        //     println!("{:?}", packet_view);

        //     // given the packet slice, we can create a TsPacket

        //     // let packet = TsPacket::from_bytes(packet);
        //     // println!("{:?}", packet_slice);
        // }
    }

    pub fn at(&self, i: usize) -> Result<PacketView> {
        // let packet_slice: &'a [u8; 188] =
        //     &self.buffer[i * PACKET_SIZE..(i + 1) * PACKET_SIZE].try_into()?;

        // Ok(PacketView::new(packet_slice))

        // let slice = self.buffer[i * PACKET_SIZE..(i + 1) * PACKET_SIZE];

        let ptr = self.buffer.as_ptr();
        let range = self.buffer.as_ptr_range();
        // range.
        todo!()
    }
}
