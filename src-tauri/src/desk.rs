//! Tauri adapter over [`desk_core`].
//!
//! All the BLE work lives in the `desk-core` crate (shared with `desk-cli`).
//! This file only bridges it to Tauri:
//!
//! * [`TauriReporter`] turns the controller's status/height callbacks into
//!   `desk-status` (String) and `desk-height` ({ raw, cm }) window events.
//! * Config (`last_address`, for auto-reconnect) is persisted in the app config
//!   dir as `desk_config.json`.
//! * The `#[tauri::command]` functions are the frontend's entry points.

use std::sync::Arc;

use desk_core::{DeskController, DeskInfo, DeskReporter};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

// ---------------------------------------------------------------------------
// Reporter — controller callbacks -> Tauri window events
// ---------------------------------------------------------------------------

#[derive(Clone, Serialize)]
struct HeightEvent {
    raw: i32,
    cm: f64,
}

struct TauriReporter {
    app: AppHandle,
}

impl DeskReporter for TauriReporter {
    fn status(&self, msg: &str) {
        let _ = self.app.emit("desk-status", msg.to_string());
    }
    fn height(&self, raw: i32, cm: f64) {
        let _ = self.app.emit("desk-height", HeightEvent { raw, cm });
    }
}

/// Build the shared controller, wired to emit Tauri events. Call from `setup`
/// and `app.manage(...)` the result.
pub fn build_controller(app: AppHandle) -> Arc<DeskController> {
    Arc::new(DeskController::new(Arc::new(TauriReporter { app })))
}

// ---------------------------------------------------------------------------
// Config — persisted as desk_config.json in the app config dir
// ---------------------------------------------------------------------------

#[derive(Default, Serialize, Deserialize)]
struct DeskConfig {
    #[serde(default)]
    last_address: Option<String>,
}

fn config_path(app: &AppHandle) -> Option<std::path::PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|d| d.join("desk_config.json"))
}

fn load_config(app: &AppHandle) -> DeskConfig {
    config_path(app)
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_config(app: &AppHandle, cfg: &DeskConfig) {
    if let Some(path) = config_path(app) {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(s) = serde_json::to_string_pretty(cfg) {
            let _ = std::fs::write(path, s);
        }
    }
}

fn remember_address(app: &AppHandle, address: Option<String>) {
    let mut cfg = load_config(app);
    cfg.last_address = address;
    save_config(app, &cfg);
}

// ---------------------------------------------------------------------------
// Tauri commands — the frontend's entry points
// ---------------------------------------------------------------------------

type Ctrl<'a> = State<'a, Arc<DeskController>>;

#[tauri::command]
pub async fn desk_scan(state: Ctrl<'_>) -> Result<Vec<DeskInfo>, String> {
    state.scan_desks().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn desk_connect(address: String, app: AppHandle, state: Ctrl<'_>) -> Result<bool, String> {
    let ok = state.connect(&address).await;
    if ok {
        remember_address(&app, Some(address)); // auto-reconnect on next launch
    }
    Ok(ok)
}

#[tauri::command]
pub async fn desk_connect_saved(app: AppHandle, state: Ctrl<'_>) -> Result<bool, String> {
    match load_config(&app).last_address {
        Some(addr) => Ok(state.connect(&addr).await),
        None => Ok(false),
    }
}

#[tauri::command]
pub async fn desk_disconnect(app: AppHandle, state: Ctrl<'_>) -> Result<(), String> {
    state.disconnect().await;
    remember_address(&app, None); // explicit disconnect clears auto-reconnect
    Ok(())
}

#[tauri::command]
pub async fn desk_move_start(direction: String, state: Ctrl<'_>) -> Result<(), String> {
    state.inner().clone().start_hold(&direction).await;
    Ok(())
}

#[tauri::command]
pub async fn desk_stop(state: Ctrl<'_>) -> Result<(), String> {
    state.inner().clone().stop_busy().await;
    Ok(())
}

#[tauri::command]
pub async fn desk_save_preset(slot: u8, state: Ctrl<'_>) -> Result<(), String> {
    state.save_preset(slot).await;
    Ok(())
}

#[tauri::command]
pub async fn desk_apply_preset(slot: u8, state: Ctrl<'_>) -> Result<(), String> {
    state.inner().clone().apply_preset_cmd(slot).await;
    Ok(())
}

#[tauri::command]
pub async fn desk_height(state: Ctrl<'_>) -> Result<Option<f64>, String> {
    Ok(state.current_cm())
}

#[tauri::command]
pub async fn desk_is_connected(state: Ctrl<'_>) -> Result<bool, String> {
    Ok(state.is_connected().await)
}
