use rusqlite::{Connection, Result, params};
use std::sync::Mutex;

use crate::models::todo::Todo;
use crate::models::note::StickyNote;
use crate::models::pomodoro::{PomodoroSettings, DayStat};

pub struct Database {
    conn: Mutex<Connection>,
    db_path: std::path::PathBuf,
}

impl Database {
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
                rounds_per_cycle INTEGER NOT NULL DEFAULT 4
            );
            CREATE TABLE IF NOT EXISTS pomodoro_sessions (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                completed_at TEXT NOT NULL DEFAULT (datetime('now')),
                duration_min INTEGER NOT NULL
            );"
        )?;

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
            "SELECT work_minutes, short_break_min, long_break_min, rounds_per_cycle FROM pomodoro_settings WHERE id = 1"
        )?;
        stmt.query_row([], |row| {
            Ok(PomodoroSettings {
                work_minutes: row.get(0)?,
                short_break_min: row.get(1)?,
                long_break_min: row.get(2)?,
                rounds_per_cycle: row.get(3)?,
            })
        })
    }

    pub fn save_pomodoro_settings(&self, settings: &PomodoroSettings) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT INTO pomodoro_settings (id, work_minutes, short_break_min, long_break_min, rounds_per_cycle)
             VALUES (1, ?1, ?2, ?3, ?4)
             ON CONFLICT(id) DO UPDATE SET
               work_minutes = excluded.work_minutes,
               short_break_min = excluded.short_break_min,
               long_break_min = excluded.long_break_min,
               rounds_per_cycle = excluded.rounds_per_cycle",
            params![settings.work_minutes, settings.short_break_min, settings.long_break_min, settings.rounds_per_cycle],
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
}
