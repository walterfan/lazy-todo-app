mod commands;
mod db;
mod models;

use tauri::{
    Emitter, Manager,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
};
use db::Database;

fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let show_hide = MenuItem::with_id(app, "show_hide", "Show/Hide", true, None::<&str>)?;
    let new_note = MenuItem::with_id(app, "new_note", "New Note", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_hide, &new_note, &separator, &quit])?;

    let _tray = TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("Lazy Todo App")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            let window = app.get_webview_window("main").unwrap();
            match event.id.as_ref() {
                "show_hide" => {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "new_note" => {
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = window.emit("tray-new-note", ());
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, button_state: tauri::tray::MouseButtonState::Up, .. } = event {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let db_dir = match std::env::var("LAZY_TODO_DB_DIR") {
                Ok(dir) if !dir.is_empty() => std::path::PathBuf::from(dir),
                _ => app.path().app_data_dir().expect("failed to get app data dir"),
            };
            let database = Database::new(&db_dir).expect("failed to initialize database");
            app.manage(database);
            setup_tray(app).expect("failed to setup system tray");
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::todo::list_todos,
            commands::todo::add_todo,
            commands::todo::toggle_todo,
            commands::todo::update_todo,
            commands::todo::delete_todo,
            commands::note::list_notes,
            commands::note::add_note,
            commands::note::update_note,
            commands::note::delete_note,
            commands::pomodoro::get_pomodoro_settings,
            commands::pomodoro::save_pomodoro_settings,
            commands::pomodoro::record_pomodoro_session,
            commands::pomodoro::get_today_pomodoro_count,
            commands::pomodoro::get_weekly_pomodoro_stats,
            commands::pomodoro::update_tray_tooltip,
            commands::app::get_db_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
