use rusqlite::{Connection, Result, params};
use std::sync::Mutex;

use crate::models::todo::Todo;
use crate::models::note::StickyNote;
use crate::models::pomodoro::{PomodoroMilestone, PomodoroSettings, DayStat};
use crate::models::settings::AppSettings;

pub struct Database {
    conn: Mutex<Connection>,
    db_path: std::path::PathBuf,
}

impl Database {
    fn ensure_pomodoro_settings_schema(conn: &Connection) -> Result<()> {
        let mut stmt = conn.prepare("PRAGMA table_info(pomodoro_settings)")?;
        let column_names = stmt
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<Vec<_>>>()?;

        if !column_names.iter().any(|name| name == "milestones_json") {
            conn.execute(
                "ALTER TABLE pomodoro_settings ADD COLUMN milestones_json TEXT NOT NULL DEFAULT '[]'",
                [],
            )?;
        }

        Ok(())
    }

    pub fn new(app_dir: &std::path::Path) -> Result<Self> {
        std::fs::create_dir_all(app_dir).ok();
        let db_path = app_dir.join("todos.db");
        let conn = Connection::open(&db_path)?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS todos (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                title       TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                priority    INTEGER NOT NULL DEFAULT 2,
                completed   INTEGER NOT NULL DEFAULT 0,
                deadline    TEXT,
                created_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS sticky_notes (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                title       TEXT NOT NULL DEFAULT '',
                content     TEXT NOT NULL DEFAULT '',
                color       TEXT NOT NULL DEFAULT 'yellow',
                created_at  TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS pomodoro_settings (
                id               INTEGER PRIMARY KEY CHECK (id = 1),
                work_minutes     INTEGER NOT NULL DEFAULT 25,
                short_break_min  INTEGER NOT NULL DEFAULT 5,
                long_break_min   INTEGER NOT NULL DEFAULT 15,
                rounds_per_cycle INTEGER NOT NULL DEFAULT 4,
                milestones_json  TEXT NOT NULL DEFAULT '[]'
            );
            CREATE TABLE IF NOT EXISTS pomodoro_sessions (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                completed_at TEXT NOT NULL DEFAULT (datetime('now')),
                duration_min INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS app_settings (
                id              INTEGER PRIMARY KEY CHECK (id = 1),
                page_size       INTEGER NOT NULL DEFAULT 50,
                todo_display    TEXT NOT NULL DEFAULT 'list',
                note_display    TEXT NOT NULL DEFAULT 'grid',
                note_template   TEXT NOT NULL DEFAULT '',
                note_folder     TEXT NOT NULL DEFAULT ''
            );"
        )?;

        Self::ensure_pomodoro_settings_schema(&conn)?;

        Ok(Self { conn: Mutex::new(conn), db_path })
    }

    pub fn db_path(&self) -> String {
        self.db_path.to_string_lossy().to_string()
    }

    pub fn list_todos(&self) -> Result<Vec<Todo>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, title, description, priority, completed, deadline, created_at
             FROM todos ORDER BY completed ASC, priority ASC, deadline ASC NULLS LAST"
        )?;

        let todos = stmt.query_map([], |row| {
            Ok(Todo {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                priority: row.get(3)?,
                completed: row.get::<_, i32>(4)? != 0,
                deadline: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?.collect::<Result<Vec<_>>>()?;

        Ok(todos)
    }

    pub fn add_todo(&self, title: &str, description: &str, priority: i32, deadline: Option<&str>) -> Result<Todo> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT INTO todos (title, description, priority, deadline) VALUES (?1, ?2, ?3, ?4)",
            params![title, description, priority, deadline],
        )?;
        let id = conn.last_insert_rowid();

        let mut stmt = conn.prepare(
            "SELECT id, title, description, priority, completed, deadline, created_at FROM todos WHERE id = ?1"
        )?;
        stmt.query_row(params![id], |row| {
            Ok(Todo {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                priority: row.get(3)?,
                completed: row.get::<_, i32>(4)? != 0,
                deadline: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
    }

    pub fn toggle_todo(&self, id: i64) -> Result<Todo> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "UPDATE todos SET completed = NOT completed WHERE id = ?1",
            params![id],
        )?;

        let mut stmt = conn.prepare(
            "SELECT id, title, description, priority, completed, deadline, created_at FROM todos WHERE id = ?1"
        )?;
        stmt.query_row(params![id], |row| {
            Ok(Todo {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                priority: row.get(3)?,
                completed: row.get::<_, i32>(4)? != 0,
                deadline: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
    }

    pub fn update_todo(&self, id: i64, title: Option<&str>, description: Option<&str>, priority: Option<i32>, deadline: Option<&str>) -> Result<Todo> {
        let conn = self.conn.lock().expect("db lock poisoned");

        if let Some(t) = title {
            conn.execute("UPDATE todos SET title = ?1 WHERE id = ?2", params![t, id])?;
        }
        if let Some(d) = description {
            conn.execute("UPDATE todos SET description = ?1 WHERE id = ?2", params![d, id])?;
        }
        if let Some(p) = priority {
            conn.execute("UPDATE todos SET priority = ?1 WHERE id = ?2", params![p, id])?;
        }
        if let Some(dl) = deadline {
            conn.execute("UPDATE todos SET deadline = ?1 WHERE id = ?2", params![dl, id])?;
        }

        let mut stmt = conn.prepare(
            "SELECT id, title, description, priority, completed, deadline, created_at FROM todos WHERE id = ?1"
        )?;
        stmt.query_row(params![id], |row| {
            Ok(Todo {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                priority: row.get(3)?,
                completed: row.get::<_, i32>(4)? != 0,
                deadline: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
    }

    pub fn delete_todo(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute("DELETE FROM todos WHERE id = ?1", params![id])?;
        Ok(())
    }

    // --- Sticky Notes ---

    pub fn list_notes(&self) -> Result<Vec<StickyNote>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, title, content, color, created_at, updated_at
             FROM sticky_notes ORDER BY updated_at DESC"
        )?;

        let notes = stmt.query_map([], |row| {
            Ok(StickyNote {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                color: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?.collect::<Result<Vec<_>>>()?;

        Ok(notes)
    }

    pub fn insert_note(&self, title: &str, content: &str, color: &str) -> Result<StickyNote> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT INTO sticky_notes (title, content, color) VALUES (?1, ?2, ?3)",
            params![title, content, color],
        )?;
        let id = conn.last_insert_rowid();

        let mut stmt = conn.prepare(
            "SELECT id, title, content, color, created_at, updated_at FROM sticky_notes WHERE id = ?1"
        )?;
        stmt.query_row(params![id], |row| {
            Ok(StickyNote {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                color: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })
    }

    pub fn update_note(&self, id: i64, title: Option<&str>, content: Option<&str>, color: Option<&str>) -> Result<StickyNote> {
        let conn = self.conn.lock().expect("db lock poisoned");

        if let Some(t) = title {
            conn.execute("UPDATE sticky_notes SET title = ?1, updated_at = datetime('now') WHERE id = ?2", params![t, id])?;
        }
        if let Some(c) = content {
            conn.execute("UPDATE sticky_notes SET content = ?1, updated_at = datetime('now') WHERE id = ?2", params![c, id])?;
        }
        if let Some(clr) = color {
            conn.execute("UPDATE sticky_notes SET color = ?1, updated_at = datetime('now') WHERE id = ?2", params![clr, id])?;
        }

        let mut stmt = conn.prepare(
            "SELECT id, title, content, color, created_at, updated_at FROM sticky_notes WHERE id = ?1"
        )?;
        stmt.query_row(params![id], |row| {
            Ok(StickyNote {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                color: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })
    }

    pub fn delete_note(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute("DELETE FROM sticky_notes WHERE id = ?1", params![id])?;
        Ok(())
    }

    // --- Pomodoro ---

    pub fn get_pomodoro_settings(&self) -> Result<PomodoroSettings> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT OR IGNORE INTO pomodoro_settings (id) VALUES (1)",
            [],
        )?;
        let mut stmt = conn.prepare(
            "SELECT work_minutes, short_break_min, long_break_min, rounds_per_cycle, milestones_json
             FROM pomodoro_settings WHERE id = 1"
        )?;
        stmt.query_row([], |row| {
            let milestones_json: String = row.get(4)?;
            let milestones = serde_json::from_str::<Vec<PomodoroMilestone>>(&milestones_json)
                .unwrap_or_default();
            Ok(PomodoroSettings {
                work_minutes: row.get(0)?,
                short_break_min: row.get(1)?,
                long_break_min: row.get(2)?,
                rounds_per_cycle: row.get(3)?,
                milestones,
            })
        })
    }

    pub fn save_pomodoro_settings(&self, settings: &PomodoroSettings) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let milestones = settings
            .milestones
            .iter()
            .filter_map(|milestone| {
                let name = milestone.name.trim();
                let deadline = milestone.deadline.trim();
                if name.is_empty() || deadline.is_empty() {
                    return None;
                }

                Some(PomodoroMilestone {
                    name: name.to_string(),
                    deadline: deadline.to_string(),
                    status: match milestone.status.as_str() {
                        "completed" => "completed".to_string(),
                        "cancelled" => "cancelled".to_string(),
                        _ => "active".to_string(),
                    },
                })
            })
            .take(3)
            .collect::<Vec<_>>();
        let milestones_json =
            serde_json::to_string(&milestones).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        conn.execute(
            "INSERT INTO pomodoro_settings (id, work_minutes, short_break_min, long_break_min, rounds_per_cycle, milestones_json)
             VALUES (1, ?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
               work_minutes = excluded.work_minutes,
               short_break_min = excluded.short_break_min,
               long_break_min = excluded.long_break_min,
               rounds_per_cycle = excluded.rounds_per_cycle,
               milestones_json = excluded.milestones_json",
            params![
                settings.work_minutes,
                settings.short_break_min,
                settings.long_break_min,
                settings.rounds_per_cycle,
                milestones_json
            ],
        )?;
        Ok(())
    }

    pub fn record_pomodoro_session(&self, duration_min: i64) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT INTO pomodoro_sessions (duration_min) VALUES (?1)",
            params![duration_min],
        )?;
        Ok(())
    }

    pub fn get_today_pomodoro_count(&self) -> Result<i64> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            "SELECT COUNT(*) FROM pomodoro_sessions WHERE date(completed_at) = date('now')"
        )?;
        stmt.query_row([], |row| row.get(0))
    }

    pub fn get_weekly_pomodoro_stats(&self) -> Result<Vec<DayStat>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stats: Vec<DayStat> = Vec::with_capacity(7);
        for i in (0..7).rev() {
            let date_str: String = conn.query_row(
                &format!("SELECT date('now', '-{} days')", i),
                [],
                |row| row.get(0),
            )?;
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM pomodoro_sessions WHERE date(completed_at) = ?1",
                params![&date_str],
                |row| row.get(0),
            )?;
            stats.push(DayStat { date: date_str, count });
        }
        Ok(stats)
    }

    // --- App Settings ---

    pub fn get_app_settings(&self) -> Result<AppSettings> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT OR IGNORE INTO app_settings (id) VALUES (1)",
            [],
        )?;
        let mut stmt = conn.prepare(
            "SELECT page_size, todo_display, note_display, note_template, note_folder
             FROM app_settings WHERE id = 1"
        )?;
        stmt.query_row([], |row| {
            Ok(AppSettings {
                page_size: row.get(0)?,
                todo_display: row.get(1)?,
                note_display: row.get(2)?,
                note_template: row.get(3)?,
                note_folder: row.get(4)?,
            })
        })
    }

    pub fn save_app_settings(&self, s: &AppSettings) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT INTO app_settings (id, page_size, todo_display, note_display, note_template, note_folder)
             VALUES (1, ?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
               page_size = excluded.page_size,
               todo_display = excluded.todo_display,
               note_display = excluded.note_display,
               note_template = excluded.note_template,
               note_folder = excluded.note_folder",
            params![s.page_size, s.todo_display, s.note_display, s.note_template, s.note_folder],
        )?;
        Ok(())
    }
}
