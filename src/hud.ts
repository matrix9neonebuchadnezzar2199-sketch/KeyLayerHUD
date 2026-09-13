import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  fetchSnapshot,
  isMockMode,
  setHudEditMode,
  setHudVisual,
  setMockLayer,
} from "./api";
import type { HudSnapshot, LayoutKey } from "./types";

const panel = document.getElementById("hud-panel") as HTMLDivElement;
const keyboard = document.getElementById("keyboard") as HTMLDivElement;
const layerTabs = document.getElementById("layer-tabs") as HTMLDivElement;
const connPill = document.getElementById("conn-pill") as HTMLSpanElement;
const layerBadge = document.getElementById("layer-badge") as HTMLSpanElement;
const pinInd = document.getElementById("pin-ind") as HTMLSpanElement;
const opacityInput = document.getElementById("opacity") as HTMLInputElement;
const scaleInput = document.getElementById("scale") as HTMLInputElement;

let snapshot: HudSnapshot | null = null;
let pinned = false;

function sourceLabel(source: string): string {
  switch (source) {
    case "mock":
      return "Mock";
    case "via_dump":
      return "VIA";
    case "layer_report":
      return "Live";
    default:
      return "Off";
  }
}

function renderKeyboard(data: HudSnapshot) {
  const labelMap = new Map(data.labels.map((l) => [l.key_id, l]));
  const leftKeys = data.keys.filter((k) => k.half === "left");
  const rightKeys = data.keys.filter((k) => k.half === "right");

  function renderHalf(label: string, keys: LayoutKey[]) {
    const rows = new Map<number, LayoutKey[]>();
    keys.forEach((k) => {
      if (!rows.has(k.row)) rows.set(k.row, []);
      rows.get(k.row)!.push(k);
    });
    const rowHtml = [...rows.entries()]
      .sort(([a], [b]) => a - b)
      .map(([row, rowKeys]) => {
        const sorted = rowKeys.sort((a, b) => a.col - b.col);
        const isThumb = row === 3;
        return `<div class="row${isThumb ? " thumb-row" : ""}">${sorted
          .map((key) => {
            const lbl = labelMap.get(key.id);
            const text = lbl?.text ?? "·";
            const cls = ["key", key.thumb ? "thumb" : "", lbl?.momentary ? "momentary" : ""]
              .filter(Boolean)
              .join(" ");
            return `<div class="${cls}" data-key="${key.id}">${text}</div>`;
          })
          .join("")}</div>`;
      })
      .join("");
    return `<div class="half"><div class="half-label">${label}</div>${rowHtml}</div>`;
  }

  keyboard.innerHTML = renderHalf("Left", leftKeys) + renderHalf("Right", rightKeys);
}

function renderLayerTabs(layer: number) {
  const names = ["Base", "Nav", "Sym", "Fn"];
  layerTabs.innerHTML = names
    .map(
      (name, i) =>
        `<button type="button" class="layer-tab${i === layer ? " active" : ""}" data-layer="${i}">L${i} ${name}</button>`,
    )
    .join("");
  layerTabs.querySelectorAll(".layer-tab").forEach((btn) => {
    btn.addEventListener("click", async () => {
      const layerIdx = Number((btn as HTMLButtonElement).dataset.layer);
      await applySnapshot(await setMockLayer(layerIdx));
    });
  });
}

async function applySnapshot(data: HudSnapshot) {
  snapshot = data;
  connPill.textContent = sourceLabel(data.source);
  connPill.className = `status-pill ${data.source === "mock" ? "mock" : data.connected ? "live" : "off"}`;
  layerBadge.textContent = `Layer ${data.highest_layer} · ${data.layer_name}`;
  panel.style.opacity = String(data.hud_opacity);
  panel.style.transform = `scale(${data.hud_scale})`;
  pinned = data.hud_pinned;
  pinInd.textContent = pinned ? "📌 固定中" : "📌";
  pinInd.classList.toggle("on", pinned);
  panel.classList.toggle("edit-mode", data.hud_edit_mode);
  document.getElementById("btn-overlay")?.classList.toggle("active", !data.hud_edit_mode);
  document.getElementById("btn-edit")?.classList.toggle("active", data.hud_edit_mode);
  opacityInput.value = String(Math.round(data.hud_opacity * 100));
  scaleInput.value = String(Math.round(data.hud_scale * 100));
  renderKeyboard(data);
  renderLayerTabs(data.highest_layer);
}

async function setEditMode(edit: boolean) {
  if (!isMockMode()) {
    await setHudEditMode(edit);
  } else {
    panel.classList.toggle("edit-mode", edit);
    document.getElementById("btn-overlay")?.classList.toggle("active", !edit);
    document.getElementById("btn-edit")?.classList.toggle("active", edit);
  }
}

document.getElementById("btn-overlay")?.addEventListener("click", () => setEditMode(false));
document.getElementById("btn-edit")?.addEventListener("click", () => setEditMode(true));

opacityInput.addEventListener("input", async () => {
  const opacity = Number(opacityInput.value) / 100;
  const scale = Number(scaleInput.value) / 100;
  if (isMockMode()) {
    panel.style.opacity = String(opacity);
    return;
  }
  await applySnapshot(await setHudVisual(opacity, scale, pinned));
});

scaleInput.addEventListener("input", async () => {
  const opacity = Number(opacityInput.value) / 100;
  const scale = Number(scaleInput.value) / 100;
  panel.style.transform = `scale(${scale})`;
  if (isMockMode()) return;
  await applySnapshot(await setHudVisual(opacity, scale, pinned));
});

document.getElementById("btn-pin")?.addEventListener("click", async () => {
  pinned = !pinned;
  const opacity = Number(opacityInput.value) / 100;
  const scale = Number(scaleInput.value) / 100;
  if (isMockMode()) {
    pinInd.textContent = pinned ? "📌 固定中" : "📌";
    pinInd.classList.toggle("on", pinned);
    return;
  }
  await applySnapshot(await setHudVisual(opacity, scale, pinned));
});

if (!isMockMode()) {
  const win = getCurrentWindow();
  panel.addEventListener("mousedown", (e) => {
    if (!snapshot?.hud_edit_mode) return;
    if ((e.target as HTMLElement).closest("button, input")) return;
    win.startDragging();
  });

  listen<HudSnapshot>("hud-snapshot", (event) => {
    applySnapshot(event.payload);
  });
}

async function main() {
  await applySnapshot(await fetchSnapshot());

  if (isMockMode()) {
    let mockLayer = 0;
    setInterval(async () => {
      if (panel.classList.contains("edit-mode")) return;
      mockLayer = (mockLayer + 1) % 4;
      await applySnapshot(await setMockLayer(mockLayer));
    }, 3000);
  }
}

main();
