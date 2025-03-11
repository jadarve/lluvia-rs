use lluvia_media::containers::ts;

fn main() -> Result<(), String> {
    let data = {
        let bytes_vec = std::fs::read("local/sample.ts").map_err(|e| e.to_string())?;
        bytes::Bytes::from_owner(bytes_vec)
    };

    print!("data len: {}", data.len());

    for i in 0..data.len() / ts::PACKET_SIZE {
        let packet_slice = data.slice(i * ts::PACKET_SIZE..(i + 1) * ts::PACKET_SIZE);
        let packet_view = ts::PacketView::new(packet_slice);
        assert!(packet_view.is_valid().is_ok());
        println!("{:?}", packet_view);
    }

    Ok(())
}
