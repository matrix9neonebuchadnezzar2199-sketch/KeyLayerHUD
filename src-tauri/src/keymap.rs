use serde::{Deserialize, Serialize};

pub const KC_NO: u16 = 0x0000;
pub const KC_TRNS: u16 = 0x0001;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeymapFile {
    pub layout_id: String,
    pub layer_count: u8,
    pub layer_names: Vec<String>,
    pub matrix: MatrixSize,
    pub layers: Vec<Vec<Vec<u16>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixSize {
    pub rows: u8,
    pub cols: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyLabel {
    pub key_id: String,
    pub text: String,
    pub code: u16,
    pub momentary: bool,
}

/// QMK の layer_state から最上位レイヤーを返す（簡易版）。
pub fn highest_layer(layer_mask: u32, layer_count: u8) -> u8 {
    let max = layer_count.min(32);
    for layer in (0..max).rev() {
        if layer_mask & (1 << layer) != 0 {
            return layer;
        }
    }
    0
}

/// 指定位置のキーコードを、上位レイヤーから下へ TRNS を辿って解決する。
pub fn resolve_keycode(layers: &[Vec<Vec<u16>>], row: u8, col: u8, up_to_layer: u8) -> u16 {
    let row = row as usize;
    let col = col as usize;
    for layer_idx in (0..=up_to_layer).rev() {
        if let Some(layer) = layers.get(layer_idx as usize) {
            if let Some(row_data) = layer.get(row) {
                if let Some(&code) = row_data.get(col) {
                    if code != KC_TRNS && code != KC_NO {
                        return code;
                    }
                }
            }
        }
    }
    KC_NO
}

pub fn keycode_label(code: u16) -> String {
    match code {
        KC_NO => "·".to_string(),
        KC_TRNS => "▽".to_string(),
        0x0004..=0x001D => {
            let idx = (code - 0x0004) as u8;
            if idx < 26 {
                return ((b'A' + idx) as char).to_string();
            }
            format!("0x{:04X}", code)
        }
        0x001E..=0x0026 => {
            let idx = (code - 0x001E) as u8;
            if idx < 9 {
                return ((b'1' + idx) as char).to_string();
            }
            format!("0x{:04X}", code)
        }
        0x0027 => "0".to_string(),
        0x0028 => "Enter".to_string(),
        0x0029 => "Esc".to_string(),
        0x002A => "Bspc".to_string(),
        0x002B => "Tab".to_string(),
        0x002C => "Spc".to_string(),
        0x002D => "-".to_string(),
        0x002E => "=".to_string(),
        0x002F => "[".to_string(),
        0x0030 => "]".to_string(),
        0x0031 => "\\".to_string(),
        0x0033 => ";".to_string(),
        0x0034 => "'".to_string(),
        0x0036 => ",".to_string(),
        0x0037 => ".".to_string(),
        0x0038 => "/".to_string(),
        0x0052 => "↑".to_string(),
        0x0051 => "↓".to_string(),
        0x0050 => "←".to_string(),
        0x004F => "→".to_string(),
        0x004C => "Del".to_string(),
        0x004D => "End".to_string(),
        0x004A => "Home".to_string(),
        0x004B => "PgUp".to_string(),
        0x004E => "PgDn".to_string(),
        0x00E1 => "LShift".to_string(),
        0x00E2 => "LCtrl".to_string(),
        0x00E3 => "LAlt".to_string(),
        0x00E4 => "LGui".to_string(),
        0x00E5 => "RShift".to_string(),
        0x00E6 => "RCtrl".to_string(),
        0x00E7 => "RAlt".to_string(),
        0x00E8 => "RGui".to_string(),
        0x00F0..=0x00FF => format!("MO{}", code - 0x00F0),
        0x00C0..=0x00DF => format!("F{}", code - 0x00C0 + 13),
        _ => format!("0x{:04X}", code),
    }
}

pub fn is_momentary(code: u16) -> bool {
    (0x00F0..=0x00FF).contains(&code) || (0x5100..=0x51FF).contains(&code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_trns_from_lower_layer() {
        let layers = vec![
            vec![vec![KC_TRNS, 0x0004]], // layer 0: A at (0,1)
            vec![vec![0x0005, KC_TRNS]], // layer 1: B at (0,0)
        ];
        assert_eq!(resolve_keycode(&layers, 0, 0, 1), 0x0005);
        assert_eq!(resolve_keycode(&layers, 0, 1, 1), 0x0004);
    }

    #[test]
    fn highest_layer_picks_topmost() {
        assert_eq!(highest_layer(0b1010, 4), 3);
        assert_eq!(highest_layer(0b0001, 4), 0);
    }
}
