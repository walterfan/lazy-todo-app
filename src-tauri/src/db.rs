use rusqlite::{Connection, Result, params};
use std::sync::Mutex;

use crate::models::todo::Todo;
use crate::models::note::StickyNote;
use crate::models::pomodoro::{PomodoroMilestone, PomodoroSettings, DayStat};
use crate::models::settings::AppSettings;
use crate::models::secretary::{
    SaveSecretaryMemory, SaveSecretaryPersona, SaveSecretaryProfile, SaveSecretaryReminder,
    SaveSecretarySettings, SecretaryConversation, SecretaryMemory, SecretaryMessage,
    SecretaryPersona, SecretaryProfile, SecretaryReminder, SecretarySettings, SecretarySkill,
};

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
            );
            CREATE TABLE IF NOT EXISTS secretary_settings (
                id                  INTEGER PRIMARY KEY CHECK (id = 1),
                base_url            TEXT NOT NULL DEFAULT '',
                model               TEXT NOT NULL DEFAULT '',
                api_key             TEXT NOT NULL DEFAULT '',
                skill_folder        TEXT NOT NULL DEFAULT '',
                conversation_folder TEXT NOT NULL DEFAULT '',
                active_persona_id   INTEGER,
                active_profile_id   INTEGER
            );
            CREATE TABLE IF NOT EXISTS secretary_personas (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                name        TEXT NOT NULL,
                voice       TEXT NOT NULL DEFAULT '',
                values_text TEXT NOT NULL DEFAULT '',
                style       TEXT NOT NULL DEFAULT '',
                boundaries  TEXT NOT NULL DEFAULT '',
                created_at  TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS secretary_profiles (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                name        TEXT NOT NULL,
                role        TEXT NOT NULL,
                domain      TEXT NOT NULL DEFAULT '',
                persona_id  INTEGER,
                skill_ids_json TEXT NOT NULL DEFAULT '[]',
                created_at  TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS secretary_skills (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                name       TEXT NOT NULL,
                summary    TEXT NOT NULL DEFAULT '',
                path       TEXT NOT NULL UNIQUE,
                content    TEXT NOT NULL DEFAULT '',
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS secretary_memories (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                content     TEXT NOT NULL,
                scope       TEXT NOT NULL DEFAULT 'global',
                domain      TEXT NOT NULL DEFAULT '',
                profile_id  INTEGER,
                status      TEXT NOT NULL DEFAULT 'active',
                pinned      INTEGER NOT NULL DEFAULT 0,
                source_conversation_id INTEGER,
                created_at  TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS secretary_reminders (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                title       TEXT NOT NULL,
                notes       TEXT NOT NULL DEFAULT '',
                due_at      TEXT NOT NULL,
                status      TEXT NOT NULL DEFAULT 'active',
                source_conversation_id INTEGER,
                created_at  TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS secretary_conversations (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                title           TEXT NOT NULL,
                profile_id      INTEGER,
                transcript_path TEXT NOT NULL DEFAULT '',
                messages_json   TEXT NOT NULL DEFAULT '[]',
                created_at      TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
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

    // --- Secretary ---

    pub fn ensure_default_secretary(&self) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute("INSERT OR IGNORE INTO secretary_settings (id) VALUES (1)", [])?;
        let persona_count: i64 = conn.query_row("SELECT COUNT(*) FROM secretary_personas", [], |row| row.get(0))?;
        if persona_count == 0 {
            conn.execute(
                "INSERT INTO secretary_personas (name, voice, values_text, style, boundaries)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    "Mira",
                    "Warm, direct, calm, and practical",
                    "Remember what matters, ask useful questions, protect the user's attention, and stay discreet",
                    "Concise personal secretary: suggest next steps, point out risks, and help keep work moving",
                    "Do not invent facts. Ask before saving memories, creating reminders, or changing notes."
                ],
            )?;
            let persona_id = conn.last_insert_rowid();
            conn.execute(
                "INSERT INTO secretary_profiles (name, role, domain, persona_id, skill_ids_json)
                 VALUES (?1, ?2, ?3, ?4, '[]')",
                params!["General Secretary", "question_answer", "Personal productivity", persona_id],
            )?;
            let profile_id = conn.last_insert_rowid();
            conn.execute(
                "UPDATE secretary_settings SET active_persona_id = ?1, active_profile_id = ?2 WHERE id = 1",
                params![persona_id, profile_id],
            )?;
        }
        Ok(())
    }

    pub fn get_secretary_settings(&self) -> Result<SecretarySettings> {
        self.ensure_default_secretary()?;
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.query_row(
            "SELECT base_url, model, api_key, skill_folder, conversation_folder, active_persona_id, active_profile_id
             FROM secretary_settings WHERE id = 1",
            [],
            |row| {
                let api_key: String = row.get(2)?;
                Ok(SecretarySettings {
                    base_url: row.get(0)?,
                    model: row.get(1)?,
                    has_saved_api_key: !api_key.is_empty(),
                    skill_folder: row.get(3)?,
                    conversation_folder: row.get(4)?,
                    active_persona_id: row.get(5)?,
                    active_profile_id: row.get(6)?,
                })
            },
        )
    }

    pub fn get_secretary_saved_api_key(&self) -> Result<String> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.query_row("SELECT api_key FROM secretary_settings WHERE id = 1", [], |row| row.get(0))
    }

    pub fn save_secretary_settings(&self, input: &SaveSecretarySettings) -> Result<SecretarySettings> {
        self.ensure_default_secretary()?;
        let conn = self.conn.lock().expect("db lock poisoned");
        if let Some(base_url) = &input.base_url {
            conn.execute("UPDATE secretary_settings SET base_url = ?1 WHERE id = 1", params![base_url.trim()])?;
        }
        if let Some(model) = &input.model {
            conn.execute("UPDATE secretary_settings SET model = ?1 WHERE id = 1", params![model.trim()])?;
        }
        if let Some(api_key) = &input.api_key {
            conn.execute("UPDATE secretary_settings SET api_key = ?1 WHERE id = 1", params![api_key.trim()])?;
        }
        if let Some(skill_folder) = &input.skill_folder {
            conn.execute("UPDATE secretary_settings SET skill_folder = ?1 WHERE id = 1", params![skill_folder.trim()])?;
        }
        if let Some(conversation_folder) = &input.conversation_folder {
            conn.execute(
                "UPDATE secretary_settings SET conversation_folder = ?1 WHERE id = 1",
                params![conversation_folder.trim()],
            )?;
        }
        conn.execute(
            "UPDATE secretary_settings SET active_persona_id = ?1, active_profile_id = ?2 WHERE id = 1",
            params![input.active_persona_id, input.active_profile_id],
        )?;
        drop(conn);
        self.get_secretary_settings()
    }

    pub fn list_secretary_personas(&self) -> Result<Vec<SecretaryPersona>> {
        self.ensure_default_secretary()?;
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, name, voice, values_text, style, boundaries, created_at, updated_at
             FROM secretary_personas ORDER BY updated_at DESC",
        )?;
        let personas = stmt.query_map([], |row| {
            Ok(SecretaryPersona {
                id: row.get(0)?,
                name: row.get(1)?,
                voice: row.get(2)?,
                values: row.get(3)?,
                style: row.get(4)?,
                boundaries: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(personas)
    }

    pub fn save_secretary_persona(&self, input: &SaveSecretaryPersona) -> Result<SecretaryPersona> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let id = if let Some(id) = input.id {
            conn.execute(
                "UPDATE secretary_personas
                 SET name = ?1, voice = ?2, values_text = ?3, style = ?4, boundaries = ?5, updated_at = datetime('now')
                 WHERE id = ?6",
                params![input.name.trim(), input.voice.trim(), input.values.trim(), input.style.trim(), input.boundaries.trim(), id],
            )?;
            id
        } else {
            conn.execute(
                "INSERT INTO secretary_personas (name, voice, values_text, style, boundaries)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![input.name.trim(), input.voice.trim(), input.values.trim(), input.style.trim(), input.boundaries.trim()],
            )?;
            conn.last_insert_rowid()
        };
        drop(conn);
        self.get_secretary_persona(id)
    }

    pub fn get_secretary_persona(&self, id: i64) -> Result<SecretaryPersona> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.query_row(
            "SELECT id, name, voice, values_text, style, boundaries, created_at, updated_at FROM secretary_personas WHERE id = ?1",
            params![id],
            |row| {
                Ok(SecretaryPersona {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    voice: row.get(2)?,
                    values: row.get(3)?,
                    style: row.get(4)?,
                    boundaries: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            },
        )
    }

    pub fn delete_secretary_persona(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute("DELETE FROM secretary_personas WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn list_secretary_profiles(&self) -> Result<Vec<SecretaryProfile>> {
        self.ensure_default_secretary()?;
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, name, role, domain, persona_id, skill_ids_json, created_at, updated_at
             FROM secretary_profiles ORDER BY updated_at DESC",
        )?;
        let profiles = stmt.query_map([], |row| {
            let skill_ids_json: String = row.get(5)?;
            let skill_ids = serde_json::from_str::<Vec<i64>>(&skill_ids_json).unwrap_or_default();
            Ok(SecretaryProfile {
                id: row.get(0)?,
                name: row.get(1)?,
                role: row.get(2)?,
                domain: row.get(3)?,
                persona_id: row.get(4)?,
                skill_ids,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(profiles)
    }

    pub fn get_secretary_profile(&self, id: i64) -> Result<SecretaryProfile> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.query_row(
            "SELECT id, name, role, domain, persona_id, skill_ids_json, created_at, updated_at FROM secretary_profiles WHERE id = ?1",
            params![id],
            |row| {
                let skill_ids_json: String = row.get(5)?;
                let skill_ids = serde_json::from_str::<Vec<i64>>(&skill_ids_json).unwrap_or_default();
                Ok(SecretaryProfile {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    role: row.get(2)?,
                    domain: row.get(3)?,
                    persona_id: row.get(4)?,
                    skill_ids,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            },
        )
    }

    pub fn save_secretary_profile(&self, input: &SaveSecretaryProfile) -> Result<SecretaryProfile> {
        let skill_ids_json =
            serde_json::to_string(&input.skill_ids).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let conn = self.conn.lock().expect("db lock poisoned");
        let id = if let Some(id) = input.id {
            conn.execute(
                "UPDATE secretary_profiles
                 SET name = ?1, role = ?2, domain = ?3, persona_id = ?4, skill_ids_json = ?5, updated_at = datetime('now')
                 WHERE id = ?6",
                params![input.name.trim(), input.role.trim(), input.domain.trim(), input.persona_id, skill_ids_json, id],
            )?;
            id
        } else {
            conn.execute(
                "INSERT INTO secretary_profiles (name, role, domain, persona_id, skill_ids_json)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![input.name.trim(), input.role.trim(), input.domain.trim(), input.persona_id, skill_ids_json],
            )?;
            conn.last_insert_rowid()
        };
        drop(conn);
        self.get_secretary_profile(id)
    }

    pub fn delete_secretary_profile(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute("DELETE FROM secretary_profiles WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn replace_secretary_skills(&self, skills: &[SecretarySkill]) -> Result<Vec<SecretarySkill>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        for skill in skills {
            conn.execute(
                "INSERT INTO secretary_skills (name, summary, path, content, updated_at)
                 VALUES (?1, ?2, ?3, ?4, datetime('now'))
                 ON CONFLICT(path) DO UPDATE SET
                   name = excluded.name,
                   summary = excluded.summary,
                   content = excluded.content,
                   updated_at = datetime('now')",
                params![skill.name, skill.summary, skill.path, skill.content],
            )?;
        }
        drop(conn);
        self.list_secretary_skills()
    }

    pub fn list_secretary_skills(&self) -> Result<Vec<SecretarySkill>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare("SELECT id, name, summary, path, content, updated_at FROM secretary_skills ORDER BY name ASC")?;
        let skills = stmt.query_map([], |row| {
            Ok(SecretarySkill {
                id: row.get(0)?,
                name: row.get(1)?,
                summary: row.get(2)?,
                path: row.get(3)?,
                content: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(skills)
    }

    pub fn list_secretary_memories(&self) -> Result<Vec<SecretaryMemory>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, content, scope, domain, profile_id, status, pinned, source_conversation_id, created_at, updated_at
             FROM secretary_memories ORDER BY pinned DESC, updated_at DESC",
        )?;
        let memories = stmt.query_map([], |row| {
            Ok(SecretaryMemory {
                id: row.get(0)?,
                content: row.get(1)?,
                scope: row.get(2)?,
                domain: row.get(3)?,
                profile_id: row.get(4)?,
                status: row.get(5)?,
                pinned: row.get::<_, i32>(6)? != 0,
                source_conversation_id: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(memories)
    }

    pub fn save_secretary_memory(&self, input: &SaveSecretaryMemory) -> Result<SecretaryMemory> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let id = if let Some(id) = input.id {
            conn.execute(
                "UPDATE secretary_memories
                 SET content = ?1, scope = ?2, domain = ?3, profile_id = ?4, status = ?5, pinned = ?6, updated_at = datetime('now')
                 WHERE id = ?7",
                params![
                    input.content.trim(),
                    input.scope.trim(),
                    input.domain.as_deref().unwrap_or("").trim(),
                    input.profile_id,
                    input.status.as_deref().unwrap_or("active"),
                    input.pinned.unwrap_or(false) as i32,
                    id,
                ],
            )?;
            id
        } else {
            conn.execute(
                "INSERT INTO secretary_memories (content, scope, domain, profile_id, status, pinned, source_conversation_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    input.content.trim(),
                    input.scope.trim(),
                    input.domain.as_deref().unwrap_or("").trim(),
                    input.profile_id,
                    input.status.as_deref().unwrap_or("active"),
                    input.pinned.unwrap_or(false) as i32,
                    input.source_conversation_id,
                ],
            )?;
            conn.last_insert_rowid()
        };
        drop(conn);
        self.get_secretary_memory(id)
    }

    pub fn get_secretary_memory(&self, id: i64) -> Result<SecretaryMemory> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.query_row(
            "SELECT id, content, scope, domain, profile_id, status, pinned, source_conversation_id, created_at, updated_at
             FROM secretary_memories WHERE id = ?1",
            params![id],
            |row| {
                Ok(SecretaryMemory {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    scope: row.get(2)?,
                    domain: row.get(3)?,
                    profile_id: row.get(4)?,
                    status: row.get(5)?,
                    pinned: row.get::<_, i32>(6)? != 0,
                    source_conversation_id: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            },
        )
    }

    pub fn delete_secretary_memory(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute("DELETE FROM secretary_memories WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn relevant_secretary_memories(&self, profile_id: Option<i64>, domain: &str, limit: usize) -> Result<Vec<SecretaryMemory>> {
        let memories = self.list_secretary_memories()?;
        let domain_lc = domain.to_lowercase();
        Ok(memories
            .into_iter()
            .filter(|memory| memory.status == "active")
            .filter(|memory| {
                memory.scope == "global"
                    || memory.profile_id == profile_id
                    || (!memory.domain.is_empty() && domain_lc.contains(&memory.domain.to_lowercase()))
            })
            .take(limit)
            .collect())
    }

    pub fn list_secretary_reminders(&self) -> Result<Vec<SecretaryReminder>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, title, notes, due_at, status, source_conversation_id, created_at, updated_at
             FROM secretary_reminders ORDER BY due_at ASC",
        )?;
        let reminders = stmt.query_map([], |row| {
            Ok(SecretaryReminder {
                id: row.get(0)?,
                title: row.get(1)?,
                notes: row.get(2)?,
                due_at: row.get(3)?,
                status: row.get(4)?,
                source_conversation_id: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(reminders)
    }

    pub fn save_secretary_reminder(&self, input: &SaveSecretaryReminder) -> Result<SecretaryReminder> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let id = if let Some(id) = input.id {
            conn.execute(
                "UPDATE secretary_reminders
                 SET title = ?1, notes = ?2, due_at = ?3, status = ?4, updated_at = datetime('now')
                 WHERE id = ?5",
                params![
                    input.title.trim(),
                    input.notes.as_deref().unwrap_or("").trim(),
                    input.due_at.trim(),
                    input.status.as_deref().unwrap_or("active"),
                    id,
                ],
            )?;
            id
        } else {
            conn.execute(
                "INSERT INTO secretary_reminders (title, notes, due_at, status, source_conversation_id)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    input.title.trim(),
                    input.notes.as_deref().unwrap_or("").trim(),
                    input.due_at.trim(),
                    input.status.as_deref().unwrap_or("active"),
                    input.source_conversation_id,
                ],
            )?;
            conn.last_insert_rowid()
        };
        drop(conn);
        self.get_secretary_reminder(id)
    }

    pub fn get_secretary_reminder(&self, id: i64) -> Result<SecretaryReminder> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.query_row(
            "SELECT id, title, notes, due_at, status, source_conversation_id, created_at, updated_at
             FROM secretary_reminders WHERE id = ?1",
            params![id],
            |row| {
                Ok(SecretaryReminder {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    notes: row.get(2)?,
                    due_at: row.get(3)?,
                    status: row.get(4)?,
                    source_conversation_id: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            },
        )
    }

    pub fn delete_secretary_reminder(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute("DELETE FROM secretary_reminders WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn due_secretary_reminders(&self) -> Result<Vec<SecretaryReminder>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, title, notes, due_at, status, source_conversation_id, created_at, updated_at
             FROM secretary_reminders
             WHERE status = 'active' AND due_at <= datetime('now')
             ORDER BY due_at ASC",
        )?;
        let reminders = stmt.query_map([], |row| {
            Ok(SecretaryReminder {
                id: row.get(0)?,
                title: row.get(1)?,
                notes: row.get(2)?,
                due_at: row.get(3)?,
                status: row.get(4)?,
                source_conversation_id: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(reminders)
    }

    pub fn save_secretary_conversation(&self, conversation: &SecretaryConversation) -> Result<SecretaryConversation> {
        let messages_json = serde_json::to_string(&conversation.messages)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let conn = self.conn.lock().expect("db lock poisoned");
        let id = if conversation.id > 0 {
            conn.execute(
                "UPDATE secretary_conversations
                 SET title = ?1, profile_id = ?2, transcript_path = ?3, messages_json = ?4, updated_at = datetime('now')
                 WHERE id = ?5",
                params![conversation.title, conversation.profile_id, conversation.transcript_path, messages_json, conversation.id],
            )?;
            conversation.id
        } else {
            conn.execute(
                "INSERT INTO secretary_conversations (title, profile_id, transcript_path, messages_json)
                 VALUES (?1, ?2, ?3, ?4)",
                params![conversation.title, conversation.profile_id, conversation.transcript_path, messages_json],
            )?;
            conn.last_insert_rowid()
        };
        drop(conn);
        self.get_secretary_conversation(id)
    }

    pub fn get_secretary_conversation(&self, id: i64) -> Result<SecretaryConversation> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.query_row(
            "SELECT id, title, profile_id, transcript_path, messages_json, created_at, updated_at
             FROM secretary_conversations WHERE id = ?1",
            params![id],
            |row| {
                let messages_json: String = row.get(4)?;
                let messages = serde_json::from_str::<Vec<SecretaryMessage>>(&messages_json).unwrap_or_default();
                Ok(SecretaryConversation {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    profile_id: row.get(2)?,
                    transcript_path: row.get(3)?,
                    messages,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            },
        )
    }

    pub fn list_secretary_conversations(&self) -> Result<Vec<SecretaryConversation>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, title, profile_id, transcript_path, messages_json, created_at, updated_at
             FROM secretary_conversations ORDER BY updated_at DESC",
        )?;
        let conversations = stmt.query_map([], |row| {
            let messages_json: String = row.get(4)?;
            let messages = serde_json::from_str::<Vec<SecretaryMessage>>(&messages_json).unwrap_or_default();
            Ok(SecretaryConversation {
                id: row.get(0)?,
                title: row.get(1)?,
                profile_id: row.get(2)?,
                transcript_path: row.get(3)?,
                messages,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(conversations)
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
