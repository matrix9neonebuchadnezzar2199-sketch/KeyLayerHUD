use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutProfile {
    pub id: String,
    pub name: String,
    pub matrix: MatrixSize,
    pub keys: Vec<LayoutKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixSize {
    pub rows: u8,
    pub cols: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutKey {
    pub id: String,
    pub half: String,
    pub row: u8,
    pub col: u8,
    pub x: i32,
    pub y: i32,
    #[serde(default)]
    pub thumb: bool,
}

pub fn load_layout(path: &Path) -> Result<LayoutProfile, String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("layout read failed: {e}"))?;
    serde_json::from_str(&raw).map_err(|e| format!("layout parse failed: {e}"))
}
