// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::Manager;

mod desk;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
            desk::desk_snapshot,
            desk::bluetooth_state,
            desk::desk_scan_start,
            desk::desk_scan_stop,
            desk::desk_connect,
            desk::desk_connect_saved,
            desk::desk_disconnect,
            desk::desk_move_start,
            desk::desk_stop,
            desk::desk_move_to_height,
            desk::open_bluetooth_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
