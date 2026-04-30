use crate::db::Database;
use crate::models::todo::{CreateTodo, Todo, UpdateTodo};
use tauri::State;

#[tauri::command]
pub fn list_todos(db: State<'_, Database>) -> Result<Vec<Todo>, String> {
    db.list_todos().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_todo(db: State<'_, Database>, input: CreateTodo) -> Result<Todo, String> {
    let description = input.description.as_deref().unwrap_or("");
    let priority = input.priority.unwrap_or(2);
    let deadline = input.deadline.as_deref();
    db.add_todo(
        &input.title,
        description,
        priority,
        deadline,
        input.recurrence.as_deref(),
        input.reminder_minutes_before,
    )
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
        input.clear_deadline.unwrap_or(false),
        input.recurrence.as_deref(),
        input.reminder_minutes_before,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_todo(db: State<'_, Database>, id: i64) -> Result<(), String> {
    db.delete_todo(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_due_todo_reminders(db: State<'_, Database>) -> Result<Vec<Todo>, String> {
    db.list_due_todo_reminders().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn mark_todo_reminded(db: State<'_, Database>, id: i64) -> Result<Todo, String> {
    db.mark_todo_reminded(id).map_err(|e| e.to_string())
}
