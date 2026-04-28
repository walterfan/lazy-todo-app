mod commands;
mod db;
mod models;

use std::path::PathBuf;
use tauri::{
    Emitter, Manager,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
};
use db::Database;

fn resolve_db_dir(default_dir: PathBuf) -> PathBuf {
    if let Ok(dir) = std::env::var("LAZY_TODO_DB_DIR") {
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }

    if let Some(home) = dirs::home_dir() {
        let config_file = home.join(".config").join("lazy-todo-app").join("config.json");
        if config_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_file) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(dir) = json.get("db_dir").and_then(|v| v.as_str()) {
                        let expanded = if dir.starts_with('~') {
                            home.join(dir.trim_start_matches("~/"))
                        } else {
                            PathBuf::from(dir)
                        };
                        return expanded;
                    }
                }
            }
        }
    }

    default_dir
}

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
            let default_dir = app.path().app_data_dir().expect("failed to get app data dir");
            let db_dir = resolve_db_dir(default_dir);
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
            commands::secretary::get_secretary_settings,
            commands::secretary::save_secretary_settings,
            commands::secretary::validate_secretary_config,
            commands::secretary::list_secretary_personas,
            commands::secretary::save_secretary_persona,
            commands::secretary::delete_secretary_persona,
            commands::secretary::list_secretary_profiles,
            commands::secretary::save_secretary_profile,
            commands::secretary::delete_secretary_profile,
            commands::secretary::list_secretary_skills,
            commands::secretary::refresh_secretary_skills,
            commands::secretary::list_secretary_memories,
            commands::secretary::save_secretary_memory,
            commands::secretary::delete_secretary_memory,
            commands::secretary::list_secretary_reminders,
            commands::secretary::due_secretary_reminders,
            commands::secretary::save_secretary_reminder,
            commands::secretary::delete_secretary_reminder,
            commands::secretary::get_secretary_app_context,
            commands::secretary::confirm_secretary_note_edit,
            commands::secretary::start_secretary_conversation,
            commands::secretary::send_secretary_message,
            commands::secretary::send_secretary_message_stream,
            commands::secretary::list_secretary_conversations,
            commands::secretary::load_secretary_conversation,
            commands::secretary::save_secretary_transcript,
            commands::app::get_db_path,
            commands::app::get_app_settings,
            commands::app::save_app_settings,
            commands::app::quit_app,
            commands::app::open_note_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
