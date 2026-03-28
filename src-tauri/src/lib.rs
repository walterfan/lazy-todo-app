mod commands;
mod db;
mod models;

use tauri::Manager;
use db::Database;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir().expect("failed to get app data dir");
            let database = Database::new(&app_dir).expect("failed to initialize database");
            app.manage(database);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::todo::list_todos,
            commands::todo::add_todo,
            commands::todo::toggle_todo,
            commands::todo::update_todo,
            commands::todo::delete_todo,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
