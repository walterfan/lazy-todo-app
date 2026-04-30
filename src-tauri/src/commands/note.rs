use crate::db::Database;
use crate::models::note::{CreateNote, StickyNote, UpdateNote};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::State;

#[derive(Debug, Deserialize)]
pub struct ExportNotesInput {
    pub note_ids: Vec<i64>,
    pub folder: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExportNotesResult {
    pub folder: String,
    pub files: Vec<String>,
}

#[tauri::command]
pub fn list_notes(db: State<'_, Database>) -> Result<Vec<StickyNote>, String> {
    db.list_notes().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_note(db: State<'_, Database>, input: CreateNote) -> Result<StickyNote, String> {
    let title = input.title.as_deref().unwrap_or("");
    let content = input.content.as_deref().unwrap_or("");
    let color = input.color.as_deref().unwrap_or("yellow");
    db.insert_note(title, content, color)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_note(db: State<'_, Database>, input: UpdateNote) -> Result<StickyNote, String> {
    db.update_note(
        input.id,
        input.title.as_deref(),
        input.content.as_deref(),
        input.color.as_deref(),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_note_pinned(
    db: State<'_, Database>,
    id: i64,
    pinned: bool,
) -> Result<StickyNote, String> {
    db.set_note_pinned(id, pinned).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_note(db: State<'_, Database>, id: i64) -> Result<(), String> {
    db.delete_note(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_notes_to_folder(
    db: State<'_, Database>,
    input: ExportNotesInput,
) -> Result<ExportNotesResult, String> {
    export_notes_to_folder_inner(&db, input)
}

fn export_notes_to_folder_inner(
    db: &Database,
    input: ExportNotesInput,
) -> Result<ExportNotesResult, String> {
    let note_ids = input
        .note_ids
        .into_iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if note_ids.is_empty() {
        return Err("Select at least one note to export.".to_string());
    }

    let notes = db.list_notes().map_err(|e| e.to_string())?;
    let selected = notes
        .into_iter()
        .filter(|note| note_ids.contains(&note.id))
        .collect::<Vec<_>>();
    if selected.len() != note_ids.len() {
        return Err("One or more selected notes no longer exist.".to_string());
    }

    let folder = export_folder(db, input.folder)?;
    fs::create_dir_all(&folder).map_err(|e| {
        format!(
            "Cannot create notes export folder {}: {e}",
            folder.display()
        )
    })?;

    let stamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
    let mut files = Vec::new();
    for note in selected {
        let title = if note.title.trim().is_empty() {
            format!("note-{}", note.id)
        } else {
            note.title.clone()
        };
        let file_name = format!(
            "{}-note-{}-{}.md",
            stamp,
            note.id,
            sanitize_file_stem(&title)
        );
        let path = unique_export_path(&folder, &file_name);
        fs::write(&path, format_note_markdown(&note))
            .map_err(|e| format!("Cannot export note {} to {}: {e}", note.id, path.display()))?;
        files.push(path.to_string_lossy().to_string());
    }

    Ok(ExportNotesResult {
        folder: folder.to_string_lossy().to_string(),
        files,
    })
}

fn export_folder(db: &Database, override_folder: Option<String>) -> Result<PathBuf, String> {
    if let Some(folder) = override_folder
        .as_deref()
        .map(str::trim)
        .filter(|folder| !folder.is_empty())
    {
        return Ok(PathBuf::from(folder));
    }

    let settings = db.get_app_settings().map_err(|e| e.to_string())?;
    if !settings.note_folder.trim().is_empty() {
        return Ok(PathBuf::from(settings.note_folder.trim()));
    }

    Ok(dirs::document_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("lazy-todo-app")
        .join("notes"))
}

fn format_note_markdown(note: &StickyNote) -> String {
    format!(
        "# {}\n\n- Note ID: {}\n- Color: {}\n- Pinned: {}\n- Created: {}\n- Updated: {}\n\n{}",
        if note.title.trim().is_empty() {
            "Untitled Note"
        } else {
            note.title.trim()
        },
        note.id,
        note.color,
        note.pinned,
        note.created_at,
        note.updated_at,
        note.content
    )
}

fn sanitize_file_stem(value: &str) -> String {
    let mut output = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else if ch.is_whitespace() {
                '-'
            } else {
                '_'
            }
        })
        .collect::<String>();
    while output.contains("--") {
        output = output.replace("--", "-");
    }
    let output = output.trim_matches(['-', '_']).to_string();
    if output.is_empty() {
        "note".to_string()
    } else {
        output.chars().take(80).collect()
    }
}

fn unique_export_path(folder: &Path, file_name: &str) -> PathBuf {
    let path = folder.join(file_name);
    if !path.exists() {
        return path;
    }
    let stem = Path::new(file_name)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("note");
    let extension = Path::new(file_name)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("md");
    for index in 2.. {
        let candidate = folder.join(format!("{stem}-{index}.{extension}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    unreachable!("unbounded unique file search should return");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::settings::AppSettings;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_test_dir(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("lazy_todo_notes_{name}_{suffix}"))
    }

    #[test]
    fn exports_selected_notes_to_configured_folder() {
        let db_root = unique_test_dir("db");
        let export_root = unique_test_dir("exports");
        let db = Database::new(&db_root).expect("create db");
        db.save_app_settings(&AppSettings {
            note_folder: export_root.to_string_lossy().to_string(),
            ..AppSettings::default()
        })
        .expect("save settings");
        let first = db
            .insert_note("Design", "Make the UI calm.", "blue")
            .expect("insert first");
        let second = db
            .insert_note("Private/Idea", "Save this one too.", "yellow")
            .expect("insert second");
        db.insert_note("Ignored", "Do not export.", "green")
            .expect("insert ignored");

        let result = export_notes_to_folder_inner(
            &db,
            ExportNotesInput {
                note_ids: vec![first.id, second.id],
                folder: None,
            },
        )
        .expect("export notes");

        assert_eq!(result.files.len(), 2);
        assert_eq!(result.folder, export_root.to_string_lossy());
        let joined = result
            .files
            .iter()
            .map(|path| fs::read_to_string(path).expect("read export"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(joined.contains("# Design"));
        assert!(joined.contains("Make the UI calm."));
        assert!(joined.contains("# Private/Idea"));
        assert!(joined.contains("Save this one too."));
        assert!(!joined.contains("Do not export."));

        let _ = fs::remove_dir_all(db_root);
        let _ = fs::remove_dir_all(export_root);
    }
}
