use tauri::{AppHandle, State};
use crate::db::Database;
use crate::models::pomodoro::{PomodoroSettings, DayStat};

#[tauri::command]
pub fn get_pomodoro_settings(db: State<'_, Database>) -> Result<PomodoroSettings, String> {
    db.get_pomodoro_settings().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_pomodoro_settings(db: State<'_, Database>, settings: PomodoroSettings) -> Result<(), String> {
    db.save_pomodoro_settings(&settings).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn record_pomodoro_session(db: State<'_, Database>, duration_min: i64) -> Result<(), String> {
    db.record_pomodoro_session(duration_min).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_today_pomodoro_count(db: State<'_, Database>) -> Result<i64, String> {
    db.get_today_pomodoro_count().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_weekly_pomodoro_stats(db: State<'_, Database>) -> Result<Vec<DayStat>, String> {
    db.get_weekly_pomodoro_stats().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_tray_tooltip(app: AppHandle, text: String) -> Result<(), String> {
    if let Some(tray) = app.tray_by_id("main") {
        tray.set_tooltip(Some(&text)).map_err(|e| e.to_string())
    } else {
        Ok(())
    }
}
