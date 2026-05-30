//! Tauri adapter over [`desk_core`].
//!
//! All the BLE work lives in the `desk-core` crate (shared with `desk-cli`).
//! This file only bridges it to Tauri:
//!
//! * [`TauriReporter`] turns the controller's callbacks into window events:
//!   `desk-height` ({ cm }), `desk-connection` ({ state, name? }),
//!   `desk-motion` ({ moving }), and `desk-discovered` ({ name, address, rssi })
//!   during a streaming scan.
//! * Config (`last_address`, for auto-reconnect) is persisted in the app config
//!   dir as `desk_config.json`.
//! * The `#[tauri::command]` functions are the frontend's entry points.

use std::sync::Arc;

use desk_core::{
    arrive_tolerance_cm, cm_to_raw, BluetoothState, ConnectionState, DeskController, DeskInfo,
    DeskReporter, Direction,
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_opener::OpenerExt;

// ---------------------------------------------------------------------------
// Reporter: controller callbacks -> Tauri window events
// ---------------------------------------------------------------------------

#[derive(Clone, Serialize)]
struct HeightEvent {
    cm: f64,
}

#[derive(Clone, Serialize)]
struct ConnectionEvent {
    state: &'static str,
    name: Option<String>,
}

#[derive(Clone, Serialize)]
struct MotionEvent {
    moving: bool,
}

#[derive(Clone, Serialize)]
struct BluetoothEvent {
    state: &'static str,
}

/// Stable string form of the Bluetooth state, shared by the `desk-bluetooth`
/// event and the `bluetooth_state` command (mirrored by the frontend).
fn bt_state_str(state: BluetoothState) -> &'static str {
    match state {
        BluetoothState::Ready => "ready",
        BluetoothState::Off => "off",
    }
}

struct TauriReporter {
    app: AppHandle,
}

impl DeskReporter for TauriReporter {
    fn height(&self, _raw: i32, cm: f64) {
        let _ = self.app.emit("desk-height", HeightEvent { cm });
    }
    fn connection(&self, state: ConnectionState, name: Option<&str>) {
        let state = match state {
            ConnectionState::Disconnected => "disconnected",
            ConnectionState::Connecting => "connecting",
            ConnectionState::Connected => "connected",
        };
        let _ = self.app.emit(
            "desk-connection",
            ConnectionEvent {
                state,
                name: name.map(|s| s.to_string()),
            },
        );
    }
    fn motion(&self, moving: bool) {
        let _ = self.app.emit("desk-motion", MotionEvent { moving });
    }
    fn discovered(&self, desk: &DeskInfo) {
        let _ = self.app.emit("desk-discovered", desk.clone());
    }
    fn bluetooth(&self, state: BluetoothState) {
        let _ = self.app.emit(
            "desk-bluetooth",
            BluetoothEvent {
                state: bt_state_str(state),
            },
        );
    }
}

/// Build the shared controller, wired to emit Tauri events. Call from `setup`
/// and `app.manage(...)` the result.
pub fn build_controller(app: AppHandle) -> Arc<DeskController> {
    Arc::new(DeskController::new(Arc::new(TauriReporter { app })))
}

// ---------------------------------------------------------------------------
// Config: persisted as desk_config.json in the app config dir
// ---------------------------------------------------------------------------

