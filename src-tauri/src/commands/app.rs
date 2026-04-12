use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder};
use crate::db::Database;
use crate::models::settings::AppSettings;

#[tauri::command]
pub fn get_db_path(db: State<'_, Database>) -> Result<String, String> {
    Ok(db.db_path())
}

#[tauri::command]
pub fn get_app_settings(db: State<'_, Database>) -> Result<AppSettings, String> {
    db.get_app_settings().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_app_settings(db: State<'_, Database>, settings: AppSettings) -> Result<(), String> {
    db.save_app_settings(&settings).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn quit_app(app: AppHandle) -> Result<(), String> {
    app.exit(0);
    Ok(())
}

#[tauri::command]
pub fn open_note_window(app: AppHandle, note_id: i64, title: String) -> Result<(), String> {
    let label = format!("note-{}", note_id);

    if let Some(win) = app.get_webview_window(&label) {
        win.show().map_err(|e| e.to_string())?;
        win.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    let url = WebviewUrl::App(format!("index.html?note={}", note_id).into());
    let win_title = if title.is_empty() {
        "Sticky Note".to_string()
    } else {
        title
    };

    WebviewWindowBuilder::new(&app, &label, url)
        .title(&win_title)
        .inner_size(400.0, 500.0)
        .min_inner_size(280.0, 200.0)
        .resizable(true)
        .minimizable(true)
        .maximizable(true)
        .build()
        .map_err(|e| e.to_string())?;

    Ok(())
}
