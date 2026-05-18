use crate::db::Database;
use crate::models::note::{CreateNote, NoteTemplate, StickyNote, UpdateNote};
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
    let note = db
        .insert_note(title, content, color)
        .map_err(|e| e.to_string())?;
    let final_note = mirror_note_after_save(&db, note, None).map_err(|e| e.to_string())?;
    Ok(final_note)
}

#[tauri::command]
pub fn update_note(db: State<'_, Database>, input: UpdateNote) -> Result<StickyNote, String> {
    let prior = find_note(&db, input.id).map_err(|e| e.to_string())?;
    let updated = db
        .update_note(
            input.id,
            input.title.as_deref(),
            input.content.as_deref(),
            input.color.as_deref(),
        )
        .map_err(|e| e.to_string())?;
    let prior_path = prior.and_then(|n| n.file_path);
    let final_note =
        mirror_note_after_save(&db, updated, prior_path.as_deref()).map_err(|e| e.to_string())?;
    Ok(final_note)
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
    let prior = find_note(&db, id).map_err(|e| e.to_string())?;
    db.delete_note(id).map_err(|e| e.to_string())?;
    if let Some(note) = prior {
        if let Some(path) = note.file_path {
            let _ = fs::remove_file(&path);
        }
    }
    Ok(())
}

#[tauri::command]
pub fn list_note_templates(db: State<'_, Database>) -> Result<Vec<NoteTemplate>, String> {
    list_note_templates_inner(&db).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_notes_to_folder(
    db: State<'_, Database>,
    input: ExportNotesInput,
) -> Result<ExportNotesResult, String> {
    export_notes_to_folder_inner(&db, input)
}

fn find_note(db: &Database, id: i64) -> Result<Option<StickyNote>, rusqlite::Error> {
    let notes = db.list_notes()?;
    Ok(notes.into_iter().find(|n| n.id == id))
}

fn mirror_note_after_save(
    db: &Database,
    note: StickyNote,
    prior_path: Option<&str>,
) -> Result<StickyNote, String> {
    let settings = db.get_app_settings().map_err(|e| e.to_string())?;
    let folder = settings.note_folder.trim();
    if folder.is_empty() {
        return Ok(note);
    }
    let folder_path = PathBuf::from(folder);
    if fs::create_dir_all(&folder_path).is_err() {
        return Ok(note);
    }

    let existing_path = note
        .file_path
        .as_deref()
        .or(prior_path)
        .map(PathBuf::from)
        .filter(|p| {
            p.parent()
                .map(|parent| parent == folder_path)
                .unwrap_or(false)
        });

    let target_path = match existing_path {
        Some(path) => path,
        None => {
            let title = if note.title.trim().is_empty() {
                format!("note-{}", note.id)
            } else {
                note.title.clone()
            };
            let file_name = format!("note-{}-{}.md", note.id, sanitize_file_stem(&title));
            unique_export_path(&folder_path, &file_name)
        }
    };

    if fs::write(&target_path, format_note_markdown(&note)).is_err() {
        return Ok(note);
    }
    let stored_path = target_path.to_string_lossy().to_string();
    match db.set_note_file_path(note.id, Some(&stored_path)) {
        Ok(updated) => Ok(updated),
        Err(_) => Ok(note),
    }
}

fn list_note_templates_inner(db: &Database) -> Result<Vec<NoteTemplate>, String> {
    let settings = db.get_app_settings().map_err(|e| e.to_string())?;
    let mut templates = vec![daily_note_template()];
    for path in settings.note_template_files {
        let trimmed = path.trim();
        if trimmed.is_empty() {
            continue;
        }
        match load_template_from_file(Path::new(trimmed)) {
            Ok(template) => templates.push(template),
            Err(_) => continue,
        }
    }
    Ok(templates)
}

fn daily_note_template() -> NoteTemplate {
    let body = "## Goals\n\n- \n\n## Notes\n\n- \n\n## Reflection\n\n- ".to_string();
    let mut t = NoteTemplate {
        id: "builtin:daily-note".to_string(),
        name: "Daily Note".to_string(),
        title: "Daily Note {{date}}".to_string(),
        body,
        source: "builtin".to_string(),
        path: None,
    };
    expand_template_placeholders(&mut t);
    t
}

fn load_template_from_file(path: &Path) -> Result<NoteTemplate, String> {
    let raw = fs::read_to_string(path)
        .map_err(|e| format!("Cannot read template {}: {e}", path.display()))?;
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("template")
        .to_string();
    let (title_from_h1, body) = parse_first_h1(&raw);
    let (name, title) = match title_from_h1 {
        Some(h1) => (template_name_from_title(&h1, &stem), h1),
        None => (stem.clone(), stem),
    };
    let mut t = NoteTemplate {
        id: format!("file:{}", path.display()),
        name,
        title,
        body,
        source: "file".to_string(),
        path: Some(path.to_string_lossy().to_string()),
    };
    expand_template_placeholders(&mut t);
    Ok(t)
}

fn parse_first_h1(markdown: &str) -> (Option<String>, String) {
    let mut lines = markdown.lines();
    while let Some(line) = lines.next() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("# ") {
            let title = rest.trim().to_string();
            let mut body_lines: Vec<&str> = lines.collect();
            while body_lines
                .first()
                .map(|l| l.trim().is_empty())
                .unwrap_or(false)
            {
                body_lines.remove(0);
            }
            return (Some(title), body_lines.join("\n"));
        } else {
            return (None, markdown.to_string());
        }
    }
    (None, markdown.to_string())
}

