#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! Desktop app entry point. Commands live in `commands/`; scanning and
//! cleaning logic lives in the `cache_cleaner` library (shared with the CLI).

mod commands;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(commands::handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
