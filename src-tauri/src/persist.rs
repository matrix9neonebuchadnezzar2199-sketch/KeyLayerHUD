use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub connection_mode: String,
    pub layout_id: String,
    pub keymap_path: String,
    pub hud_opacity: f32,
    pub hud_scale: f32,
    pub hud_pinned: bool,
    pub hud_edit_mode: bool,
    pub mock_auto_cycle: bool,
    pub mock_cycle_secs: u64,
    pub hotkey: String,
    pub vid: u16,
    pub pid: u16,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            connection_mode: "mock".to_string(),
            layout_id: "corne-42".to_string(),
            keymap_path: "keymaps/sample-via.json".to_string(),
            hud_opacity: 0.88,
            hud_scale: 1.0,
            hud_pinned: false,
            hud_edit_mode: false,
            mock_auto_cycle: true,
            mock_cycle_secs: 3,
            hotkey: "Ctrl+Shift+K".to_string(),
            vid: 0,
            pid: 0,
        }
    }
}

pub fn settings_path() -> PathBuf {
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("KeyLayerHUD").join("settings.toml")
}

pub fn load_settings() -> AppSettings {
    let path = settings_path();
    if !path.exists() {
        return AppSettings::default();
    }
    let raw = fs::read_to_string(&path).unwrap_or_default();
    toml::from_str(&raw).unwrap_or_default()
}

pub fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("mkdir failed: {e}"))?;
    }
    let raw = toml::to_string_pretty(settings).map_err(|e| format!("toml encode: {e}"))?;
    fs::write(&path, raw).map_err(|e| format!("write failed: {e}"))?;
    Ok(())
}
