use crate::db::Database;
use crate::models::settings::AppSettings;
use crate::models::toolbox::{DatabaseQueryInput, DatabaseQueryResult};
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder};

const MAX_TOOLBOX_SQL_CHARS: usize = 20_000;
const DEFAULT_TOOLBOX_QUERY_ROWS: usize = 100;
const MAX_TOOLBOX_QUERY_ROWS: usize = 500;

#[tauri::command]
pub fn get_db_path(db: State<'_, Database>) -> Result<String, String> {
    Ok(db.db_path())
}

#[tauri::command]
pub fn query_database(
    db: State<'_, Database>,
    input: DatabaseQueryInput,
) -> Result<DatabaseQueryResult, String> {
    let sql = normalize_toolbox_sql(&input.sql)?;
    let max_rows = input
        .max_rows
        .unwrap_or(DEFAULT_TOOLBOX_QUERY_ROWS)
        .clamp(1, MAX_TOOLBOX_QUERY_ROWS);
    match normalize_toolbox_db_path(input.db_path.as_deref()) {
        Some(path) => Database::query_database_file_readonly(&path, &sql, max_rows),
        None => db.query_database_readonly(&sql, max_rows),
    }
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

fn normalize_toolbox_sql(sql: &str) -> Result<String, String> {
    let trimmed = sql.trim();
    if trimmed.is_empty() {
        return Err("SQL is required.".to_string());
    }
    if trimmed.len() > MAX_TOOLBOX_SQL_CHARS {
        return Err(format!(
            "SQL is too long: {} characters > {} characters.",
            trimmed.len(),
            MAX_TOOLBOX_SQL_CHARS
        ));
    }

    let sql = trimmed.strip_suffix(';').unwrap_or(trimmed).trim();
    if sql.contains(';') {
        return Err("Only one SQL statement can be run at a time.".to_string());
    }

    let first = sql
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    match first.as_str() {
        "select" | "with" | "pragma" | "explain" => Ok(sql.to_string()),
        _ => {
            Err("Only read-only SELECT, WITH, PRAGMA, and EXPLAIN queries are allowed.".to_string())
        }
    }
}

fn normalize_toolbox_db_path(path: Option<&str>) -> Option<PathBuf> {
    let trimmed = path?.trim();
    if trimmed.is_empty() {
        return None;
    }
    let expanded = if trimmed == "~" {
        dirs::home_dir()
    } else if let Some(rest) = trimmed.strip_prefix("~/") {
        dirs::home_dir().map(|home| home.join(rest))
    } else {
        None
    };
    Some(expanded.unwrap_or_else(|| PathBuf::from(trimmed)))
}
