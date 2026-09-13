mod hid;
mod keymap;
mod layer_report;
mod layout;
mod persist;
mod state;
mod via;

use parking_lot::Mutex;
use persist::{load_settings, save_settings, AppSettings};
use state::{DataSource, HudSnapshot, RuntimeState};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State, WebviewWindow,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

pub struct AppState {
    pub runtime: Mutex<RuntimeState>,
    pub hid_path: Mutex<Option<String>>,
}

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("project root")
        .to_path_buf()
}

fn load_keymap_file(rel: &str) -> Result<keymap::KeymapFile, String> {
    let path = project_root().join(rel);
    let raw = std::fs::read_to_string(&path).map_err(|e| format!("keymap read: {e}"))?;
    serde_json::from_str(&raw).map_err(|e| format!("keymap parse: {e}"))
}

fn load_layout_file(id: &str) -> Result<layout::LayoutProfile, String> {
    let path = project_root().join("layouts").join(format!("{id}.json"));
    layout::load_layout(&path)
}

fn init_runtime() -> Result<RuntimeState, String> {
    let settings = load_settings();
    let is_mock = settings.connection_mode == "mock";
    let layout = load_layout_file(&settings.layout_id)?;
    let keymap = load_keymap_file(&settings.keymap_path)?;
    let mut runtime = RuntimeState {
        settings,
        layout,
        keymap,
        mock_layer: 0,
        mock_layer_mask: 1,
        live_layer: 0,
        live_layer_mask: 1,
        connected: is_mock,
        source: if is_mock {
            DataSource::Mock
        } else {
            DataSource::Disconnected
        },
        last_error: None,
    };
    runtime.set_mock_layer(0);
    Ok(runtime)
}

fn emit_snapshot(app: &AppHandle, snapshot: &HudSnapshot) {
    let _ = app.emit("hud-snapshot", snapshot);
}

fn apply_hud_window_mode(window: &WebviewWindow, edit_mode: bool) -> Result<(), String> {
    window
        .set_ignore_cursor_events(!edit_mode)
        .map_err(|e| format!("ignore cursor: {e}"))?;
    Ok(())
}

#[tauri::command]
fn get_snapshot(state: State<'_, Arc<AppState>>) -> HudSnapshot {
    state.runtime.lock().build_snapshot()
}

#[tauri::command]
fn get_settings(state: State<'_, Arc<AppState>>) -> AppSettings {
    state.runtime.lock().settings.clone()
}

#[tauri::command]
fn save_app_settings(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    settings: AppSettings,
) -> Result<(), String> {
    {
        let mut rt = state.runtime.lock();
        rt.settings = settings.clone();
    }
    save_settings(&settings)?;
    let snapshot = state.runtime.lock().build_snapshot();
    emit_snapshot(&app, &snapshot);
    Ok(())
}