fn expand_template_placeholders(template: &mut NoteTemplate) {
    let now = Local::now();
    let date = now.format("%Y-%m-%d").to_string();
    let datetime = now.format("%Y-%m-%d %H:%M").to_string();
    let weekday = now.format("%A").to_string();
    template.title = expand_placeholders(&template.title, &date, &datetime, &weekday);
    template.body = expand_placeholders(&template.body, &date, &datetime, &weekday);
}

fn expand_placeholders(value: &str, date: &str, datetime: &str, weekday: &str) -> String {
    value
        .replace("{{date}}", date)
        .replace("{{datetime}}", datetime)
        .replace("{{weekday}}", weekday)
}

fn template_name_from_title(title: &str, fallback: &str) -> String {
    let name = title
        .replace("{{date}}", "")
        .replace("{{datetime}}", "")
        .replace("{{weekday}}", "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if name.is_empty() {
        fallback.to_string()
    } else {
        name
    }
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
    let title = if note.title.trim().is_empty() {
        "Untitled Note"
    } else {
        note.title.trim()
    };
    format!(
        "# {}\n\n- Note ID: {}\n- Color: {}\n- Pinned: {}\n- Created: {}\n- Updated: {}\n\n{}",
        title, note.id, note.color, note.pinned, note.created_at, note.updated_at, note.content
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

    fn db_with_folder(name: &str) -> (Database, PathBuf, PathBuf) {
        let db_root = unique_test_dir(&format!("{name}_db"));
        let folder = unique_test_dir(&format!("{name}_folder"));
        let db = Database::new(&db_root).expect("create db");
        db.save_app_settings(&AppSettings {
            note_folder: folder.to_string_lossy().to_string(),
            ..AppSettings::default()
        })
        .expect("save settings");
        (db, db_root, folder)
    }

    #[test]
    fn parse_first_h1_extracts_title_and_body() {
        let (title, body) = parse_first_h1("# Meeting Notes\n\nFirst topic\nSecond line");
        assert_eq!(title.as_deref(), Some("Meeting Notes"));
        assert_eq!(body, "First topic\nSecond line");
    }

    #[test]
    fn parse_first_h1_returns_none_when_missing() {
        let raw = "Just some body\nwithout heading.";
        let (title, body) = parse_first_h1(raw);
        assert!(title.is_none());
        assert_eq!(body, raw);
    }

    #[test]
    fn daily_note_template_expands_date_placeholder() {
        let template = daily_note_template();
        let today = Local::now().format("%Y-%m-%d").to_string();
        assert_eq!(template.id, "builtin:daily-note");
        assert_eq!(template.name, "Daily Note");
        assert!(template.title.contains(&today));
        assert!(!template.title.contains("{{date}}"));
    }

    #[test]
    fn list_note_templates_returns_builtin_and_configured_files() {
        let (db, db_root, folder) = db_with_folder("list_templates");
        let template_dir = unique_test_dir("templates");
        fs::create_dir_all(&template_dir).expect("create template dir");
        let template_path = template_dir.join("standup.md");
        fs::write(&template_path, "# Standup {{date}}\n\n## Updates\n\n- ")
            .expect("write template file");
        let missing_path = template_dir.join("missing.md");
        let mut settings = db.get_app_settings().expect("get settings");
        settings.note_template_files = vec![
            template_path.to_string_lossy().to_string(),
            missing_path.to_string_lossy().to_string(),
        ];
        db.save_app_settings(&settings).expect("save settings");

        let templates = list_note_templates_inner(&db).expect("list templates");
        assert_eq!(templates.len(), 2);
        assert_eq!(templates[0].id, "builtin:daily-note");
        assert_eq!(templates[1].name, "Standup");
        let today = Local::now().format("%Y-%m-%d").to_string();
        assert!(templates[1].title.contains(&today));

        let _ = fs::remove_dir_all(db_root);
        let _ = fs::remove_dir_all(folder);
        let _ = fs::remove_dir_all(template_dir);
    }

    #[test]
    fn add_note_writes_mirror_file_when_folder_configured() {
        let (db, db_root, folder) = db_with_folder("mirror_add");
        let note = db
            .insert_note("Plan", "Body text", "yellow")
            .expect("insert");
        let final_note = mirror_note_after_save(&db, note, None).expect("mirror");
        assert!(final_note.file_path.is_some());
        let path = PathBuf::from(final_note.file_path.unwrap());
        assert!(path.exists(), "mirrored file should exist");
        let contents = fs::read_to_string(&path).expect("read mirror file");
        assert!(contents.contains("# Plan"));
        assert!(contents.contains("Body text"));
        let _ = fs::remove_dir_all(db_root);
        let _ = fs::remove_dir_all(folder);
    }

    #[test]
    fn update_note_reuses_existing_mirror_path() {
        let (db, db_root, folder) = db_with_folder("mirror_update");
        let note = db.insert_note("Title", "v1", "yellow").expect("insert");
        let first = mirror_note_after_save(&db, note, None).expect("first mirror");
        let first_path = first.file_path.clone().expect("path");

        let updated = db
            .update_note(first.id, None, Some("v2"), None)
            .expect("update");
        let second = mirror_note_after_save(&db, updated, Some(&first_path)).expect("mirror");
        assert_eq!(second.file_path.as_deref(), Some(first_path.as_str()));
        let contents = fs::read_to_string(&first_path).expect("read");
        assert!(contents.contains("v2"));
        assert!(!contents.contains("v1\n"));
        let _ = fs::remove_dir_all(db_root);
        let _ = fs::remove_dir_all(folder);
    }

    #[test]
    fn delete_note_removes_mirror_file() {
        let (db, db_root, folder) = db_with_folder("mirror_delete");
        let note = db.insert_note("Bye", "Body", "yellow").expect("insert");
        let stored = mirror_note_after_save(&db, note, None).expect("mirror");
        let path = stored.file_path.clone().expect("path");
        assert!(PathBuf::from(&path).exists());

        // Simulate delete_note command behavior.
        let prior = find_note(&db, stored.id).expect("find");
        db.delete_note(stored.id).expect("delete");
        if let Some(note) = prior {
            if let Some(p) = note.file_path {
                let _ = fs::remove_file(&p);
            }
        }
        assert!(!PathBuf::from(&path).exists(), "mirror file removed");
        let _ = fs::remove_dir_all(db_root);
        let _ = fs::remove_dir_all(folder);
    }

    #[test]
    fn mirror_no_op_when_folder_blank() {
        let db_root = unique_test_dir("mirror_blank_db");
        let db = Database::new(&db_root).expect("create db");
        let note = db.insert_note("X", "Y", "yellow").expect("insert");
        let final_note = mirror_note_after_save(&db, note, None).expect("mirror noop");
        assert!(final_note.file_path.is_none());
        let _ = fs::remove_dir_all(db_root);
    }

    #[test]
    fn mirror_failure_does_not_block_saved_note() {
        let db_root = unique_test_dir("mirror_invalid_db");
        let folder_parent = unique_test_dir("mirror_invalid_parent");
        fs::create_dir_all(&folder_parent).expect("create invalid folder parent");
        let folder_file = folder_parent.join("not-a-folder");
        fs::write(&folder_file, "plain file").expect("write folder placeholder file");
        let db = Database::new(&db_root).expect("create db");
        db.save_app_settings(&AppSettings {
            note_folder: folder_file.to_string_lossy().to_string(),
            ..AppSettings::default()
        })
        .expect("save settings");
        let note = db
            .insert_note("Keep", "Still saved", "yellow")
            .expect("insert");

        let final_note = mirror_note_after_save(&db, note, None).expect("mirror best effort");

        assert_eq!(final_note.title, "Keep");
        assert!(final_note.file_path.is_none());
        let stored = find_note(&db, final_note.id)
            .expect("find saved note")
            .expect("note remains in sqlite");
        assert_eq!(stored.content, "Still saved");
        let _ = fs::remove_dir_all(db_root);
        let _ = fs::remove_dir_all(folder_parent);
    }

    #[test]
    fn exports_selected_notes_to_configured_folder() {
        let db_root = unique_test_dir("export_db");
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
