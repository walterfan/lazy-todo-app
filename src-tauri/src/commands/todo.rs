use tauri::State;
use crate::db::Database;
use crate::models::todo::{Todo, CreateTodo, UpdateTodo};

#[tauri::command]
pub fn list_todos(db: State<'_, Database>) -> Result<Vec<Todo>, String> {
    db.list_todos().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_todo(db: State<'_, Database>, input: CreateTodo) -> Result<Todo, String> {
    let description = input.description.as_deref().unwrap_or("");
    let priority = input.priority.unwrap_or(2);
    let deadline = input.deadline.as_deref();
    db.add_todo(&input.title, description, priority, deadline)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_todo(db: State<'_, Database>, id: i64) -> Result<Todo, String> {
    db.toggle_todo(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_todo(db: State<'_, Database>, input: UpdateTodo) -> Result<Todo, String> {
    db.update_todo(
        input.id,
        input.title.as_deref(),
        input.description.as_deref(),
        input.priority,
        input.deadline.as_deref(),
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_todo(db: State<'_, Database>, id: i64) -> Result<(), String> {
    db.delete_todo(id).map_err(|e| e.to_string())
}
