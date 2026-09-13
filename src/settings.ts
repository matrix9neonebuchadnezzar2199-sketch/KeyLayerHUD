import { open } from "@tauri-apps/plugin-dialog";
import {
  fetchSettings,
  fetchSnapshot,
  getResourceInfo,
  importKeymap,
  listHidDevices,
  quitApp,
  saveSettings,
  setConnectionMode,
  isMockMode,
} from "./api";
import type { AppSettings } from "./types";

function wireSidebar() {
  document.querySelectorAll(".side-link").forEach((link) => {
    link.addEventListener("click", () => {
      document.querySelectorAll(".side-link").forEach((l) => l.classList.remove("active"));
      link.classList.add("active");
      document.querySelectorAll(".panel").forEach((p) => p.classList.remove("active"));
      document.getElementById(`panel-${(link as HTMLButtonElement).dataset.panel}`)?.classList.add("active");
    });
  });

  const sidebar = document.getElementById("sidebar") as HTMLElement;
  const resizer = document.getElementById("sidebar-resizer") as HTMLElement;
  let dragging = false;
  resizer.addEventListener("mousedown", () => {
    dragging = true;
  });
  document.addEventListener("mousemove", (e) => {
    if (!dragging) return;
    sidebar.style.width = `${Math.min(420, Math.max(160, e.clientX - sidebar.getBoundingClientRect().left))}px`;
  });
  document.addEventListener("mouseup", () => {
    dragging = false;
  });
  document.getElementById("collapse-btn")?.addEventListener("click", () => {
    document.body.classList.add("sidebar-collapsed");
  });
  document.getElementById("sidebar-balloon")?.addEventListener("click", () => {
    document.body.classList.remove("sidebar-collapsed");
  });
}

function wireTheme() {
  const themeBtn = document.getElementById("theme-toggle") as HTMLButtonElement;
  themeBtn.addEventListener("click", () => {
    const light = document.documentElement.classList.toggle("light");
    themeBtn.textContent = light ? "☀" : "🌙";
  });
  let fontSize = 14;
  document.getElementById("font-plus")?.addEventListener("click", () => {
    fontSize = Math.min(20, fontSize + 1);
    document.documentElement.style.fontSize = `${fontSize}px`;
  });
  document.getElementById("font-minus")?.addEventListener("click", () => {
    fontSize = Math.max(11, fontSize - 1);
    document.documentElement.style.fontSize = `${fontSize}px`;
  });
}

function copyButtons(root: ParentNode) {
  root.querySelectorAll(".copy-btn").forEach((btn) => {
    btn.addEventListener("click", () => {
      const value = (btn as HTMLButtonElement).dataset.copy ?? "";
      navigator.clipboard.writeText(value).then(() => {
        btn.textContent = "✓";
        setTimeout(() => {
          btn.textContent = "📋";
        }, 1200);
      });
    });
  });
}

async function renderConnection(settings: AppSettings, info: Record<string, string>) {
  const panel = document.getElementById("panel-connection")!;
  panel.innerHTML = `
    <div class="card">
      <h2>接続</h2>
      <p class="sub">VIA Raw HID（Usage Page ${info.usage_page ?? "0xFF60"}）。VIA/VIAL GUI と同時接続はできません。</p>
      <div class="field">
        <label>接続モード</label>
        <select id="conn-mode">
          <option value="mock" ${settings.connection_mode === "mock" ? "selected" : ""}>Mock（開発用）</option>
          <option value="via" ${settings.connection_mode === "via" ? "selected" : ""}>VIA / VIAL（実機）</option>
          <option value="off" ${settings.connection_mode === "off" ? "selected" : ""}>切断</option>
        </select>
      </div>
      <div class="kv"><span>VID</span><code>0x${settings.vid.toString(16).padStart(4, "0")}</code>
        <button class="copy-btn" data-copy="0x${settings.vid.toString(16).padStart(4, "0")}">📋</button></div>
      <div class="kv"><span>PID</span><code>0x${settings.pid.toString(16).padStart(4, "0")}</code>
        <button class="copy-btn" data-copy="0x${settings.pid.toString(16).padStart(4, "0")}">📋</button></div>
      <div class="kv"><span>Usage Page</span><code>${info.usage_page ?? "0xFF60"}</code>
        <button class="copy-btn" data-copy="${info.usage_page ?? "0xFF60"}">📋</button></div>
      <div class="kv"><span>Layer Report</span><code>${info.layer_cmd ?? "0x42"}</code>
        <button class="copy-btn" data-copy="${info.layer_cmd ?? "0x42"}">📋</button></div>
      <p class="warn" id="device-list">デバイスをスキャン中…</p>
      <button class="btn" id="scan-devices" type="button">デバイスをスキャン</button>
    </div>`;
  copyButtons(panel);

  const select = panel.querySelector("#conn-mode") as HTMLSelectElement;
  select.addEventListener("change", async () => {
    await setConnectionMode(select.value);
  });

  const listEl = panel.querySelector("#device-list") as HTMLParagraphElement;
  const renderDevices = async () => {
    const devices = await listHidDevices();
    if (devices.length === 0) {
      listEl.textContent = "VIA Raw HID デバイスは見つかりませんでした（Mock モードで継続可能）。";
      return;
    }
    listEl.innerHTML = devices
      .map(([path, vid, pid]) => `<span class="mono">${path} — VID 0x${vid.toString(16)} PID 0x${pid.toString(16)}</span>`)
      .join("<br>");
    listEl.className = "ok";
  };
  panel.querySelector("#scan-devices")?.addEventListener("click", renderDevices);
  if (!isMockMode()) await renderDevices();
  else listEl.textContent = "ブラウザ Mock モード（?mock=1）";
}