#[derive(Default, Serialize, Deserialize)]
struct DeskConfig {
    #[serde(default)]
    last_address: Option<String>,
    /// The remembered desk's advertised name, shown on the "Connecting…" screen
    /// during auto-reconnect (before the live name is available).
    #[serde(default)]
    last_name: Option<String>,
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

/// Persist (or, with `None`, clear) the desk to auto-reconnect to on next launch.
fn remember_desk(app: &AppHandle, desk: Option<(String, String)>) {
    let mut cfg = load_config(app);
    let (address, name) = match desk {
        Some((address, name)) => (Some(address), Some(name)),
        None => (None, None),
    };
    cfg.last_address = address;
    cfg.last_name = name;
    save_config(app, &cfg);
}

// ---------------------------------------------------------------------------
// Boot: the single place that decides what the app does on launch. The frontend
// calls `desk_boot` once and renders the result, then follows the window events
// for the rest; it no longer orchestrates reconnect-vs-scan itself.
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum BootScreen {
    /// Already connected (e.g. the webview reloaded while the link stayed up).
    Connected,
    /// Reconnecting to the remembered desk; the connect runs in the background
    /// and reports its outcome via the `desk-connection` event.
    Connecting,
    /// Bluetooth is off: show the enable-Bluetooth overlay.
    BluetoothOff,
    /// No remembered desk: a discovery scan is running, results stream in via
    /// the `desk-discovered` event.
    Scanning,
}

#[derive(Serialize)]
pub struct BootState {
    screen: BootScreen,
    /// Remembered/connected desk name, for the connecting and connected screens.
    name: Option<String>,
    /// Remembered/connected desk address, so the connecting screen can list the
    /// desk as a row (with a "trying to connect" marker) before it's discovered.
    address: Option<String>,
    height_cm: Option<f64>,
    moving: bool,
    /// How close (cm) the live height must be to a preset target to count as
    /// "at" it. Owned by desk-core so the frontend never hardcodes its own copy.
    arrive_tolerance_cm: f64,
}

// ---------------------------------------------------------------------------
// Tauri commands: the frontend's entry points
// ---------------------------------------------------------------------------

type Ctrl<'a> = State<'a, Arc<DeskController>>;

#[tauri::command]
pub async fn desk_boot(app: AppHandle, state: Ctrl<'_>) -> Result<BootState, String> {
    let tolerance = arrive_tolerance_cm();
    let cfg = load_config(&app);
    let state_for = |screen, name, address, height_cm, moving| BootState {
        screen,
        name,
        address,
        height_cm,
        moving,
        arrive_tolerance_cm: tolerance,
    };

    // Already connected: the webview reloaded but the backend kept the link.
    if state.is_connected().await {
        return Ok(state_for(
            BootScreen::Connected,
            cfg.last_name,
            cfg.last_address,
            state.current_cm(),
            state.is_busy().await,
        ));
    }

    // A boot is already running (StrictMode remount in dev, or a Bluetooth
    // recovery racing launch): report it rather than starting a second connect.
    if !state.try_begin_boot() {
        return Ok(state_for(
            BootScreen::Connecting,
            cfg.last_name,
            cfg.last_address,
            None,
            false,
        ));
    }

    if matches!(state.bluetooth_state().await, BluetoothState::Off) {
        state.end_boot();
        return Ok(state_for(BootScreen::BluetoothOff, None, None, None, false));
    }

    match cfg.last_address {
        Some(address) => {
            // Reconnect off-thread so this command returns immediately and the
            // UI can paint the connecting state. `connect_named_persistent`
            // retries forever (1s backoff) and stays quiet on intermediate
            // failures, so the UI keeps showing "Connecting to <desk>" until
            // the desk is actually reachable, instead of flickering through a
            // scan screen on every failed attempt.
            let name = cfg.last_name;
            let ctrl = state.inner().clone();
            let connect_name = name.clone();
            let connect_address = address.clone();
            tauri::async_runtime::spawn(async move {
                ctrl.connect_named_persistent(&connect_address, connect_name.as_deref())
                    .await;
                ctrl.end_boot();
            });
            Ok(state_for(BootScreen::Connecting, name, Some(address), None, false))
        }
        None => {
            let result = state.scan_start().await;
            state.end_boot();
            result.map_err(|e| e.to_string())?;
            Ok(state_for(BootScreen::Scanning, None, None, None, false))
        }
    }
}

#[tauri::command]
pub async fn bluetooth_state(state: Ctrl<'_>) -> Result<String, String> {
    Ok(bt_state_str(state.bluetooth_state().await).into())
}

#[tauri::command]
pub async fn desk_scan_start(state: Ctrl<'_>) -> Result<(), String> {
    state.scan_start().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn desk_scan_stop(state: Ctrl<'_>) -> Result<(), String> {
    state.scan_stop().await;
    Ok(())
}

#[tauri::command]
pub async fn desk_connect(
    address: String,
    name: String,
    app: AppHandle,
    state: Ctrl<'_>,
) -> Result<bool, String> {
    let ok = state.connect_named(&address, Some(&name)).await;
    if ok {
        remember_desk(&app, Some((address, name))); // auto-reconnect on next launch
    }
    Ok(ok)
}

#[tauri::command]
pub async fn desk_disconnect(app: AppHandle, state: Ctrl<'_>) -> Result<(), String> {
    state.disconnect().await;
    remember_desk(&app, None); // explicit disconnect clears auto-reconnect
    Ok(())
}

/// Drop the live connection without forgetting the desk. Used when the radio
/// goes off: the link is dead, but we keep the remembered desk so the next boot
/// (after Bluetooth recovers) reconnects to it.
#[tauri::command]
pub async fn desk_drop(state: Ctrl<'_>) -> Result<(), String> {
    state.disconnect().await;
    Ok(())
}

#[tauri::command]
pub async fn desk_move_start(direction: Direction, state: Ctrl<'_>) -> Result<(), String> {
    state.inner().clone().start_hold(direction).await;
    Ok(())
}

#[tauri::command]
pub async fn desk_stop(state: Ctrl<'_>) -> Result<(), String> {
    state.inner().clone().stop_busy().await;
    Ok(())
}

#[tauri::command]
pub async fn desk_move_to_height(cm: f64, state: Ctrl<'_>) -> Result<(), String> {
    // The frontend sends cm; desk-core drives in raw counts.
    state
        .inner()
        .clone()
        .move_to_height_cmd(cm_to_raw(cm))
        .await;
    Ok(())
}

/// Open the OS Bluetooth settings (the "Enable Bluetooth" button). We don't
/// toggle the radio programmatically; we just take the user there. Routed
/// through `tauri-plugin-opener` so Windows doesn't flash a console window.
#[tauri::command]
pub fn open_bluetooth_settings(app: AppHandle) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let url = "ms-settings:bluetooth";
    #[cfg(target_os = "macos")]
    let url = "x-apple.systempreferences:com.apple.preference.bluetooth";
    #[cfg(all(unix, not(target_os = "macos")))]
    let url = "settings://bluetooth";

    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| e.to_string())
}
