use tauri::State;
use crate::db::Database;

#[tauri::command]
pub fn get_db_path(db: State<'_, Database>) -> Result<String, String> {
    Ok(db.db_path())
}