function renderLayout(snapshotLayout: string) {
  const panel = document.getElementById("panel-layout")!;
  panel.innerHTML = `
    <div class="card">
      <h2>レイアウトプロファイル</h2>
      <p class="sub">物理キー配置の正本。キーマップとは別管理です。</p>
      <div class="field">
        <label>プロファイル</label>
        <select disabled>
          <option selected>${snapshotLayout} — サンプル</option>
        </select>
      </div>
      <p class="muted">Phase B: Lily58 / Keyball 等は JSON プロファイル追加のみで対応予定。</p>
    </div>`;
}

function renderKeymap(settings: AppSettings) {
  const panel = document.getElementById("panel-keymap")!;
  panel.innerHTML = `
    <div class="card">
      <h2>キーマップ</h2>
      <p class="sub">優先順位: ライブ VIA dump → インポート JSON → 同梱サンプル</p>
      <p>現在使用中: <code>${settings.keymap_path}</code></p>
      <button class="btn" id="import-keymap" type="button">VIA / JSON をインポート</button>
    </div>`;
  panel.querySelector("#import-keymap")?.addEventListener("click", async () => {
    if (isMockMode()) return;
    const selected = await open({
      multiple: false,
      filters: [{ name: "Keymap JSON", extensions: ["json"] }],
    });
    if (typeof selected === "string") {
      await importKeymap(selected);
      alert("キーマップを読み込みました");
    }
  });
}

function renderHud(settings: AppSettings) {
  const panel = document.getElementById("panel-hud")!;
  panel.innerHTML = `
    <div class="card">
      <h2>HUD 表示</h2>
      <div class="field"><label>グローバルホットキー</label><input value="${settings.hotkey}" readonly /></div>
      <div class="field"><label>Mock 自動切替</label>
        <select id="mock-cycle">
          <option value="1" ${settings.mock_auto_cycle ? "selected" : ""}>有効（${settings.mock_cycle_secs}s）</option>
          <option value="0" ${!settings.mock_auto_cycle ? "selected" : ""}>手動のみ</option>
        </select>
      </div>
      <button class="btn-save" id="save-hud" type="button">設定を保存</button>
    </div>`;
  panel.querySelector("#save-hud")?.addEventListener("click", async () => {
    const cycle = (panel.querySelector("#mock-cycle") as HTMLSelectElement).value === "1";
    const next = { ...settings, mock_auto_cycle: cycle };
    await saveSettings(next);
    alert("保存しました");
  });
}

function renderExit() {
  const panel = document.getElementById("panel-exit")!;
  panel.innerHTML = `
    <div class="card">
      <h2>終了</h2>
      <p class="sub">HUD を閉じてもトレイ常駐します。完全終了はトレイまたはここから行います。</p>
      <button class="btn-stop" id="quit" type="button">KeyLayerHUD を終了</button>
    </div>`;
  panel.querySelector("#quit")?.addEventListener("click", () => quitApp());
}

async function boot() {
  wireSidebar();
  wireTheme();
  const settings = await fetchSettings();
  const snapshot = await fetchSnapshot();
  const info = await getResourceInfo();
  await renderConnection(settings, info);
  renderLayout(snapshot.layout_name);
  renderKeymap(settings);
  renderHud(settings);
  renderExit();
}

boot();
