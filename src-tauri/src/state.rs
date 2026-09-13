use crate::keymap::{
    is_momentary, keycode_label, resolve_keycode, KeyLabel, KeymapFile, highest_layer,
};
use crate::layout::{LayoutKey, LayoutProfile};
use crate::persist::AppSettings;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DataSource {
    Mock,
    ViaDump,
    LayerReport,
    Disconnected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HudSnapshot {
    pub source: DataSource,
    pub connected: bool,
    pub highest_layer: u8,
    pub layer_mask: u32,
    pub layer_name: String,
    pub layout_id: String,
    pub layout_name: String,
    pub labels: Vec<KeyLabel>,
    pub keys: Vec<LayoutKey>,
    pub hud_opacity: f32,
    pub hud_scale: f32,
    pub hud_pinned: bool,
    pub hud_edit_mode: bool,
}

pub struct RuntimeState {
    pub settings: AppSettings,
    pub layout: LayoutProfile,
    pub keymap: KeymapFile,
    pub mock_layer: u8,
    pub mock_layer_mask: u32,
    pub live_layer: u8,
    pub live_layer_mask: u32,
    pub connected: bool,
    pub source: DataSource,
    pub last_error: Option<String>,
}

impl RuntimeState {
    pub fn build_snapshot(&self) -> HudSnapshot {
        let active_layer = match self.source {
            DataSource::Mock => self.mock_layer,
            DataSource::LayerReport | DataSource::ViaDump => self.live_layer,
            DataSource::Disconnected => 0,
        };
        let layer_mask = match self.source {
            DataSource::Mock => self.mock_layer_mask,
            DataSource::LayerReport | DataSource::ViaDump => self.live_layer_mask,
            DataSource::Disconnected => 1,
        };
        let layer_name = self
            .keymap
            .layer_names
            .get(active_layer as usize)
            .cloned()
            .unwrap_or_else(|| format!("Layer {}", active_layer));

        let labels = self
            .layout
            .keys
            .iter()
            .map(|key| {
                let code = resolve_keycode(
                    &self.keymap.layers,
                    key.row,
                    key.col,
                    active_layer,
                );
                KeyLabel {
                    key_id: key.id.clone(),
                    text: keycode_label(code),
                    code,
                    momentary: is_momentary(code),
                }
            })
            .collect();

        HudSnapshot {
            source: self.source.clone(),
            connected: self.connected,
            highest_layer: active_layer,
            layer_mask,
            layer_name,
            layout_id: self.layout.id.clone(),
            layout_name: self.layout.name.clone(),
            labels,
            keys: self.layout.keys.clone(),
            hud_opacity: self.settings.hud_opacity,
            hud_scale: self.settings.hud_scale,
            hud_pinned: self.settings.hud_pinned,
            hud_edit_mode: self.settings.hud_edit_mode,
        }
    }

    pub fn set_mock_layer(&mut self, layer: u8) {
        self.mock_layer = layer.min(self.keymap.layer_count.saturating_sub(1));
        self.mock_layer_mask = 1u32 << self.mock_layer;
    }

    pub fn advance_mock_layer(&mut self) {
        let next = (self.mock_layer + 1) % self.keymap.layer_count.max(1);
        self.set_mock_layer(next);
    }

    pub fn apply_layer_report(&mut self, highest: u8, mask: u32) {
        self.live_layer = highest;
        self.live_layer_mask = mask;
        if mask == 0 {
            self.live_layer_mask = 1;
        }
    }

    pub fn recompute_highest_from_mask(&mut self) {
        self.live_layer = highest_layer(self.live_layer_mask, self.keymap.layer_count);
    }
}