#[tauri::command]
fn set_connection_mode(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    mode: String,
) -> Result<HudSnapshot, String> {
    let snapshot = {
        let mut rt = state.runtime.lock();
        rt.settings.connection_mode = mode.clone();
        rt.source = match mode.as_str() {
            "mock" => {
                rt.connected = true;
                DataSource::Mock
            }
            "via" => {
                rt.connected = false;
                DataSource::Disconnected
            }
            _ => {
                rt.connected = false;
                DataSource::Disconnected
            }
        };
        rt.last_error = None;
        save_settings(&rt.settings)?;
        rt.build_snapshot()
    };
    emit_snapshot(&app, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
fn set_mock_layer(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    layer: u8,
) -> Result<HudSnapshot, String> {
    let snapshot = {
        let mut rt = state.runtime.lock();
        rt.set_mock_layer(layer);
        rt.build_snapshot()
    };
    emit_snapshot(&app, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
fn set_hud_edit_mode(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    edit_mode: bool,
) -> Result<(), String> {
    {
        let mut rt = state.runtime.lock();
        rt.settings.hud_edit_mode = edit_mode;
        save_settings(&rt.settings)?;
    }
    if let Some(window) = app.get_webview_window("hud") {
        apply_hud_window_mode(&window, edit_mode)?;
    }
    let snapshot = state.runtime.lock().build_snapshot();
    emit_snapshot(&app, &snapshot);
    Ok(())
}

#[tauri::command]
fn set_hud_visual(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    opacity: f32,
    scale: f32,
    pinned: bool,
) -> Result<HudSnapshot, String> {
    let snapshot = {
        let mut rt = state.runtime.lock();
        rt.settings.hud_opacity = opacity.clamp(0.3, 1.0);
        rt.settings.hud_scale = scale.clamp(0.7, 1.3);
        rt.settings.hud_pinned = pinned;
        save_settings(&rt.settings)?;
        rt.build_snapshot()
    };
    emit_snapshot(&app, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
fn toggle_overlay_edit(app: AppHandle, state: State<'_, Arc<AppState>>) -> Result<bool, String> {
    let edit = {
        let mut rt = state.runtime.lock();
        rt.settings.hud_edit_mode = !rt.settings.hud_edit_mode;
        save_settings(&rt.settings)?;
        rt.settings.hud_edit_mode
    };
    if let Some(window) = app.get_webview_window("hud") {
        apply_hud_window_mode(&window, edit)?;
    }
    let snapshot = state.runtime.lock().build_snapshot();
    emit_snapshot(&app, &snapshot);
    Ok(edit)
}

#[tauri::command]
fn show_settings(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn hide_hud(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("hud") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn show_hud(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("hud") {
        window.show().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn quit_app(app: AppHandle) -> Result<(), String> {
    app.exit(0);
    Ok(())
}

#[tauri::command]
fn list_hid_devices() -> Result<Vec<(String, u16, u16)>, String> {
    let api = hid::init_hid()?;
    Ok(hid::ViaDevice::enumerate(&api))
}

#[tauri::command]
fn import_keymap_json(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    path: String,
) -> Result<HudSnapshot, String> {
    let raw = std::fs::read_to_string(&path).map_err(|e| format!("read keymap: {e}"))?;
    let keymap: keymap::KeymapFile =
        serde_json::from_str(&raw).map_err(|e| format!("parse keymap: {e}"))?;
    let snapshot = {
        let mut rt = state.runtime.lock();
        rt.keymap = keymap;
        rt.settings.keymap_path = path;
        save_settings(&rt.settings)?;
        rt.build_snapshot()
    };
    emit_snapshot(&app, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
fn get_resource_info() -> serde_json::Value {
    serde_json::json!({
        "project_root": project_root(),
        "usage_page": format!("0x{:04X}", via::VIA_USAGE_PAGE),
        "layer_cmd": format!("0x{:02X}", layer_report::CMD_LAYER_QUERY),
    })
}

fn register_shortcut(app: &AppHandle) -> Result<(), String> {
    let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyK);
    app.global_shortcut()
        .on_shortcut(shortcut, |app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                let state = app.state::<Arc<AppState>>();
                let _ = toggle_overlay_edit(app.clone(), state);
            }
        })
        .map_err(|e| format!("shortcut register: {e}"))?;
    Ok(())
}

fn build_tray(app: &AppHandle) -> Result<(), String> {
    let show_hud_item = MenuItem::with_id(app, "show_hud", "HUD を表示", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let settings_item = MenuItem::with_id(app, "settings", "設定…", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let toggle_mode_item =
        MenuItem::with_id(app, "toggle_mode", "Overlay / Edit 切替", true, None::<&str>)
            .map_err(|e| e.to_string())?;
    let quit_item = MenuItem::with_id(app, "quit", "終了", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let menu = Menu::with_items(app, &[&show_hud_item, &settings_item, &toggle_mode_item, &quit_item])
        .map_err(|e| e.to_string())?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show_hud" => {
                let _ = show_hud(app.clone());
            }
            "settings" => {
                let _ = show_settings(app.clone());
            }
            "toggle_mode" => {
                let state = app.state::<Arc<AppState>>();
                let _ = toggle_overlay_edit(app.clone(), state);
            }
            "quit" => {
                let _ = quit_app(app.clone());
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                let _ = show_settings(app.clone());
            }
        })
        .build(app)
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn start_background_poll(app: AppHandle, state: Arc<AppState>) {
    thread::spawn(move || {
        let mut tick = 0u64;
        loop {
            thread::sleep(Duration::from_millis(500));
            tick += 1;
            let mut maybe_snapshot = None;
            {
                let mut rt = state.runtime.lock();
                if rt.settings.connection_mode == "mock" && rt.settings.mock_auto_cycle && tick % 6 == 0 {
                    rt.advance_mock_layer();
                    maybe_snapshot = Some(rt.build_snapshot());
                }
            }
            if let Some(snapshot) = maybe_snapshot {
                emit_snapshot(&app, &snapshot);
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let runtime = init_runtime().expect("failed to init runtime");
    let app_state = Arc::new(AppState {
        runtime: Mutex::new(runtime),
        hid_path: Mutex::new(None),
    });

    let state_for_setup = app_state.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_snapshot,
            get_settings,
            save_app_settings,
            set_connection_mode,
            set_mock_layer,
            set_hud_edit_mode,
            set_hud_visual,
            toggle_overlay_edit,
            show_settings,
            hide_hud,
            show_hud,
            quit_app,
            list_hid_devices,
            import_keymap_json,
            get_resource_info
        ])
        .setup(move |app| {
            if let Some(hud) = app.get_webview_window("hud") {
                let edit = state_for_setup.runtime.lock().settings.hud_edit_mode;
                apply_hud_window_mode(&hud, edit)?;
            }
            build_tray(app.handle())?;
            register_shortcut(app.handle())?;
            start_background_poll(app.handle().clone(), state_for_setup.clone());
            let snapshot = state_for_setup.runtime.lock().build_snapshot();
            emit_snapshot(app.handle(), &snapshot);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
