//! Tauri adapter over [`desk_core`].
//!
//! All the BLE work lives in the `desk-core` crate (shared with `desk-cli`).
//! This file only bridges it to Tauri:
//!
//! * [`TauriReporter`] turns the controller's callbacks into window events:
//!   `desk-height` ({ cm }), `desk-connection` ({ state, name?, address? }),
//!   `desk-motion` ({ moving, direction? }), `desk-screen` ({ screen }), and
//!   `desk-discovered` ({ name, address, rssi }) during a streaming scan.
//! * Config (`last_address`, for auto-reconnect) is persisted in the app config
//!   dir as `desk_config.json`.
//! * The `#[tauri::command]` functions are the frontend's entry points.

use std::sync::Arc;

use desk_core::{
    arrive_tolerance_cm, cm_to_raw, BluetoothState, ConnectionState, DeskController, DeskInfo,
    DeskReporter, Direction, Screen,
};
use serde::Serialize;
use serde_json::json;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_store::StoreExt;

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
    address: Option<String>,
}

#[derive(Clone, Serialize)]
struct MotionEvent {
    moving: bool,
    direction: Option<Direction>,
}

#[derive(Clone, Serialize)]
struct BluetoothEvent {
    state: &'static str,
}

#[derive(Clone, Serialize)]
struct ScreenEvent {
    screen: &'static str,
}

struct TauriReporter {
    app: AppHandle,
}

impl DeskReporter for TauriReporter {
    fn height(&self, _raw: i32, cm: f64) {
        let _ = self.app.emit("desk-height", HeightEvent { cm });
    }
    fn connection(&self, state: ConnectionState, name: Option<&str>, address: Option<&str>) {
        let _ = self.app.emit(
            "desk-connection",
            ConnectionEvent {
                state: state.as_str(),
                name: name.map(|s| s.to_string()),
                address: address.map(|s| s.to_string()),
            },
        );
    }
    fn motion(&self, moving: bool, direction: Option<Direction>) {
        let _ = self
            .app
            .emit("desk-motion", MotionEvent { moving, direction });
    }
    fn discovered(&self, desk: &DeskInfo) {
        let _ = self.app.emit("desk-discovered", desk.clone());
    }
    fn bluetooth(&self, state: BluetoothState) {
        let _ = self.app.emit(
            "desk-bluetooth",
            BluetoothEvent {
                state: state.as_str(),
            },
        );
    }
    fn screen(&self, screen: Screen) {
        let _ = self.app.emit(
            "desk-screen",
            ScreenEvent {
                screen: screen.as_str(),
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
// Config: persisted as desk_config.json via tauri-plugin-store
// ---------------------------------------------------------------------------

const CONFIG_STORE: &str = "desk_config.json";
const KEY_LAST_ADDRESS: &str = "last_address";
/// The remembered desk's advertised name, shown on the "Connecting…" screen
/// during auto-reconnect (before the live name is available).
const KEY_LAST_NAME: &str = "last_name";

#[derive(Default)]
struct DeskConfig {
    last_address: Option<String>,
    last_name: Option<String>,
}

fn load_config(app: &AppHandle) -> DeskConfig {
    let Ok(store) = app.store(CONFIG_STORE) else {
        return DeskConfig::default();
    };
    DeskConfig {
        last_address: store
            .get(KEY_LAST_ADDRESS)
            .and_then(|v| v.as_str().map(str::to_owned)),
        last_name: store
            .get(KEY_LAST_NAME)
            .and_then(|v| v.as_str().map(str::to_owned)),
    }
}

/// Persist (or, with `None`, clear) the desk to auto-reconnect to on next launch.
fn remember_desk(app: &AppHandle, desk: Option<(String, String)>) {
    let Ok(store) = app.store(CONFIG_STORE) else {
        return;
    };
    match desk {
        Some((address, name)) => {
            store.set(KEY_LAST_ADDRESS, json!(address));
            store.set(KEY_LAST_NAME, json!(name));
        }
        None => {
            store.delete(KEY_LAST_ADDRESS);
            store.delete(KEY_LAST_NAME);
        }
    }
    let _ = store.save();
}

// ---------------------------------------------------------------------------
// Boot: the single place that decides what the app does on launch. The frontend
// calls `desk_boot` once and renders the result, then follows the window events
// for the rest; it no longer orchestrates reconnect-vs-scan itself.
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct BootState {
    /// Initial screen, matching the strings emitted on the `desk-screen` event.
    screen: &'static str,
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
    let state_for = |screen: Screen, name, address, height_cm, moving| BootState {
        screen: screen.as_str(),
        name,
        address,
        height_cm,
        moving,
        arrive_tolerance_cm: tolerance,
    };

    // Already connected: the webview reloaded but the backend kept the link.
    if state.is_connected().await {
        return Ok(state_for(
            Screen::Connected,
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
            Screen::Connecting,
            cfg.last_name,
            cfg.last_address,
            None,
            false,
        ));
    }

    if matches!(state.bluetooth_state().await, BluetoothState::Off) {
        state.end_boot();
        return Ok(state_for(Screen::BluetoothOff, None, None, None, false));
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
            Ok(state_for(Screen::Connecting, name, Some(address), None, false))
        }
        None => {
            let result = state.scan_start().await;
            state.end_boot();
            result.map_err(|e| e.to_string())?;
            Ok(state_for(Screen::Scanning, None, None, None, false))
        }
    }
}

#[tauri::command]
pub async fn bluetooth_state(state: Ctrl<'_>) -> Result<String, String> {
    Ok(state.bluetooth_state().await.as_str().into())
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
