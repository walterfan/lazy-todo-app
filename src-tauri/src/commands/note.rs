use tauri::State;
use crate::db::Database;
use crate::models::note::{StickyNote, CreateNote, UpdateNote};

#[tauri::command]
pub fn list_notes(db: State<'_, Database>) -> Result<Vec<StickyNote>, String> {
    db.list_notes().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_note(db: State<'_, Database>, input: CreateNote) -> Result<StickyNote, String> {
    let title = input.title.as_deref().unwrap_or("");
    let content = input.content.as_deref().unwrap_or("");
    let color = input.color.as_deref().unwrap_or("yellow");
    db.insert_note(title, content, color).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_note(db: State<'_, Database>, input: UpdateNote) -> Result<StickyNote, String> {
    db.update_note(
        input.id,
        input.title.as_deref(),
        input.content.as_deref(),
        input.color.as_deref(),
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_note(db: State<'_, Database>, id: i64) -> Result<(), String> {
    db.delete_note(id).map_err(|e| e.to_string())
}
