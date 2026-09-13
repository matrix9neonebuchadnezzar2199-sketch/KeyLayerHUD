import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, HudSnapshot } from "./types";

const MOCK_MODE = new URLSearchParams(window.location.search).get("mock") === "1";

const MOCK_SNAPSHOT: HudSnapshot = {
  source: "mock",
  connected: true,
  highest_layer: 0,
  layer_mask: 1,
  layer_name: "Base",
  layout_id: "corne-42",
  layout_name: "Corne 42",
  hud_opacity: 0.88,
  hud_scale: 1,
  hud_pinned: false,
  hud_edit_mode: false,
  keys: [],
  labels: [],
};

let mockLayer = 0;

function buildMockKeys(): HudSnapshot {
  const layers = [
    ["Tab", "Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P", "Bspc"],
    ["Ctrl", "A", "S", "D", "F", "G", "H", "J", "K", "L", ";", "Ent"],
    ["Shift", "Z", "X", "C", "V", "B", "N", "M", ",", ".", "/", "Shift"],
    ["", "Esc", "LGui", "LAlt", "", "", "", "", "RAlt", "MO1", "Spc", "Ent"],
  ];
  const labels = layers.flatMap((row, ri) =>
    row.map((text, ci) => ({
      key_id: `K${ri}${ci}`,
      text: text || "·",
      code: 0,
      momentary: text.startsWith("MO"),
    })),
  );
  const keys: HudSnapshot["keys"] = [];
  let idx = 0;
  for (const half of ["left", "right"]) {
    for (let row = 0; row < 4; row++) {
      for (let col = 0; col < 6; col++) {
        keys.push({
          id: `${half}-${row}-${col}`,
          half,
          row,
          col: half === "left" ? col : col + 6,
          x: col * 46,
          y: row * 46,
          thumb: row === 3,
        });
        idx++;
      }
    }
  }
  return {
    ...MOCK_SNAPSHOT,
    highest_layer: mockLayer,
    layer_mask: 1 << mockLayer,
    layer_name: ["Base", "Nav", "Sym", "Fn"][mockLayer] ?? `Layer ${mockLayer}`,
    keys,
    labels: labels.slice(0, keys.length),
  };
}

export function isMockMode(): boolean {
  return MOCK_MODE || !("__TAURI_INTERNALS__" in window);
}

export async function fetchSnapshot(): Promise<HudSnapshot> {
  if (isMockMode()) return buildMockKeys();
  return invoke<HudSnapshot>("get_snapshot");
}

export async function fetchSettings(): Promise<AppSettings> {
  if (isMockMode()) {
    return {
      connection_mode: "mock",
      layout_id: "corne-42",
      keymap_path: "keymaps/sample-via.json",
      hud_opacity: 0.88,
      hud_scale: 1,
      hud_pinned: false,
      hud_edit_mode: false,
      mock_auto_cycle: true,
      mock_cycle_secs: 3,
      hotkey: "Ctrl+Shift+K",
      vid: 0,
      pid: 0,
    };
  }
  return invoke<AppSettings>("get_settings");
}

export async function setHudEditMode(edit: boolean): Promise<void> {
  if (isMockMode()) return;
  await invoke("set_hud_edit_mode", { editMode: edit });
}

export async function setHudVisual(opacity: number, scale: number, pinned: boolean): Promise<HudSnapshot> {
  if (isMockMode()) return { ...buildMockKeys(), hud_opacity: opacity, hud_scale: scale, hud_pinned: pinned };
  return invoke("set_hud_visual", { opacity, scale, pinned });
}

export async function setMockLayer(layer: number): Promise<HudSnapshot> {
  if (isMockMode()) {
    mockLayer = layer;
    return buildMockKeys();
  }
  return invoke("set_mock_layer", { layer });
}

export async function setConnectionMode(mode: string): Promise<HudSnapshot> {
  if (isMockMode()) return buildMockKeys();
  return invoke("set_connection_mode", { mode });
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  if (isMockMode()) return;
  await invoke("save_app_settings", { settings });
}

export async function importKeymap(path: string): Promise<HudSnapshot> {
  if (isMockMode()) return buildMockKeys();
  return invoke("import_keymap_json", { path });
}

export async function listHidDevices(): Promise<[string, number, number][]> {
  if (isMockMode()) return [];
  return invoke("list_hid_devices");
}

export async function quitApp(): Promise<void> {
  if (isMockMode()) return;
  await invoke("quit_app");
}

export async function getResourceInfo(): Promise<Record<string, string>> {
  if (isMockMode()) {
    return { usage_page: "0xFF60", layer_cmd: "0x42" };
  }
  return invoke("get_resource_info");
}
