pub const CMD_LAYER_QUERY: u8 = 0x42;
pub const CMD_LAYER_REPORT: u8 = 0x43;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayerReport {
    pub highest_layer: u8,
    pub layer_mask: u32,
    pub mods: u8,
}

/// Raw HID パケットからレイヤー報告を解析する。
pub fn parse_layer_report(data: &[u8]) -> Option<LayerReport> {
    if data.len() < 6 {
        return None;
    }
    if data[0] != CMD_LAYER_REPORT && data[0] != CMD_LAYER_QUERY {
        return None;
    }
    // 0x43 = push report, 0x42 response mirrors layout
    let highest = data[1];
    let layer_mask = u32::from_le_bytes([data[2], data[3], data[4], data[5]]);
    let mods = if data.len() > 6 { data[6] } else { 0 };
    Some(LayerReport {
        highest_layer: highest,
        layer_mask,
        mods,
    })
}

pub fn build_layer_query_packet() -> [u8; 32] {
    let mut buf = [0u8; 32];
    buf[0] = CMD_LAYER_QUERY;
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_layer_report() {
        let mut pkt = [0u8; 32];
        pkt[0] = CMD_LAYER_REPORT;
        pkt[1] = 2;
        pkt[2] = 0b00000101;
        let report = parse_layer_report(&pkt).expect("report");
        assert_eq!(report.highest_layer, 2);
        assert_eq!(report.layer_mask, 5);
    }
}
