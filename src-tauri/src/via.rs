pub const VIA_USAGE_PAGE: u16 = 0xFF60;
pub const VIA_USAGE: u16 = 0x61;

pub const CMD_GET_PROTOCOL_VERSION: u8 = 0x01;
pub const CMD_DYNAMIC_KEYMAP_GET_LAYER_COUNT: u8 = 0x11;
pub const CMD_DYNAMIC_KEYMAP_GET_BUFFER: u8 = 0x12;

pub const PROTOCOL_BETA: u16 = 8;

pub fn shift_to_16bit(hi: u8, lo: u8) -> u16 {
    ((hi as u16) << 8) | lo as u16
}

pub fn shift_from_16bit(value: u16) -> (u8, u8) {
    ((value >> 8) as u8, (value & 0xFF) as u8)
}

pub fn build_hid_command(command: u8, payload: &[u8]) -> Vec<u8> {
    let mut padded = vec![0u8; 33];
    padded[0] = 0x00;
    padded[1] = command;
    for (idx, byte) in payload.iter().enumerate() {
        padded[2 + idx] = *byte;
    }
    padded
}

pub fn parse_keymap_buffer_chunk(response: &[u8], size: u8) -> Vec<u8> {
    let take = size as usize;
    if response.len() < 4 + take {
        return Vec::new();
    }
    response[4..4 + take].to_vec()
}

pub fn buffer_chunk_to_keycodes(bytes: &[u8]) -> Vec<u16> {
    let mut codes = Vec::new();
    for chunk in bytes.chunks(2) {
        if chunk.len() == 2 {
            codes.push(shift_to_16bit(chunk[0], chunk[1]));
        }
    }
    codes
}

/// レイヤー全体のキーマップをフラット配列から行列へ復元する。
pub fn flat_to_matrix(flat: &[u16], rows: u8, cols: u8) -> Vec<Vec<u16>> {
    let cols = cols as usize;
    let mut matrix = Vec::new();
    for row in 0..rows as usize {
        let start = row * cols;
        let end = start + cols;
        matrix.push(flat.get(start..end).unwrap_or(&[]).to_vec());
    }
    matrix
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_16bit() {
        let (hi, lo) = shift_from_16bit(0x1234);
        assert_eq!(shift_to_16bit(hi, lo), 0x1234);
    }

    #[test]
    fn builds_via_command() {
        let cmd = build_hid_command(CMD_DYNAMIC_KEYMAP_GET_LAYER_COUNT, &[]);
        assert_eq!(cmd[1], CMD_DYNAMIC_KEYMAP_GET_LAYER_COUNT);
    }
}
