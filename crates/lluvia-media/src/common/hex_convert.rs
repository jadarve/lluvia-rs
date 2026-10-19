const HEX_CHARS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

pub fn to_hex_string(data: &[u8]) -> String {
    let mut s = String::with_capacity(2 * data.len());

    for &byte in data {
        // TODO: I could still remove the look up table and use char offsets in the ASCII table
        s.push(HEX_CHARS[(byte >> 4) as usize]);
        s.push(HEX_CHARS[(byte & 0xF) as usize]);
    }
    s
}
