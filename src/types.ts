export type DataSource = "mock" | "via_dump" | "layer_report" | "disconnected";

export interface KeyLabel {
  key_id: string;
  text: string;
  code: number;
  momentary: boolean;
}

export interface LayoutKey {
  id: string;
  half: string;
  row: number;
  col: number;
  x: number;
  y: number;
  thumb?: boolean;
}

export interface HudSnapshot {
  source: DataSource;
  connected: boolean;
  highest_layer: number;
  layer_mask: number;
  layer_name: string;
  layout_id: string;
  layout_name: string;
  labels: KeyLabel[];
  keys: LayoutKey[];
  hud_opacity: number;
  hud_scale: number;
  hud_pinned: boolean;
  hud_edit_mode: boolean;
}

export interface AppSettings {
  connection_mode: string;
  layout_id: string;
  keymap_path: string;
  hud_opacity: number;
  hud_scale: number;
  hud_pinned: boolean;
  hud_edit_mode: boolean;
  mock_auto_cycle: boolean;
  mock_cycle_secs: number;
  hotkey: string;
  vid: number;
  pid: number;
}
