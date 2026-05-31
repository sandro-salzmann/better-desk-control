// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use serde_json::json;
use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_store::StoreExt;

mod desk;
mod tray;
mod update;

/// Marks that we've already applied the first-run autostart default, so a later
/// manual toggle-off is never re-enabled on the next launch.
const KEY_AUTOSTART_INITIALIZED: &str = "autostart_initialized";

/// Opt the user into launch-at-startup once, on first run. After this the user's
/// choice (via the settings toggle) is the only thing that decides autostart.
fn default_enable_autostart(app: &AppHandle) {
    let Ok(store) = app.store(desk::CONFIG_STORE) else {
        return;
    };
    if store.get(KEY_AUTOSTART_INITIALIZED).is_some() {
        return;
    }
    let _ = app.autolaunch().enable();
    store.set(KEY_AUTOSTART_INITIALIZED, json!(true));
    let _ = store.save();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }))
        // launch-at-startup: the registered command carries `--minimized` so a
        // boot-time launch comes up quietly in the tray (see `setup`).
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        // restore window size/position, but NOT visibility: whether the window
        // shows on launch is decided by the `--minimized` flag below, not by
        // whether it happened to be hidden in the tray at last exit.
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(
                    tauri_plugin_window_state::StateFlags::all()
                        & !tauri_plugin_window_state::StateFlags::VISIBLE,
                )
                .build(),
        )
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // single DeskController shared across all commands
            let controller = desk::build_controller(app.handle().clone());
            // watch the adapter so the UI reacts to Bluetooth being toggled
            tauri::async_runtime::spawn(controller.clone().watch_bluetooth());
            app.manage(controller);

            // self-update: check + download in the background, then the UI
            // prompts the user to restart (see `update` module).
            app.manage(update::PendingUpdate::default());
            tauri::async_runtime::spawn(update::check_and_download(app.handle().clone()));

            // closing the window hides it to the system tray (see `tray`)
            tray::build(app.handle())?;

            // launch-at-startup is on by default; only applied once (see fn).
            default_enable_autostart(app.handle());

            // when launched at startup we pass `--minimized`; start hidden in
            // the tray instead of popping the window onto the user's screen.
            if std::env::args().any(|arg| arg == "--minimized") {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            desk::desk_boot,
            desk::bluetooth_state,
            desk::desk_scan_start,
            desk::desk_scan_stop,
            desk::desk_connect,
            desk::desk_disconnect,
            desk::desk_move_start,
            desk::desk_move_to_start,
            desk::desk_stop,
            desk::open_bluetooth_settings,
            update::update_install,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
