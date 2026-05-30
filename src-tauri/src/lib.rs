// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::Manager;

mod desk;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }))
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // single DeskController shared across all commands
            let controller = desk::build_controller(app.handle().clone());
            // watch the adapter so the UI reacts to Bluetooth being toggled
            tauri::async_runtime::spawn(controller.clone().watch_bluetooth());
            app.manage(controller);
            Ok(())
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
