// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::Manager;

mod desk;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // single DeskController shared across all commands
            app.manage(desk::build_controller(app.handle().clone()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            desk::desk_scan,
            desk::desk_connect,
            desk::desk_connect_saved,
            desk::desk_disconnect,
            desk::desk_move_start,
            desk::desk_stop,
            desk::desk_save_preset,
            desk::desk_apply_preset,
            desk::desk_height,
            desk::desk_is_connected,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
