use chrono::{Datelike, Duration, Local, NaiveDate, NaiveDateTime, Timelike};
use rusqlite::types::ValueRef;
use rusqlite::{params, Connection, OpenFlags, Result, Row};
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;

use crate::models::agents::{
    AgentConversationSummary, AgentExternalCliCallResult, AgentExternalCliTool, AgentMemory,
    AgentMemoryProposal, AgentMessage, AgentMigrationStatus, AgentPlugin,
    AgentPluginDirectorySettings, AgentRagChunk, AgentSafeFileRootSettings, AgentSession,
    AgentToolAction, AgentUserIdentity, SaveAgentExternalCliTool, SaveAgentMemory,
    SaveAgentPluginDirectorySettings, SaveAgentSafeFileRootSettings, SaveAgentUserIdentity,
};
use crate::models::note::StickyNote;
use crate::models::pomodoro::{DayStat, PomodoroMilestone, PomodoroSettings};
use crate::models::secretary::{
    SaveSecretaryMemory, SaveSecretaryPersona, SaveSecretaryProfile, SaveSecretaryReminder,
    SaveSecretarySettings, SecretaryConversation, SecretaryMemory, SecretaryMessage,
    SecretaryPersona, SecretaryProfile, SecretaryReminder, SecretarySettings, SecretarySkill,
};
use crate::models::settings::AppSettings;
use crate::models::todo::Todo;
use crate::models::toolbox::DatabaseQueryResult;

pub struct Database {
    conn: Mutex<Connection>,
    db_path: std::path::PathBuf,
}

fn sql_value_to_display(value: ValueRef<'_>) -> String {
    match value {
        ValueRef::Null => "NULL".to_string(),
        ValueRef::Integer(value) => value.to_string(),
        ValueRef::Real(value) => value.to_string(),
        ValueRef::Text(value) => String::from_utf8_lossy(value).to_string(),
        ValueRef::Blob(value) => format!("<blob {} bytes>", value.len()),
    }
}

fn query_connection_readonly(
    conn: &Connection,
    sql: &str,
    max_rows: usize,
) -> std::result::Result<DatabaseQueryResult, String> {
    let started = Instant::now();
    let mut stmt = conn.prepare(sql).map_err(|error| error.to_string())?;
    if !stmt.readonly() {
        return Err("Only read-only SQL queries are allowed in Toolbox Database.".to_string());
    }

    let columns = stmt
        .column_names()
        .into_iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let column_count = stmt.column_count();
    let mut rows = stmt.query([]).map_err(|error| error.to_string())?;
    let mut result_rows = Vec::new();
    let mut truncated = false;

    while let Some(row) = rows.next().map_err(|error| error.to_string())? {
        if result_rows.len() >= max_rows {
            truncated = true;
            break;
        }
        let mut values = Vec::with_capacity(column_count);
        for index in 0..column_count {
            let value = row.get_ref(index).map_err(|error| error.to_string())?;
            values.push(sql_value_to_display(value));
        }
        result_rows.push(values);
    }

    Ok(DatabaseQueryResult {
        columns,
        row_count: result_rows.len(),
        rows: result_rows,
        truncated,
        elapsed_ms: started.elapsed().as_millis(),
    })
}

fn normalize_recurrence(value: Option<&str>) -> String {
    match value.unwrap_or("none").trim().to_ascii_lowercase().as_str() {
        "daily" => "daily".to_string(),
        "weekly" => "weekly".to_string(),
        "monthly" => "monthly".to_string(),
        "yearly" => "yearly".to_string(),
        _ => "none".to_string(),
    }
}

fn normalize_reminder_minutes(value: Option<i64>) -> Option<i64> {
    value.filter(|minutes| *minutes > 0)
}

fn normalize_recurrence_weekday(value: Option<i64>) -> Option<i64> {
    value.filter(|weekday| (1..=7).contains(weekday))
}

fn normalize_recurrence_month_day(value: Option<i64>) -> Option<i64> {
    value.filter(|day| (1..=31).contains(day))
}

fn parse_todo_datetime(value: &str) -> Option<NaiveDateTime> {
    let trimmed = value.trim();
    for format in [
        "%Y-%m-%dT%H:%M:%S%.f",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M",
        "%Y-%m-%d %H:%M:%S",
    ] {
        if let Ok(parsed) = NaiveDateTime::parse_from_str(trimmed, format) {
            return Some(parsed);
        }
    }
    None
}

fn format_todo_datetime(value: NaiveDateTime) -> String {
    value.format("%Y-%m-%dT%H:%M").to_string()
}

fn last_day_of_month(year: i32, month: u32) -> u32 {
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .expect("valid first day")
        .pred_opt()
        .expect("valid previous day")
        .day()
}

fn add_months_clamped(value: NaiveDateTime, months: i32, anchor_day: u32) -> NaiveDateTime {
    let month_index = value.year() * 12 + value.month0() as i32 + months;
    let year = month_index.div_euclid(12);
    let month0 = month_index.rem_euclid(12);
    let month = (month0 + 1) as u32;
    let day = anchor_day.min(last_day_of_month(year, month));
    NaiveDate::from_ymd_opt(year, month, day)
        .expect("valid clamped date")
        .and_hms_opt(value.hour(), value.minute(), value.second())
        .expect("valid original time")
}

fn next_weekday_after(value: NaiveDateTime, target_weekday: i64) -> NaiveDateTime {
    let current_weekday = i64::from(value.weekday().number_from_monday());
    let mut days = (target_weekday - current_weekday).rem_euclid(7);
    if days == 0 {
        days = 7;
    }
    value + Duration::days(days)
}

fn next_month_day_after(value: NaiveDateTime, target_day: i64) -> NaiveDateTime {
    let anchor_day = target_day as u32;
    let candidate = NaiveDate::from_ymd_opt(
        value.year(),
        value.month(),
        anchor_day.min(last_day_of_month(value.year(), value.month())),
    )
    .expect("valid clamped date")
    .and_hms_opt(value.hour(), value.minute(), value.second())
    .expect("valid original time");

    if candidate > value {
        candidate
    } else {
        add_months_clamped(value, 1, anchor_day)
    }
}

fn next_recurrence_deadline(
    current_deadline: &str,
    recurrence: &str,
    recurrence_anchor: Option<&str>,
    recurrence_weekday: Option<i64>,
    recurrence_month_day: Option<i64>,
) -> Option<String> {
    let current = parse_todo_datetime(current_deadline)?;
    let anchor_day = recurrence_anchor
        .and_then(parse_todo_datetime)
        .map(|anchor| anchor.day())
        .unwrap_or_else(|| current.day());
    let next = match recurrence {
        "daily" => current + Duration::days(1),
        "weekly" => recurrence_weekday
            .and_then(|weekday| normalize_recurrence_weekday(Some(weekday)))
            .map(|weekday| next_weekday_after(current, weekday))
            .unwrap_or_else(|| current + Duration::weeks(1)),
        "monthly" => recurrence_month_day
            .and_then(|day| normalize_recurrence_month_day(Some(day)))
            .map(|day| next_month_day_after(current, day))
            .unwrap_or_else(|| add_months_clamped(current, 1, anchor_day)),
        "yearly" => add_months_clamped(current, 12, anchor_day),
        _ => return None,
    };
    Some(format_todo_datetime(next))
}

fn reminder_state_for(
    completed: bool,
    deadline: Option<&str>,
    reminder_minutes_before: Option<i64>,
    last_reminded_deadline: Option<&str>,
    now: NaiveDateTime,
) -> (String, Option<String>) {
    if completed {
        return ("none".to_string(), None);
    }

    let Some(deadline_text) = deadline else {
        return ("none".to_string(), None);
    };
    let Some(deadline_at) = parse_todo_datetime(deadline_text) else {
        return ("none".to_string(), None);
    };

    if now > deadline_at {
        return ("overdue".to_string(), None);
    }

    let Some(minutes) = reminder_minutes_before else {
        return ("none".to_string(), None);
    };
    let reminder_due_at = deadline_at - Duration::minutes(minutes);
    let reminder_due_text = format_todo_datetime(reminder_due_at);
    if last_reminded_deadline == Some(deadline_text) {
        return ("reminded".to_string(), Some(reminder_due_text));
    }
    if now >= reminder_due_at {
        return ("due".to_string(), Some(reminder_due_text));
    }
    ("upcoming".to_string(), Some(reminder_due_text))
}

fn todo_from_row(row: &Row<'_>, now: NaiveDateTime) -> Result<Todo> {
    let completed = row.get::<_, i32>(4)? != 0;
    let deadline: Option<String> = row.get(5)?;
    let recurrence: String = row.get(7)?;
    let recurrence_anchor: Option<String> = row.get(8)?;
    let recurrence_weekday: Option<i64> = row.get(9)?;
    let recurrence_month_day: Option<i64> = row.get(10)?;
    let reminder_minutes_before: Option<i64> = row.get(11)?;
    let last_reminded_at: Option<String> = row.get(12)?;
    let last_reminded_deadline: Option<String> = row.get(13)?;
    let (reminder_state, reminder_due_at) = reminder_state_for(
        completed,
        deadline.as_deref(),
        reminder_minutes_before,
        last_reminded_deadline.as_deref(),
        now,
    );

    Ok(Todo {
        id: row.get(0)?,
        title: row.get(1)?,
        description: row.get(2)?,
        priority: row.get(3)?,
        completed,
        deadline,
        created_at: row.get(6)?,
        recurrence,
        recurrence_anchor,
        recurrence_weekday,
        recurrence_month_day,
        reminder_minutes_before,
        reminder_due_at,
        reminder_state,
        last_reminded_at,
        last_reminded_deadline,
    })
}

fn fetch_todo_by_id(conn: &Connection, id: i64) -> Result<Todo> {
    let now = Local::now().naive_local();
    let mut stmt = conn.prepare(
        "SELECT id, title, description, priority, completed, deadline, created_at,
                recurrence, recurrence_anchor, recurrence_weekday, recurrence_month_day,
                reminder_minutes_before,
                last_reminded_at, last_reminded_deadline
         FROM todos WHERE id = ?1",
    )?;
    stmt.query_row(params![id], |row| todo_from_row(row, now))
}

impl Database {
    fn ensure_todos_schema(conn: &Connection) -> Result<()> {
        let mut stmt = conn.prepare("PRAGMA table_info(todos)")?;
        let column_names = stmt
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<Vec<_>>>()?;

        if !column_names.iter().any(|name| name == "recurrence") {
            conn.execute(
                "ALTER TABLE todos ADD COLUMN recurrence TEXT NOT NULL DEFAULT 'none'",
                [],
            )?;
        }
        if !column_names.iter().any(|name| name == "recurrence_anchor") {
            conn.execute("ALTER TABLE todos ADD COLUMN recurrence_anchor TEXT", [])?;
        }
        if !column_names.iter().any(|name| name == "recurrence_weekday") {
            conn.execute(
                "ALTER TABLE todos ADD COLUMN recurrence_weekday INTEGER",
                [],
            )?;
        }
        if !column_names
            .iter()
            .any(|name| name == "recurrence_month_day")
        {
            conn.execute(
                "ALTER TABLE todos ADD COLUMN recurrence_month_day INTEGER",
                [],
            )?;
        }
        if !column_names
            .iter()
            .any(|name| name == "reminder_minutes_before")
        {
            conn.execute(
                "ALTER TABLE todos ADD COLUMN reminder_minutes_before INTEGER",
                [],
            )?;
        }
        if !column_names.iter().any(|name| name == "last_reminded_at") {
            conn.execute("ALTER TABLE todos ADD COLUMN last_reminded_at TEXT", [])?;
        }
        if !column_names
            .iter()
            .any(|name| name == "last_reminded_deadline")
        {
            conn.execute(
                "ALTER TABLE todos ADD COLUMN last_reminded_deadline TEXT",
                [],
            )?;
        }

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS todo_occurrences (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                todo_id      INTEGER NOT NULL,
                due_at       TEXT NOT NULL,
                completed_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY(todo_id) REFERENCES todos(id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_todo_occurrences_todo_id ON todo_occurrences(todo_id, due_at);",
        )?;

        Ok(())
    }

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

    fn ensure_sticky_notes_schema(conn: &Connection) -> Result<()> {
        let mut stmt = conn.prepare("PRAGMA table_info(sticky_notes)")?;
        let column_names = stmt
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<Vec<_>>>()?;

        if !column_names.iter().any(|name| name == "pinned") {
            conn.execute(
                "ALTER TABLE sticky_notes ADD COLUMN pinned INTEGER NOT NULL DEFAULT 0",
                [],
            )?;
        }

        Ok(())
    }

    fn ensure_agent_settings_schema(conn: &Connection) -> Result<()> {
        let mut stmt = conn.prepare("PRAGMA table_info(agent_settings)")?;
        let column_names = stmt
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<Vec<_>>>()?;

        if !column_names
            .iter()
            .any(|name| name == "safe_file_roots_json")
        {
            conn.execute(
                "ALTER TABLE agent_settings ADD COLUMN safe_file_roots_json TEXT NOT NULL DEFAULT '[]'",
                [],
            )?;
        }

        Ok(())
    }

    fn ensure_app_settings_schema(conn: &Connection) -> Result<()> {
        let mut stmt = conn.prepare("PRAGMA table_info(app_settings)")?;
        let column_names = stmt
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<Vec<_>>>()?;

        if !column_names.iter().any(|name| name == "language") {
            conn.execute(
                "ALTER TABLE app_settings ADD COLUMN language TEXT NOT NULL DEFAULT 'en'",
                [],
            )?;
        }
        if !column_names.iter().any(|name| name == "note_page_size") {
            conn.execute(
                "ALTER TABLE app_settings ADD COLUMN note_page_size INTEGER NOT NULL DEFAULT 10",
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
                pinned      INTEGER NOT NULL DEFAULT 0,
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
                page_size       INTEGER NOT NULL DEFAULT 10,
                note_page_size  INTEGER NOT NULL DEFAULT 10,
                todo_display    TEXT NOT NULL DEFAULT 'list',
                note_display    TEXT NOT NULL DEFAULT 'list',
                note_template   TEXT NOT NULL DEFAULT '',
                note_folder     TEXT NOT NULL DEFAULT '',
                language        TEXT NOT NULL DEFAULT 'en'
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
            );
            CREATE TABLE IF NOT EXISTS agent_settings (
                id               INTEGER PRIMARY KEY CHECK (id = 1),
                plugin_directory TEXT NOT NULL DEFAULT '',
                safe_file_roots_json TEXT NOT NULL DEFAULT '[]'
            );
            CREATE TABLE IF NOT EXISTS agent_plugins (
                plugin_id              TEXT PRIMARY KEY,
                plugin_name            TEXT NOT NULL,
                plugin_version         TEXT NOT NULL,
                author                 TEXT NOT NULL DEFAULT '',
                description            TEXT NOT NULL DEFAULT '',
                tags_json              TEXT NOT NULL DEFAULT '[]',
                path                   TEXT NOT NULL,
                avatar_path            TEXT NOT NULL DEFAULT '',
                readme_path            TEXT NOT NULL DEFAULT '',
                bundled                INTEGER NOT NULL DEFAULT 0,
                enabled                INTEGER NOT NULL DEFAULT 1,
                lifecycle_state        TEXT NOT NULL DEFAULT 'discovered',
                rag_enabled            INTEGER NOT NULL DEFAULT 0,
                is_multi_agent_supported INTEGER NOT NULL DEFAULT 0,
                has_rag_knowledge      INTEGER NOT NULL DEFAULT 0,
                validation_json        TEXT NOT NULL DEFAULT '[]',
                created_at             TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at             TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS agent_sessions (
                session_id     TEXT PRIMARY KEY,
                session_type   INTEGER NOT NULL,
                agent_ids_json TEXT NOT NULL DEFAULT '[]',
                session_title  TEXT NOT NULL DEFAULT '',
                memory_enabled INTEGER NOT NULL DEFAULT 1,
                created_at     TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at     TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS agent_session_participants (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id  TEXT NOT NULL,
                agent_id    TEXT NOT NULL,
                joined_at   TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS agent_messages (
                message_id       TEXT PRIMARY KEY,
                session_id       TEXT NOT NULL,
                sender_type      INTEGER NOT NULL,
                agent_id         TEXT,
                content          TEXT NOT NULL DEFAULT '',
                turn_index       INTEGER NOT NULL DEFAULT 0,
                stream_status    TEXT NOT NULL DEFAULT 'final',
                error_text       TEXT NOT NULL DEFAULT '',
                created_at       TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS agent_user_identity (
                id                    INTEGER PRIMARY KEY CHECK (id = 1),
                display_name          TEXT NOT NULL DEFAULT '',
                preferred_language    TEXT NOT NULL DEFAULT '',
                communication_style   TEXT NOT NULL DEFAULT '',
                roles_json            TEXT NOT NULL DEFAULT '[]',
                goals_json            TEXT NOT NULL DEFAULT '[]',
                boundaries            TEXT NOT NULL DEFAULT '',
                important_facts       TEXT NOT NULL DEFAULT '',
                enabled               INTEGER NOT NULL DEFAULT 1,
                updated_at            TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS agent_memories (
                memory_id      TEXT PRIMARY KEY,
                content        TEXT NOT NULL,
                scope          TEXT NOT NULL DEFAULT 'global',
                agent_id       TEXT,
                status         TEXT NOT NULL DEFAULT 'active',
                pinned         INTEGER NOT NULL DEFAULT 0,
                source_session_id TEXT,
                source_agent_id   TEXT,
                source_message_id TEXT,
                created_at     TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at     TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS agent_memory_proposals (
                proposal_id       TEXT PRIMARY KEY,
                source_session_id TEXT,
                source_agent_id   TEXT,
                source_message_id TEXT,
                proposed_text     TEXT NOT NULL,
                status            TEXT NOT NULL DEFAULT 'pending',
                created_at        TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at        TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS agent_conversation_summaries (
                summary_id   TEXT PRIMARY KEY,
                session_id   TEXT NOT NULL,
                agent_id     TEXT,
                title        TEXT NOT NULL DEFAULT '',
                summary      TEXT NOT NULL DEFAULT '',
                topics_json  TEXT NOT NULL DEFAULT '[]',
                created_at   TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at   TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS agent_memory_usage (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id  TEXT NOT NULL,
                message_id  TEXT,
                memory_id   TEXT,
                summary_id  TEXT,
                created_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS agent_memory_links (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                memory_id   TEXT NOT NULL,
                linked_type TEXT NOT NULL,
                linked_id   TEXT NOT NULL,
                created_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS agent_private_data (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                agent_id    TEXT NOT NULL,
                key         TEXT NOT NULL,
                value_json  TEXT NOT NULL DEFAULT '{}',
                created_at  TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(agent_id, key)
            );
            CREATE TABLE IF NOT EXISTS agent_external_cli_tools (
                tool_id               TEXT PRIMARY KEY,
                display_name          TEXT NOT NULL,
                executable            TEXT NOT NULL,
                allowed_subcommands_json TEXT NOT NULL DEFAULT '[]',
                argument_schema_json  TEXT NOT NULL DEFAULT '{}',
                working_directory     TEXT NOT NULL DEFAULT '',
                environment_json      TEXT NOT NULL DEFAULT '[]',
                timeout_ms            INTEGER NOT NULL DEFAULT 30000,
                output_limit          INTEGER NOT NULL DEFAULT 12000,
                safety_class          TEXT NOT NULL DEFAULT 'read',
                enabled               INTEGER NOT NULL DEFAULT 0,
                created_at            TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at            TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS agent_external_cli_permissions (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                tool_id     TEXT NOT NULL,
                scope_type  TEXT NOT NULL,
                scope_id    TEXT NOT NULL DEFAULT '',
                enabled     INTEGER NOT NULL DEFAULT 1,
                UNIQUE(tool_id, scope_type, scope_id)
            );
            CREATE TABLE IF NOT EXISTS agent_external_cli_audit (
                audit_id        TEXT PRIMARY KEY,
                session_id      TEXT,
                agent_id        TEXT,
                tool_id         TEXT NOT NULL,
                arguments_json  TEXT NOT NULL DEFAULT '{}',
                confirmation_status TEXT NOT NULL DEFAULT 'not_required',
                exit_code       INTEGER,
                stdout_text     TEXT NOT NULL DEFAULT '',
                stderr_text     TEXT NOT NULL DEFAULT '',
                duration_ms     INTEGER NOT NULL DEFAULT 0,
                timed_out       INTEGER NOT NULL DEFAULT 0,
                truncated       INTEGER NOT NULL DEFAULT 0,
                created_at      TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS agent_tool_actions (
                action_id      TEXT PRIMARY KEY,
                session_id     TEXT,
                agent_id       TEXT,
                tool_name      TEXT NOT NULL,
                arguments_json TEXT NOT NULL DEFAULT '{}',
                preview_json   TEXT NOT NULL DEFAULT '{}',
                status         TEXT NOT NULL DEFAULT 'pending',
                created_at     TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at     TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS agent_builtin_tool_audit (
                audit_id       TEXT PRIMARY KEY,
                session_id     TEXT,
                agent_id       TEXT,
                tool_name      TEXT NOT NULL,
                action_id      TEXT,
                arguments_json TEXT NOT NULL DEFAULT '{}',
                status         TEXT NOT NULL,
                result_json    TEXT NOT NULL DEFAULT '{}',
                created_at     TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS agent_rag_chunks (
                chunk_id        TEXT PRIMARY KEY,
                plugin_id       TEXT NOT NULL,
                plugin_version  TEXT NOT NULL,
                source_hash     TEXT NOT NULL,
                embedding_model TEXT NOT NULL DEFAULT '',
                embedding_dim   INTEGER NOT NULL DEFAULT 0,
                chunk_text      TEXT NOT NULL,
                embedding_json  TEXT NOT NULL DEFAULT '[]',
                created_at      TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS agent_migration_status (
                migration_id TEXT PRIMARY KEY,
                status       TEXT NOT NULL,
                details      TEXT NOT NULL DEFAULT '',
                created_at   TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at   TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_agent_plugins_state ON agent_plugins(lifecycle_state, enabled);
            CREATE INDEX IF NOT EXISTS idx_agent_sessions_updated ON agent_sessions(updated_at);
            CREATE INDEX IF NOT EXISTS idx_agent_messages_session ON agent_messages(session_id, turn_index, created_at);
            CREATE INDEX IF NOT EXISTS idx_agent_participants_session ON agent_session_participants(session_id, agent_id);
            CREATE INDEX IF NOT EXISTS idx_agent_memories_scope ON agent_memories(scope, agent_id, status, pinned);
            CREATE INDEX IF NOT EXISTS idx_agent_memory_usage_session ON agent_memory_usage(session_id, message_id);
            CREATE INDEX IF NOT EXISTS idx_agent_cli_audit_tool ON agent_external_cli_audit(tool_id, created_at);
            CREATE INDEX IF NOT EXISTS idx_agent_tool_actions_status ON agent_tool_actions(status, tool_name, created_at);
            CREATE INDEX IF NOT EXISTS idx_agent_builtin_tool_audit ON agent_builtin_tool_audit(tool_name, created_at);
            CREATE INDEX IF NOT EXISTS idx_agent_rag_plugin_hash ON agent_rag_chunks(plugin_id, source_hash);"
        )?;

        Self::ensure_todos_schema(&conn)?;
        Self::ensure_sticky_notes_schema(&conn)?;
        Self::ensure_pomodoro_settings_schema(&conn)?;
        Self::ensure_app_settings_schema(&conn)?;
        Self::ensure_agent_settings_schema(&conn)?;

        Ok(Self {
            conn: Mutex::new(conn),
            db_path,
        })
    }

    pub fn db_path(&self) -> String {
        self.db_path.to_string_lossy().to_string()
    }

    pub fn query_database_readonly(
        &self,
        sql: &str,
        max_rows: usize,
    ) -> std::result::Result<DatabaseQueryResult, String> {
        let conn = self.conn.lock().expect("db lock poisoned");
        query_connection_readonly(&conn, sql, max_rows)
    }

    pub fn query_database_file_readonly(
        path: &Path,
        sql: &str,
        max_rows: usize,
    ) -> std::result::Result<DatabaseQueryResult, String> {
        let metadata = std::fs::metadata(path).map_err(|error| {
            format!(
                "Cannot access SQLite database path {}: {error}",
                path.display()
            )
        })?;
        if !metadata.is_file() {
            return Err(format!(
                "SQLite database path is not a file: {}",
                path.display()
            ));
        }
        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|error| format!("Cannot open SQLite database {}: {error}", path.display()))?;
        query_connection_readonly(&conn, sql, max_rows)
    }

    pub fn get_agent_plugin_directory_settings(&self) -> Result<AgentPluginDirectorySettings> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute("INSERT OR IGNORE INTO agent_settings (id) VALUES (1)", [])?;
        conn.query_row(
            "SELECT plugin_directory FROM agent_settings WHERE id = 1",
            [],
            |row| {
                Ok(AgentPluginDirectorySettings {
                    plugin_directory: row.get(0)?,
                })
            },
        )
    }

    pub fn save_agent_plugin_directory_settings(
        &self,
        input: &SaveAgentPluginDirectorySettings,
    ) -> Result<AgentPluginDirectorySettings> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT INTO agent_settings (id, plugin_directory)
             VALUES (1, ?1)
             ON CONFLICT(id) DO UPDATE SET plugin_directory = excluded.plugin_directory",
            params![input.plugin_directory.trim()],
        )?;
        drop(conn);
        self.get_agent_plugin_directory_settings()
    }

    pub fn get_agent_safe_file_root_settings(&self) -> Result<AgentSafeFileRootSettings> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute("INSERT OR IGNORE INTO agent_settings (id) VALUES (1)", [])?;
        conn.query_row(
            "SELECT safe_file_roots_json FROM agent_settings WHERE id = 1",
            [],
            |row| {
                let roots_json: String = row.get(0)?;
                Ok(AgentSafeFileRootSettings {
                    safe_file_roots: serde_json::from_str(&roots_json).unwrap_or_default(),
                })
            },
        )
    }

    pub fn save_agent_safe_file_root_settings(
        &self,
        input: &SaveAgentSafeFileRootSettings,
    ) -> Result<AgentSafeFileRootSettings> {
        let roots_json = serde_json::to_string(&input.safe_file_roots)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute("INSERT OR IGNORE INTO agent_settings (id) VALUES (1)", [])?;
        conn.execute(
            "UPDATE agent_settings SET safe_file_roots_json = ?1 WHERE id = 1",
            params![roots_json],
        )?;
        drop(conn);
        self.get_agent_safe_file_root_settings()
    }

    pub fn list_agent_external_cli_tools(&self) -> Result<Vec<AgentExternalCliTool>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            "SELECT tool_id, display_name, executable, allowed_subcommands_json,
                    argument_schema_json, working_directory, environment_json, timeout_ms,
                    output_limit, safety_class, enabled, created_at, updated_at
             FROM agent_external_cli_tools
             ORDER BY display_name ASC, tool_id ASC",
        )?;
        let tools = stmt
            .query_map([], |row| {
                let allowed_subcommands_json: String = row.get(3)?;
                let argument_schema_json: String = row.get(4)?;
                let environment_json: String = row.get(6)?;
                Ok(AgentExternalCliTool {
                    tool_id: row.get(0)?,
                    display_name: row.get(1)?,
                    executable: row.get(2)?,
                    allowed_subcommands: serde_json::from_str(&allowed_subcommands_json)
                        .unwrap_or_default(),
                    argument_schema: serde_json::from_str(&argument_schema_json)
                        .unwrap_or_else(|_| serde_json::json!({})),
                    working_directory: row.get(5)?,
                    environment_allowlist: serde_json::from_str(&environment_json)
                        .unwrap_or_default(),
                    timeout_ms: row.get(7)?,
                    output_limit: row.get(8)?,
                    safety_class: row.get(9)?,
                    enabled: row.get::<_, i32>(10)? != 0,
                    available: true,
                    availability_error: String::new(),
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(tools)
    }

    pub fn get_agent_external_cli_tool(&self, tool_id: &str) -> Result<AgentExternalCliTool> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.query_row(
            "SELECT tool_id, display_name, executable, allowed_subcommands_json,
                    argument_schema_json, working_directory, environment_json, timeout_ms,
                    output_limit, safety_class, enabled, created_at, updated_at
             FROM agent_external_cli_tools
             WHERE tool_id = ?1",
            params![tool_id],
            |row| {
                let allowed_subcommands_json: String = row.get(3)?;
                let argument_schema_json: String = row.get(4)?;
                let environment_json: String = row.get(6)?;
                Ok(AgentExternalCliTool {
                    tool_id: row.get(0)?,
                    display_name: row.get(1)?,
                    executable: row.get(2)?,
                    allowed_subcommands: serde_json::from_str(&allowed_subcommands_json)
                        .unwrap_or_default(),
                    argument_schema: serde_json::from_str(&argument_schema_json)
                        .unwrap_or_else(|_| serde_json::json!({})),
                    working_directory: row.get(5)?,
                    environment_allowlist: serde_json::from_str(&environment_json)
                        .unwrap_or_default(),
                    timeout_ms: row.get(7)?,
                    output_limit: row.get(8)?,
                    safety_class: row.get(9)?,
                    enabled: row.get::<_, i32>(10)? != 0,
                    available: true,
                    availability_error: String::new(),
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                })
            },
        )
    }

    pub fn save_agent_external_cli_tool(
        &self,
        input: &SaveAgentExternalCliTool,
    ) -> Result<AgentExternalCliTool> {
        let tool_id = input
            .tool_id
            .as_deref()
            .unwrap_or_default()
            .trim()
            .to_string();
        let allowed_subcommands_json = serde_json::to_string(&input.allowed_subcommands)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let argument_schema_json = serde_json::to_string(&input.argument_schema)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let environment_json = serde_json::to_string(&input.environment_allowlist)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT INTO agent_external_cli_tools (
                tool_id, display_name, executable, allowed_subcommands_json,
                argument_schema_json, working_directory, environment_json, timeout_ms,
                output_limit, safety_class, enabled
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(tool_id) DO UPDATE SET
                display_name = excluded.display_name,
                executable = excluded.executable,
                allowed_subcommands_json = excluded.allowed_subcommands_json,
                argument_schema_json = excluded.argument_schema_json,
                working_directory = excluded.working_directory,
                environment_json = excluded.environment_json,
                timeout_ms = excluded.timeout_ms,
                output_limit = excluded.output_limit,
                safety_class = excluded.safety_class,
                enabled = excluded.enabled,
                updated_at = datetime('now')",
            params![
                tool_id,
                input.display_name.trim(),
                input.executable.trim(),
                allowed_subcommands_json,
                argument_schema_json,
                input.working_directory.trim(),
                environment_json,
                input.timeout_ms,
                input.output_limit,
                input.safety_class.trim(),
                input.enabled as i32,
            ],
        )?;
        drop(conn);
        self.get_agent_external_cli_tool(&tool_id)
    }

    pub fn set_agent_external_cli_tool_enabled(
        &self,
        tool_id: &str,
        enabled: bool,
    ) -> Result<AgentExternalCliTool> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "UPDATE agent_external_cli_tools
             SET enabled = ?1, updated_at = datetime('now')
             WHERE tool_id = ?2",
            params![enabled as i32, tool_id],
        )?;
        drop(conn);
        self.get_agent_external_cli_tool(tool_id)
    }

    pub fn delete_agent_external_cli_tool(&self, tool_id: &str) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "DELETE FROM agent_external_cli_tools WHERE tool_id = ?1",
            params![tool_id],
        )?;
        conn.execute(
            "DELETE FROM agent_external_cli_permissions WHERE tool_id = ?1",
            params![tool_id],
        )?;
        Ok(())
    }

    pub fn upsert_agent_plugin(&self, plugin: &AgentPlugin) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let tags_json = serde_json::to_string(&plugin.tags)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let validation_json = serde_json::to_string(&plugin.validation_diagnostics)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        conn.execute(
            "INSERT INTO agent_plugins (
                plugin_id, plugin_name, plugin_version, author, description, tags_json, path,
                avatar_path, readme_path, bundled, enabled, lifecycle_state, rag_enabled,
                is_multi_agent_supported, has_rag_knowledge, validation_json
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
             ON CONFLICT(plugin_id) DO UPDATE SET
                plugin_name = excluded.plugin_name,
                plugin_version = excluded.plugin_version,
                author = excluded.author,
                description = excluded.description,
                tags_json = excluded.tags_json,
                path = excluded.path,
                avatar_path = excluded.avatar_path,
                readme_path = excluded.readme_path,
                bundled = excluded.bundled,
                enabled = CASE
                    WHEN excluded.lifecycle_state = 'invalid' THEN 0
                    ELSE agent_plugins.enabled
                END,
                lifecycle_state = excluded.lifecycle_state,
                rag_enabled = excluded.rag_enabled,
                is_multi_agent_supported = excluded.is_multi_agent_supported,
                has_rag_knowledge = excluded.has_rag_knowledge,
                validation_json = excluded.validation_json,
                updated_at = datetime('now')",
            params![
                plugin.plugin_id,
                plugin.plugin_name,
                plugin.plugin_version,
                plugin.author,
                plugin.description,
                tags_json,
                plugin.path,
                plugin.avatar_path,
                plugin.readme_path,
                plugin.bundled as i32,
                plugin.enabled as i32,
                plugin.lifecycle_state,
                plugin.rag_enabled as i32,
                plugin.is_multi_agent_supported as i32,
                plugin.has_rag_knowledge as i32,
                validation_json,
            ],
        )?;
        Ok(())
    }

    pub fn set_agent_plugin_enabled(&self, plugin_id: &str, enabled: bool) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "UPDATE agent_plugins SET enabled = ?1, updated_at = datetime('now') WHERE plugin_id = ?2",
            params![enabled as i32, plugin_id],
        )?;
        Ok(())
    }

    pub fn mark_agent_plugin_uninstalled(&self, plugin_id: &str) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "UPDATE agent_plugins
             SET enabled = 0, lifecycle_state = 'uninstalled', updated_at = datetime('now')
             WHERE plugin_id = ?1",
            params![plugin_id],
        )?;
        Ok(())
    }

    pub fn list_agent_plugins(&self) -> Result<Vec<AgentPlugin>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            "SELECT plugin_id, plugin_name, plugin_version, author, description, tags_json, path,
                    avatar_path, readme_path, bundled, enabled, lifecycle_state, rag_enabled,
                    is_multi_agent_supported, has_rag_knowledge, validation_json
             FROM agent_plugins
             WHERE lifecycle_state != 'uninstalled'
             ORDER BY bundled DESC, plugin_name ASC",
        )?;
        let plugins = stmt
            .query_map([], |row| {
                let tags_json: String = row.get(5)?;
                let validation_json: String = row.get(15)?;
                Ok(AgentPlugin {
                    plugin_id: row.get(0)?,
                    plugin_name: row.get(1)?,
                    plugin_version: row.get(2)?,
                    author: row.get(3)?,
                    description: row.get(4)?,
                    tags: serde_json::from_str(&tags_json).unwrap_or_default(),
                    path: row.get(6)?,
                    avatar_path: row.get(7)?,
                    readme_path: row.get(8)?,
                    bundled: row.get::<_, i32>(9)? != 0,
                    enabled: row.get::<_, i32>(10)? != 0,
                    lifecycle_state: row.get(11)?,
                    rag_enabled: row.get::<_, i32>(12)? != 0,
                    is_multi_agent_supported: row.get::<_, i32>(13)? != 0,
                    has_rag_knowledge: row.get::<_, i32>(14)? != 0,
                    validation_diagnostics: serde_json::from_str(&validation_json)
                        .unwrap_or_default(),
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(plugins)
    }

    pub fn replace_agent_rag_chunks(
        &self,
        plugin_id: &str,
        chunks: &[AgentRagChunk],
    ) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "DELETE FROM agent_rag_chunks WHERE plugin_id = ?1",
            params![plugin_id],
        )?;
        for chunk in chunks {
            conn.execute(
                "INSERT INTO agent_rag_chunks (
                    chunk_id, plugin_id, plugin_version, source_hash, embedding_model,
                    embedding_dim, chunk_text, embedding_json
                 )
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, '[]')",
                params![
                    chunk.chunk_id,
                    chunk.plugin_id,
                    chunk.plugin_version,
                    chunk.source_hash,
                    chunk.embedding_model,
                    chunk.embedding_dim,
                    chunk.chunk_text,
                ],
            )?;
        }
        Ok(())
    }

    pub fn delete_agent_rag_chunks(&self, plugin_id: &str) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "DELETE FROM agent_rag_chunks WHERE plugin_id = ?1",
            params![plugin_id],
        )?;
        Ok(())
    }

    pub fn list_agent_rag_chunks(&self, plugin_id: &str) -> Result<Vec<AgentRagChunk>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            "SELECT chunk_id, plugin_id, plugin_version, source_hash, embedding_model,
                    embedding_dim, chunk_text, created_at
             FROM agent_rag_chunks
             WHERE plugin_id = ?1
             ORDER BY chunk_id ASC",
        )?;
        let chunks = stmt
            .query_map(params![plugin_id], |row| {
                Ok(AgentRagChunk {
                    chunk_id: row.get(0)?,
                    plugin_id: row.get(1)?,
                    plugin_version: row.get(2)?,
                    source_hash: row.get(3)?,
                    embedding_model: row.get(4)?,
                    embedding_dim: row.get(5)?,
                    chunk_text: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(chunks)
    }

    pub fn get_agent_migration_status(&self, migration_id: &str) -> Result<AgentMigrationStatus> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT OR IGNORE INTO agent_migration_status (migration_id, status, details)
             VALUES (?1, 'not_started', '')",
            params![migration_id],
        )?;
        conn.query_row(
            "SELECT migration_id, status, details, created_at, updated_at
             FROM agent_migration_status WHERE migration_id = ?1",
            params![migration_id],
            |row| {
                Ok(AgentMigrationStatus {
                    migration_id: row.get(0)?,
                    status: row.get(1)?,
                    details: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            },
        )
    }

    pub fn save_agent_migration_status(
        &self,
        migration_id: &str,
        status: &str,
        details: &str,
    ) -> Result<AgentMigrationStatus> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT INTO agent_migration_status (migration_id, status, details)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(migration_id) DO UPDATE SET
                status = excluded.status,
                details = excluded.details,
                updated_at = datetime('now')",
            params![migration_id, status, details],
        )?;
        drop(conn);
        self.get_agent_migration_status(migration_id)
    }

    pub fn save_agent_session(&self, session: &AgentSession) -> Result<AgentSession> {
        let agent_ids_json = serde_json::to_string(&session.agent_ids)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT INTO agent_sessions (
                session_id, session_type, agent_ids_json, session_title, memory_enabled
             )
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(session_id) DO UPDATE SET
                session_type = excluded.session_type,
                agent_ids_json = excluded.agent_ids_json,
                session_title = excluded.session_title,
                memory_enabled = excluded.memory_enabled,
                updated_at = datetime('now')",
            params![
                session.session_id,
                session.session_type,
                agent_ids_json,
                session.session_title,
                session.memory_enabled as i32,
            ],
        )?;
        for agent_id in &session.agent_ids {
            conn.execute(
                "INSERT OR IGNORE INTO agent_session_participants (session_id, agent_id)
                 VALUES (?1, ?2)",
                params![session.session_id, agent_id],
            )?;
        }
        drop(conn);
        self.get_agent_session(&session.session_id)
    }

    pub fn append_agent_message(&self, message: &AgentMessage) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT INTO agent_messages (
                message_id, session_id, sender_type, agent_id, content, turn_index, stream_status, error_text
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                message.message_id,
                message.session_id,
                message.sender_type,
                message.agent_id,
                message.content,
                message.turn_index,
                message.stream_status,
                message.error_text,
            ],
        )?;
        conn.execute(
            "UPDATE agent_sessions SET updated_at = datetime('now') WHERE session_id = ?1",
            params![message.session_id],
        )?;
        Ok(())
    }

    pub fn append_agent_message_if_missing(&self, message: &AgentMessage) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT OR IGNORE INTO agent_messages (
                message_id, session_id, sender_type, agent_id, content, turn_index, stream_status, error_text
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                message.message_id,
                message.session_id,
                message.sender_type,
                message.agent_id,
                message.content,
                message.turn_index,
                message.stream_status,
                message.error_text,
            ],
        )?;
        conn.execute(
            "UPDATE agent_sessions SET updated_at = datetime('now') WHERE session_id = ?1",
            params![message.session_id],
        )?;
        Ok(())
    }

    pub fn get_agent_message(&self, message_id: &str) -> Result<AgentMessage> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.query_row(
            "SELECT message_id, session_id, sender_type, agent_id, content, turn_index,
                    stream_status, error_text, created_at
             FROM agent_messages
             WHERE message_id = ?1",
            params![message_id],
            |row| {
                Ok(AgentMessage {
                    message_id: row.get(0)?,
                    session_id: row.get(1)?,
                    sender_type: row.get(2)?,
                    agent_id: row.get(3)?,
                    content: row.get(4)?,
                    turn_index: row.get(5)?,
                    stream_status: row.get(6)?,
                    error_text: row.get(7)?,
                    created_at: row.get(8)?,
                })
            },
        )
    }

    pub fn delete_agent_message(&self, message_id: &str) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let session_id: Option<String> = conn
            .query_row(
                "SELECT session_id FROM agent_messages WHERE message_id = ?1",
                params![message_id],
                |row| row.get(0),
            )
            .ok();
        conn.execute(
            "DELETE FROM agent_messages WHERE message_id = ?1",
            params![message_id],
        )?;
        if let Some(session_id) = session_id {
            conn.execute(
                "UPDATE agent_sessions SET updated_at = datetime('now') WHERE session_id = ?1",
                params![session_id],
            )?;
        }
        Ok(())
    }

    pub fn reset_agent_session(&self, session_id: &str) -> Result<AgentSession> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "DELETE FROM agent_messages WHERE session_id = ?1",
            params![session_id],
        )?;
        conn.execute(
            "DELETE FROM agent_conversation_summaries WHERE session_id = ?1",
            params![session_id],
        )?;
        conn.execute(
            "DELETE FROM agent_memory_usage WHERE session_id = ?1",
            params![session_id],
        )?;
        conn.execute(
            "DELETE FROM agent_tool_actions WHERE session_id = ?1",
            params![session_id],
        )?;
        conn.execute(
            "DELETE FROM agent_builtin_tool_audit WHERE session_id = ?1",
            params![session_id],
        )?;
        conn.execute(
            "DELETE FROM agent_external_cli_audit WHERE session_id = ?1",
            params![session_id],
        )?;
        conn.execute(
            "DELETE FROM agent_memory_proposals WHERE source_session_id = ?1",
            params![session_id],
        )?;
        conn.execute(
            "DELETE FROM agent_memory_links WHERE linked_type = 'session' AND linked_id = ?1",
            params![session_id],
        )?;
        conn.execute(
            "UPDATE agent_sessions SET updated_at = datetime('now') WHERE session_id = ?1",
            params![session_id],
        )?;
        drop(conn);
        self.get_agent_session(session_id)
    }

    pub fn delete_agent_session(&self, session_id: &str) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "DELETE FROM agent_messages WHERE session_id = ?1",
            params![session_id],
        )?;
        conn.execute(
            "DELETE FROM agent_session_participants WHERE session_id = ?1",
            params![session_id],
        )?;
        conn.execute(
            "DELETE FROM agent_conversation_summaries WHERE session_id = ?1",
            params![session_id],
        )?;
        conn.execute(
            "DELETE FROM agent_memory_usage WHERE session_id = ?1",
            params![session_id],
        )?;
        conn.execute(
            "DELETE FROM agent_tool_actions WHERE session_id = ?1",
            params![session_id],
        )?;
        conn.execute(
            "DELETE FROM agent_builtin_tool_audit WHERE session_id = ?1",
            params![session_id],
        )?;
        conn.execute(
            "DELETE FROM agent_external_cli_audit WHERE session_id = ?1",
            params![session_id],
        )?;
        conn.execute(
            "DELETE FROM agent_memory_proposals WHERE source_session_id = ?1",
            params![session_id],
        )?;
        conn.execute(
            "DELETE FROM agent_memory_links WHERE linked_type = 'session' AND linked_id = ?1",
            params![session_id],
        )?;
        conn.execute(
            "DELETE FROM agent_sessions WHERE session_id = ?1",
            params![session_id],
        )?;
        Ok(())
    }

    pub fn list_agent_sessions(&self) -> Result<Vec<AgentSession>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            "SELECT session_id, session_type, agent_ids_json, session_title, memory_enabled, created_at, updated_at
             FROM agent_sessions
             ORDER BY updated_at DESC",
        )?;
        let sessions = stmt
            .query_map([], |row| {
                let agent_ids_json: String = row.get(2)?;
                Ok(AgentSession {
                    session_id: row.get(0)?,
                    session_type: row.get(1)?,
                    agent_ids: serde_json::from_str(&agent_ids_json).unwrap_or_default(),
                    session_title: row.get(3)?,
                    memory_enabled: row.get::<_, i32>(4)? != 0,
                    messages: Vec::new(),
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(sessions)
    }

    pub fn get_agent_session(&self, session_id: &str) -> Result<AgentSession> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut session = conn.query_row(
            "SELECT session_id, session_type, agent_ids_json, session_title, memory_enabled, created_at, updated_at
             FROM agent_sessions WHERE session_id = ?1",
            params![session_id],
            |row| {
                let agent_ids_json: String = row.get(2)?;
                Ok(AgentSession {
                    session_id: row.get(0)?,
                    session_type: row.get(1)?,
                    agent_ids: serde_json::from_str(&agent_ids_json).unwrap_or_default(),
                    session_title: row.get(3)?,
                    memory_enabled: row.get::<_, i32>(4)? != 0,
                    messages: Vec::new(),
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            },
        )?;
        let mut stmt = conn.prepare(
            "SELECT message_id, session_id, sender_type, agent_id, content, turn_index,
                    stream_status, error_text, created_at
             FROM agent_messages
             WHERE session_id = ?1
             ORDER BY turn_index ASC, created_at ASC",
        )?;
        session.messages = stmt
            .query_map(params![session_id], |row| {
                Ok(AgentMessage {
                    message_id: row.get(0)?,
                    session_id: row.get(1)?,
                    sender_type: row.get(2)?,
                    agent_id: row.get(3)?,
                    content: row.get(4)?,
                    turn_index: row.get(5)?,
                    stream_status: row.get(6)?,
                    error_text: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(session)
    }

    pub fn get_agent_user_identity(&self) -> Result<AgentUserIdentity> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT OR IGNORE INTO agent_user_identity (id) VALUES (1)",
            [],
        )?;
        conn.query_row(
            "SELECT display_name, preferred_language, communication_style, roles_json,
                    goals_json, boundaries, important_facts, enabled, updated_at
             FROM agent_user_identity WHERE id = 1",
            [],
            |row| {
                let roles_json: String = row.get(3)?;
                let goals_json: String = row.get(4)?;
                Ok(AgentUserIdentity {
                    display_name: row.get(0)?,
                    preferred_language: row.get(1)?,
                    communication_style: row.get(2)?,
                    roles: serde_json::from_str(&roles_json).unwrap_or_default(),
                    goals: serde_json::from_str(&goals_json).unwrap_or_default(),
                    boundaries: row.get(5)?,
                    important_facts: row.get(6)?,
                    enabled: row.get::<_, i32>(7)? != 0,
                    updated_at: row.get(8)?,
                })
            },
        )
    }

    pub fn save_agent_user_identity(
        &self,
        input: &SaveAgentUserIdentity,
    ) -> Result<AgentUserIdentity> {
        let roles_json = serde_json::to_string(&input.roles)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let goals_json = serde_json::to_string(&input.goals)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT OR IGNORE INTO agent_user_identity (id) VALUES (1)",
            [],
        )?;
        conn.execute(
            "UPDATE agent_user_identity
             SET display_name = ?1,
                 preferred_language = ?2,
                 communication_style = ?3,
                 roles_json = ?4,
                 goals_json = ?5,
                 boundaries = ?6,
                 important_facts = ?7,
                 enabled = ?8,
                 updated_at = datetime('now')
             WHERE id = 1",
            params![
                input.display_name.trim(),
                input.preferred_language.trim(),
                input.communication_style.trim(),
                roles_json,
                goals_json,
                input.boundaries.trim(),
                input.important_facts.trim(),
                input.enabled as i32,
            ],
        )?;
        drop(conn);
        self.get_agent_user_identity()
    }

    pub fn list_agent_memories(&self, agent_id: Option<&str>) -> Result<Vec<AgentMemory>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let sql = if agent_id.is_some() {
            "SELECT memory_id, content, scope, agent_id, status, pinned, source_session_id,
                    source_agent_id, source_message_id, created_at, updated_at
             FROM agent_memories
             WHERE agent_id IS NULL OR agent_id = ?1
             ORDER BY pinned DESC, updated_at DESC"
        } else {
            "SELECT memory_id, content, scope, agent_id, status, pinned, source_session_id,
                    source_agent_id, source_message_id, created_at, updated_at
             FROM agent_memories
             ORDER BY pinned DESC, updated_at DESC"
        };
        let mut stmt = conn.prepare(sql)?;
        let map_row = |row: &rusqlite::Row<'_>| {
            Ok(AgentMemory {
                memory_id: row.get(0)?,
                content: row.get(1)?,
                scope: row.get(2)?,
                agent_id: row.get(3)?,
                status: row.get(4)?,
                pinned: row.get::<_, i32>(5)? != 0,
                source_session_id: row.get(6)?,
                source_agent_id: row.get(7)?,
                source_message_id: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        };
        if let Some(agent_id) = agent_id {
            stmt.query_map(params![agent_id], map_row)?
                .collect::<Result<Vec<_>>>()
        } else {
            stmt.query_map([], map_row)?.collect::<Result<Vec<_>>>()
        }
    }

    pub fn get_agent_memory(&self, memory_id: &str) -> Result<AgentMemory> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.query_row(
            "SELECT memory_id, content, scope, agent_id, status, pinned, source_session_id,
                    source_agent_id, source_message_id, created_at, updated_at
             FROM agent_memories WHERE memory_id = ?1",
            params![memory_id],
            |row| {
                Ok(AgentMemory {
                    memory_id: row.get(0)?,
                    content: row.get(1)?,
                    scope: row.get(2)?,
                    agent_id: row.get(3)?,
                    status: row.get(4)?,
                    pinned: row.get::<_, i32>(5)? != 0,
                    source_session_id: row.get(6)?,
                    source_agent_id: row.get(7)?,
                    source_message_id: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            },
        )
    }

    pub fn save_agent_memory(&self, input: &SaveAgentMemory) -> Result<AgentMemory> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let memory_id = input
            .memory_id
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| {
                format!(
                    "agent-memory-{}",
                    chrono::Local::now()
                        .timestamp_nanos_opt()
                        .unwrap_or_default()
                )
            });
        conn.execute(
            "INSERT INTO agent_memories (
                memory_id, content, scope, agent_id, status, pinned, source_session_id,
                source_agent_id, source_message_id
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(memory_id) DO UPDATE SET
                content = excluded.content,
                scope = excluded.scope,
                agent_id = excluded.agent_id,
                status = excluded.status,
                pinned = excluded.pinned,
                source_session_id = excluded.source_session_id,
                source_agent_id = excluded.source_agent_id,
                source_message_id = excluded.source_message_id,
                updated_at = datetime('now')",
            params![
                memory_id,
                input.content.trim(),
                input.scope.trim(),
                input
                    .agent_id
                    .as_deref()
                    .filter(|value| !value.trim().is_empty()),
                input.status.as_deref().unwrap_or("active"),
                input.pinned.unwrap_or(false) as i32,
                input.source_session_id.as_deref(),
                input.source_agent_id.as_deref(),
                input.source_message_id.as_deref(),
            ],
        )?;
        drop(conn);
        self.get_agent_memory(&memory_id)
    }

    pub fn delete_agent_memory(&self, memory_id: &str) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "DELETE FROM agent_memories WHERE memory_id = ?1",
            params![memory_id],
        )?;
        Ok(())
    }

    pub fn set_agent_memory_pinned(&self, memory_id: &str, pinned: bool) -> Result<AgentMemory> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "UPDATE agent_memories SET pinned = ?1, updated_at = datetime('now') WHERE memory_id = ?2",
            params![pinned as i32, memory_id],
        )?;
        drop(conn);
        self.get_agent_memory(memory_id)
    }

    pub fn set_agent_memory_status(&self, memory_id: &str, status: &str) -> Result<AgentMemory> {
        let status = match status {
            "active" | "archived" | "deleted" => status,
            _ => "active",
        };
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "UPDATE agent_memories SET status = ?1, updated_at = datetime('now') WHERE memory_id = ?2",
            params![status, memory_id],
        )?;
        drop(conn);
        self.get_agent_memory(memory_id)
    }

    pub fn save_agent_memory_proposal(
        &self,
        proposal: &AgentMemoryProposal,
    ) -> Result<AgentMemoryProposal> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT INTO agent_memory_proposals (
                proposal_id, source_session_id, source_agent_id, source_message_id,
                proposed_text, status
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(proposal_id) DO UPDATE SET
                source_session_id = excluded.source_session_id,
                source_agent_id = excluded.source_agent_id,
                source_message_id = excluded.source_message_id,
                proposed_text = excluded.proposed_text,
                status = excluded.status,
                updated_at = datetime('now')",
            params![
                proposal.proposal_id,
                proposal.source_session_id,
                proposal.source_agent_id,
                proposal.source_message_id,
                proposal.proposed_text,
                proposal.status,
            ],
        )?;
        drop(conn);
        self.get_agent_memory_proposal(&proposal.proposal_id)
    }

    pub fn get_agent_memory_proposal(&self, proposal_id: &str) -> Result<AgentMemoryProposal> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.query_row(
            "SELECT proposal_id, source_session_id, source_agent_id, source_message_id,
                    proposed_text, status, created_at, updated_at
             FROM agent_memory_proposals WHERE proposal_id = ?1",
            params![proposal_id],
            |row| {
                Ok(AgentMemoryProposal {
                    proposal_id: row.get(0)?,
                    source_session_id: row.get(1)?,
                    source_agent_id: row.get(2)?,
                    source_message_id: row.get(3)?,
                    proposed_text: row.get(4)?,
                    status: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            },
        )
    }

    pub fn list_agent_memory_proposals(&self) -> Result<Vec<AgentMemoryProposal>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            "SELECT proposal_id, source_session_id, source_agent_id, source_message_id,
                    proposed_text, status, created_at, updated_at
             FROM agent_memory_proposals
             WHERE status = 'pending'
             ORDER BY created_at DESC",
        )?;
        let proposals = stmt
            .query_map([], |row| {
                Ok(AgentMemoryProposal {
                    proposal_id: row.get(0)?,
                    source_session_id: row.get(1)?,
                    source_agent_id: row.get(2)?,
                    source_message_id: row.get(3)?,
                    proposed_text: row.get(4)?,
                    status: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(proposals)
    }

    pub fn relevant_agent_memories(
        &self,
        agent_id: &str,
        limit: usize,
    ) -> Result<Vec<AgentMemory>> {
        Ok(self
            .list_agent_memories(Some(agent_id))?
            .into_iter()
            .filter(|memory| memory.status == "active")
            .filter(|memory| {
                memory.scope == "global" || memory.agent_id.as_deref() == Some(agent_id)
            })
            .take(limit)
            .collect())
    }

    pub fn recent_agent_messages_for_context(
        &self,
        agent_id: &str,
        exclude_session_id: &str,
        limit: usize,
    ) -> Result<Vec<AgentMessage>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            "SELECT m.message_id, m.session_id, m.sender_type, m.agent_id, m.content, m.turn_index,
                    m.stream_status, m.error_text, m.created_at
             FROM agent_messages m
             INNER JOIN agent_session_participants p ON p.session_id = m.session_id
             WHERE p.agent_id = ?1 AND m.session_id != ?2 AND m.content != ''
             ORDER BY m.created_at DESC
             LIMIT ?3",
        )?;
        let mut messages = stmt
            .query_map(params![agent_id, exclude_session_id, limit as i64], |row| {
                Ok(AgentMessage {
                    message_id: row.get(0)?,
                    session_id: row.get(1)?,
                    sender_type: row.get(2)?,
                    agent_id: row.get(3)?,
                    content: row.get(4)?,
                    turn_index: row.get(5)?,
                    stream_status: row.get(6)?,
                    error_text: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        messages.reverse();
        Ok(messages)
    }

    pub fn save_agent_conversation_summary(
        &self,
        summary: &AgentConversationSummary,
    ) -> Result<AgentConversationSummary> {
        let topics_json = serde_json::to_string(&summary.topics)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT INTO agent_conversation_summaries (
                summary_id, session_id, agent_id, title, summary, topics_json
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(summary_id) DO UPDATE SET
                session_id = excluded.session_id,
                agent_id = excluded.agent_id,
                title = excluded.title,
                summary = excluded.summary,
                topics_json = excluded.topics_json,
                updated_at = datetime('now')",
            params![
                summary.summary_id,
                summary.session_id,
                summary.agent_id,
                summary.title,
                summary.summary,
                topics_json,
            ],
        )?;
        drop(conn);
        self.get_agent_conversation_summary(&summary.summary_id)
    }

    pub fn get_agent_conversation_summary(
        &self,
        summary_id: &str,
    ) -> Result<AgentConversationSummary> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.query_row(
            "SELECT summary_id, session_id, agent_id, title, summary, topics_json, created_at, updated_at
             FROM agent_conversation_summaries WHERE summary_id = ?1",
            params![summary_id],
            |row| {
                let topics_json: String = row.get(5)?;
                Ok(AgentConversationSummary {
                    summary_id: row.get(0)?,
                    session_id: row.get(1)?,
                    agent_id: row.get(2)?,
                    title: row.get(3)?,
                    summary: row.get(4)?,
                    topics: serde_json::from_str(&topics_json).unwrap_or_default(),
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            },
        )
    }

    pub fn relevant_agent_conversation_summaries(
        &self,
        agent_id: &str,
        exclude_session_id: &str,
        limit: usize,
    ) -> Result<Vec<AgentConversationSummary>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            "SELECT summary_id, session_id, agent_id, title, summary, topics_json, created_at, updated_at
             FROM agent_conversation_summaries
             WHERE session_id != ?1 AND (agent_id IS NULL OR agent_id = ?2)
             ORDER BY updated_at DESC
             LIMIT ?3",
        )?;
        let summaries = stmt
            .query_map(params![exclude_session_id, agent_id, limit as i64], |row| {
                let topics_json: String = row.get(5)?;
                Ok(AgentConversationSummary {
                    summary_id: row.get(0)?,
                    session_id: row.get(1)?,
                    agent_id: row.get(2)?,
                    title: row.get(3)?,
                    summary: row.get(4)?,
                    topics: serde_json::from_str(&topics_json).unwrap_or_default(),
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(summaries)
    }

    pub fn save_agent_tool_action(&self, action: &AgentToolAction) -> Result<AgentToolAction> {
        let arguments_json = serde_json::to_string(&action.arguments)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let preview_json = serde_json::to_string(&action.preview)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT INTO agent_tool_actions (
                action_id, session_id, agent_id, tool_name, arguments_json, preview_json, status
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(action_id) DO UPDATE SET
                session_id = excluded.session_id,
                agent_id = excluded.agent_id,
                tool_name = excluded.tool_name,
                arguments_json = excluded.arguments_json,
                preview_json = excluded.preview_json,
                status = excluded.status,
                updated_at = datetime('now')",
            params![
                action.action_id,
                action.session_id,
                action.agent_id,
                action.tool_name,
                arguments_json,
                preview_json,
                action.status,
            ],
        )?;
        drop(conn);
        self.get_agent_tool_action(&action.action_id)
    }

    pub fn get_agent_tool_action(&self, action_id: &str) -> Result<AgentToolAction> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.query_row(
            "SELECT action_id, session_id, agent_id, tool_name, arguments_json,
                    preview_json, status, created_at, updated_at
             FROM agent_tool_actions WHERE action_id = ?1",
            params![action_id],
            |row| {
                let arguments_json: String = row.get(4)?;
                let preview_json: String = row.get(5)?;
                Ok(AgentToolAction {
                    action_id: row.get(0)?,
                    session_id: row.get(1)?,
                    agent_id: row.get(2)?,
                    tool_name: row.get(3)?,
                    arguments: serde_json::from_str(&arguments_json).unwrap_or_default(),
                    preview: serde_json::from_str(&preview_json).unwrap_or_default(),
                    status: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            },
        )
    }

    pub fn set_agent_tool_action_status(
        &self,
        action_id: &str,
        status: &str,
    ) -> Result<AgentToolAction> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "UPDATE agent_tool_actions SET status = ?1, updated_at = datetime('now') WHERE action_id = ?2",
            params![status, action_id],
        )?;
        drop(conn);
        self.get_agent_tool_action(action_id)
    }

    pub fn list_pending_agent_tool_actions(&self) -> Result<Vec<AgentToolAction>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            "SELECT action_id, session_id, agent_id, tool_name, arguments_json,
                    preview_json, status, created_at, updated_at
             FROM agent_tool_actions
             WHERE status = 'pending'
             ORDER BY created_at DESC",
        )?;
        let actions = stmt
            .query_map([], |row| {
                let arguments_json: String = row.get(4)?;
                let preview_json: String = row.get(5)?;
                Ok(AgentToolAction {
                    action_id: row.get(0)?,
                    session_id: row.get(1)?,
                    agent_id: row.get(2)?,
                    tool_name: row.get(3)?,
                    arguments: serde_json::from_str(&arguments_json).unwrap_or_default(),
                    preview: serde_json::from_str(&preview_json).unwrap_or_default(),
                    status: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(actions)
    }

    pub fn list_agent_tool_actions_for_session(
        &self,
        session_id: &str,
    ) -> Result<Vec<AgentToolAction>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            "SELECT action_id, session_id, agent_id, tool_name, arguments_json,
                    preview_json, status, created_at, updated_at
             FROM agent_tool_actions
             WHERE session_id = ?1
             ORDER BY created_at ASC",
        )?;
        let actions = stmt
            .query_map(params![session_id], |row| {
                let arguments_json: String = row.get(4)?;
                let preview_json: String = row.get(5)?;
                Ok(AgentToolAction {
                    action_id: row.get(0)?,
                    session_id: row.get(1)?,
                    agent_id: row.get(2)?,
                    tool_name: row.get(3)?,
                    arguments: serde_json::from_str(&arguments_json).unwrap_or_default(),
                    preview: serde_json::from_str(&preview_json).unwrap_or_default(),
                    status: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(actions)
    }

    pub fn insert_agent_builtin_tool_audit(
        &self,
        audit_id: &str,
        session_id: Option<&str>,
        agent_id: Option<&str>,
        tool_name: &str,
        action_id: Option<&str>,
        arguments: &serde_json::Value,
        status: &str,
        result: &serde_json::Value,
    ) -> Result<()> {
        let arguments_json = serde_json::to_string(arguments)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let result_json = serde_json::to_string(result)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT INTO agent_builtin_tool_audit (
                audit_id, session_id, agent_id, tool_name, action_id,
                arguments_json, status, result_json
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                audit_id,
                session_id,
                agent_id,
                tool_name,
                action_id,
                arguments_json,
                status,
                result_json,
            ],
        )?;
        Ok(())
    }

    pub fn insert_agent_external_cli_audit(
        &self,
        result: &AgentExternalCliCallResult,
        session_id: Option<&str>,
        agent_id: Option<&str>,
        arguments: &serde_json::Value,
    ) -> Result<()> {
        let arguments_json = serde_json::to_string(arguments)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT INTO agent_external_cli_audit (
                audit_id, session_id, agent_id, tool_id, arguments_json, confirmation_status,
                exit_code, stdout_text, stderr_text, duration_ms, timed_out, truncated
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                result.audit_id,
                session_id,
                agent_id,
                result.tool_id,
                arguments_json,
                result.confirmation_status,
                result.exit_code,
                result.stdout,
                result.stderr,
                result.duration_ms,
                result.timed_out as i32,
                result.truncated as i32,
            ],
        )?;
        Ok(())
    }

    pub fn list_todos(&self) -> Result<Vec<Todo>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let now = Local::now().naive_local();
        let mut stmt = conn.prepare(
            "SELECT id, title, description, priority, completed, deadline, created_at,
                    recurrence, recurrence_anchor, recurrence_weekday, recurrence_month_day,
                    reminder_minutes_before,
                    last_reminded_at, last_reminded_deadline
             FROM todos ORDER BY completed ASC, priority ASC, deadline ASC NULLS LAST",
        )?;

        let todos = stmt
            .query_map([], |row| todo_from_row(row, now))?
            .collect::<Result<Vec<_>>>()?;

        Ok(todos)
    }

    pub fn add_todo(
        &self,
        title: &str,
        description: &str,
        priority: i32,
        deadline: Option<&str>,
        recurrence: Option<&str>,
        recurrence_weekday: Option<i64>,
        recurrence_month_day: Option<i64>,
        reminder_minutes_before: Option<i64>,
    ) -> Result<Todo> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut recurrence = normalize_recurrence(recurrence);
        if deadline.is_none() {
            recurrence = "none".to_string();
        }
        let recurrence_anchor = if recurrence == "none" { None } else { deadline };
        let recurrence_weekday = if recurrence == "weekly" {
            normalize_recurrence_weekday(recurrence_weekday)
        } else {
            None
        };
        let recurrence_month_day = if recurrence == "monthly" {
            normalize_recurrence_month_day(recurrence_month_day)
        } else {
            None
        };
        let reminder_minutes_before = if deadline.is_some() {
            normalize_reminder_minutes(reminder_minutes_before)
        } else {
            None
        };
        conn.execute(
            "INSERT INTO todos (
                title, description, priority, deadline, recurrence, recurrence_anchor,
                recurrence_weekday, recurrence_month_day, reminder_minutes_before
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                title,
                description,
                priority,
                deadline,
                recurrence,
                recurrence_anchor,
                recurrence_weekday,
                recurrence_month_day,
                reminder_minutes_before
            ],
        )?;
        let id = conn.last_insert_rowid();
        fetch_todo_by_id(&conn, id)
    }

    pub fn toggle_todo(&self, id: i64) -> Result<Todo> {
        let mut conn = self.conn.lock().expect("db lock poisoned");
        let current = fetch_todo_by_id(&conn, id)?;

        if !current.completed && current.recurrence != "none" {
            if let Some(deadline) = current.deadline.as_deref() {
                if let Some(next_deadline) = next_recurrence_deadline(
                    deadline,
                    &current.recurrence,
                    current.recurrence_anchor.as_deref(),
                    current.recurrence_weekday,
                    current.recurrence_month_day,
                ) {
                    let tx = conn.transaction()?;
                    tx.execute(
                        "INSERT INTO todo_occurrences (todo_id, due_at) VALUES (?1, ?2)",
                        params![id, deadline],
                    )?;
                    tx.execute(
                        "UPDATE todos
                         SET deadline = ?1,
                             completed = 0,
                             last_reminded_at = NULL,
                             last_reminded_deadline = NULL
                         WHERE id = ?2",
                        params![next_deadline, id],
                    )?;
                    tx.commit()?;
                    return fetch_todo_by_id(&conn, id);
                }
            }
        }

        conn.execute(
            "UPDATE todos
             SET completed = NOT completed,
                 last_reminded_at = CASE WHEN completed = 0 THEN NULL ELSE last_reminded_at END,
                 last_reminded_deadline = CASE WHEN completed = 0 THEN NULL ELSE last_reminded_deadline END
             WHERE id = ?1",
            params![id],
        )?;
        fetch_todo_by_id(&conn, id)
    }

    pub fn update_todo(
        &self,
        id: i64,
        title: Option<&str>,
        description: Option<&str>,
        priority: Option<i32>,
        deadline: Option<&str>,
        clear_deadline: bool,
        recurrence: Option<&str>,
        recurrence_weekday: Option<i64>,
        recurrence_month_day: Option<i64>,
        reminder_minutes_before: Option<i64>,
    ) -> Result<Todo> {
        let conn = self.conn.lock().expect("db lock poisoned");

        if let Some(t) = title {
            conn.execute("UPDATE todos SET title = ?1 WHERE id = ?2", params![t, id])?;
        }
        if let Some(d) = description {
            conn.execute(
                "UPDATE todos SET description = ?1 WHERE id = ?2",
                params![d, id],
            )?;
        }
        if let Some(p) = priority {
            conn.execute(
                "UPDATE todos SET priority = ?1 WHERE id = ?2",
                params![p, id],
            )?;
        }
        if let Some(dl) = deadline {
            conn.execute(
                "UPDATE todos
                 SET deadline = ?1,
                     last_reminded_at = NULL,
                     last_reminded_deadline = NULL
                 WHERE id = ?2",
                params![dl, id],
            )?;
        }
        if clear_deadline {
            conn.execute(
                "UPDATE todos
                 SET deadline = NULL,
                     recurrence = 'none',
                     recurrence_anchor = NULL,
                     recurrence_weekday = NULL,
                     recurrence_month_day = NULL,
                     reminder_minutes_before = NULL,
                     last_reminded_at = NULL,
                     last_reminded_deadline = NULL
                 WHERE id = ?1",
                params![id],
            )?;
        }
        if let Some(value) = recurrence {
            let recurrence = normalize_recurrence(Some(value));
            let current_deadline = deadline.map(ToString::to_string).or_else(|| {
                if clear_deadline {
                    None
                } else {
                    fetch_todo_by_id(&conn, id)
                        .ok()
                        .and_then(|todo| todo.deadline)
                }
            });
            let recurrence_anchor = if recurrence == "none" {
                None
            } else {
                current_deadline.as_deref()
            };
            let recurrence_weekday = if recurrence == "weekly" {
                normalize_recurrence_weekday(recurrence_weekday)
            } else {
                None
            };
            let recurrence_month_day = if recurrence == "monthly" {
                normalize_recurrence_month_day(recurrence_month_day)
            } else {
                None
            };
            conn.execute(
                "UPDATE todos
                 SET recurrence = ?1,
                     recurrence_anchor = ?2,
                     recurrence_weekday = ?3,
                     recurrence_month_day = ?4,
                     completed = CASE WHEN ?1 = 'none' THEN completed ELSE 0 END,
                     last_reminded_at = NULL,
                     last_reminded_deadline = NULL
                 WHERE id = ?5",
                params![
                    recurrence,
                    recurrence_anchor,
                    recurrence_weekday,
                    recurrence_month_day,
                    id
                ],
            )?;
        }
        if reminder_minutes_before.is_some() {
            let reminder_minutes_before = normalize_reminder_minutes(reminder_minutes_before);
            conn.execute(
                "UPDATE todos
                 SET reminder_minutes_before = ?1,
                     last_reminded_at = NULL,
                     last_reminded_deadline = NULL
                 WHERE id = ?2",
                params![reminder_minutes_before, id],
            )?;
        }

        fetch_todo_by_id(&conn, id)
    }

    pub fn delete_todo(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "DELETE FROM todo_occurrences WHERE todo_id = ?1",
            params![id],
        )?;
        conn.execute("DELETE FROM todos WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn list_due_todo_reminders(&self) -> Result<Vec<Todo>> {
        let todos = self.list_todos()?;
        Ok(todos
            .into_iter()
            .filter(|todo| todo.reminder_state == "due" || todo.reminder_state == "overdue")
            .filter(|todo| {
                todo.deadline.as_deref() != todo.last_reminded_deadline.as_deref()
                    && todo.reminder_minutes_before.is_some()
            })
            .collect())
    }

    pub fn mark_todo_reminded(&self, id: i64) -> Result<Todo> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let current = fetch_todo_by_id(&conn, id)?;
        if let Some(deadline) = current.deadline.as_deref() {
            let reminded_at = format_todo_datetime(Local::now().naive_local());
            conn.execute(
                "UPDATE todos
                 SET last_reminded_at = ?1,
                     last_reminded_deadline = ?2
                 WHERE id = ?3",
                params![reminded_at, deadline, id],
            )?;
        }
        fetch_todo_by_id(&conn, id)
    }

    // --- Sticky Notes ---

    pub fn list_notes(&self) -> Result<Vec<StickyNote>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, title, content, color, pinned, created_at, updated_at
             FROM sticky_notes ORDER BY pinned DESC, updated_at DESC",
        )?;

        let notes = stmt
            .query_map([], |row| {
                Ok(StickyNote {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    color: row.get(3)?,
                    pinned: row.get::<_, i32>(4)? != 0,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

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
            "SELECT id, title, content, color, pinned, created_at, updated_at FROM sticky_notes WHERE id = ?1"
        )?;
        stmt.query_row(params![id], |row| {
            Ok(StickyNote {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                color: row.get(3)?,
                pinned: row.get::<_, i32>(4)? != 0,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
    }

    pub fn update_note(
        &self,
        id: i64,
        title: Option<&str>,
        content: Option<&str>,
        color: Option<&str>,
    ) -> Result<StickyNote> {
        let conn = self.conn.lock().expect("db lock poisoned");

        if let Some(t) = title {
            conn.execute(
                "UPDATE sticky_notes SET title = ?1, updated_at = datetime('now') WHERE id = ?2",
                params![t, id],
            )?;
        }
        if let Some(c) = content {
            conn.execute(
                "UPDATE sticky_notes SET content = ?1, updated_at = datetime('now') WHERE id = ?2",
                params![c, id],
            )?;
        }
        if let Some(clr) = color {
            conn.execute(
                "UPDATE sticky_notes SET color = ?1, updated_at = datetime('now') WHERE id = ?2",
                params![clr, id],
            )?;
        }

        let mut stmt = conn.prepare(
            "SELECT id, title, content, color, pinned, created_at, updated_at FROM sticky_notes WHERE id = ?1"
        )?;
        stmt.query_row(params![id], |row| {
            Ok(StickyNote {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                color: row.get(3)?,
                pinned: row.get::<_, i32>(4)? != 0,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
    }

    pub fn set_note_pinned(&self, id: i64, pinned: bool) -> Result<StickyNote> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "UPDATE sticky_notes SET pinned = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![pinned as i32, id],
        )?;

        let mut stmt = conn.prepare(
            "SELECT id, title, content, color, pinned, created_at, updated_at FROM sticky_notes WHERE id = ?1"
        )?;
        stmt.query_row(params![id], |row| {
            Ok(StickyNote {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                color: row.get(3)?,
                pinned: row.get::<_, i32>(4)? != 0,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
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
        let milestones_json = serde_json::to_string(&milestones)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

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
            "SELECT COUNT(*) FROM pomodoro_sessions WHERE date(completed_at) = date('now')",
        )?;
        stmt.query_row([], |row| row.get(0))
    }

    pub fn get_weekly_pomodoro_stats(&self) -> Result<Vec<DayStat>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stats: Vec<DayStat> = Vec::with_capacity(7);
        for i in (0..7).rev() {
            let date_str: String =
                conn.query_row(&format!("SELECT date('now', '-{} days')", i), [], |row| {
                    row.get(0)
                })?;
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM pomodoro_sessions WHERE date(completed_at) = ?1",
                params![&date_str],
                |row| row.get(0),
            )?;
            stats.push(DayStat {
                date: date_str,
                count,
            });
        }
        Ok(stats)
    }

    // --- Secretary ---

    pub fn ensure_default_secretary(&self) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT OR IGNORE INTO secretary_settings (id) VALUES (1)",
            [],
        )?;
        let persona_count: i64 =
            conn.query_row("SELECT COUNT(*) FROM secretary_personas", [], |row| {
                row.get(0)
            })?;
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
                params![
                    "General Secretary",
                    "question_answer",
                    "Personal productivity",
                    persona_id
                ],
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
        conn.query_row(
            "SELECT api_key FROM secretary_settings WHERE id = 1",
            [],
            |row| row.get(0),
        )
    }

    pub fn save_secretary_settings(
        &self,
        input: &SaveSecretarySettings,
    ) -> Result<SecretarySettings> {
        self.ensure_default_secretary()?;
        let conn = self.conn.lock().expect("db lock poisoned");
        if let Some(base_url) = &input.base_url {
            conn.execute(
                "UPDATE secretary_settings SET base_url = ?1 WHERE id = 1",
                params![base_url.trim()],
            )?;
        }
        if let Some(model) = &input.model {
            conn.execute(
                "UPDATE secretary_settings SET model = ?1 WHERE id = 1",
                params![model.trim()],
            )?;
        }
        if let Some(api_key) = &input.api_key {
            conn.execute(
                "UPDATE secretary_settings SET api_key = ?1 WHERE id = 1",
                params![api_key.trim()],
            )?;
        }
        if let Some(skill_folder) = &input.skill_folder {
            conn.execute(
                "UPDATE secretary_settings SET skill_folder = ?1 WHERE id = 1",
                params![skill_folder.trim()],
            )?;
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
        let personas = stmt
            .query_map([], |row| {
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
            })?
            .collect::<Result<Vec<_>>>()?;
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
                params![
                    input.name.trim(),
                    input.voice.trim(),
                    input.values.trim(),
                    input.style.trim(),
                    input.boundaries.trim()
                ],
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
        let profiles = stmt
            .query_map([], |row| {
                let skill_ids_json: String = row.get(5)?;
                let skill_ids =
                    serde_json::from_str::<Vec<i64>>(&skill_ids_json).unwrap_or_default();
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
            })?
            .collect::<Result<Vec<_>>>()?;
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
        let skill_ids_json = serde_json::to_string(&input.skill_ids)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
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
                params![
                    input.name.trim(),
                    input.role.trim(),
                    input.domain.trim(),
                    input.persona_id,
                    skill_ids_json
                ],
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

    pub fn replace_secretary_skills(
        &self,
        skills: &[SecretarySkill],
    ) -> Result<Vec<SecretarySkill>> {
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
        let skills = stmt
            .query_map([], |row| {
                Ok(SecretarySkill {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    summary: row.get(2)?,
                    path: row.get(3)?,
                    content: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(skills)
    }

    pub fn list_secretary_memories(&self) -> Result<Vec<SecretaryMemory>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, content, scope, domain, profile_id, status, pinned, source_conversation_id, created_at, updated_at
             FROM secretary_memories ORDER BY pinned DESC, updated_at DESC",
        )?;
        let memories = stmt
            .query_map([], |row| {
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
            })?
            .collect::<Result<Vec<_>>>()?;
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

    pub fn relevant_secretary_memories(
        &self,
        profile_id: Option<i64>,
        domain: &str,
        limit: usize,
    ) -> Result<Vec<SecretaryMemory>> {
        let memories = self.list_secretary_memories()?;
        let domain_lc = domain.to_lowercase();
        Ok(memories
            .into_iter()
            .filter(|memory| memory.status == "active")
            .filter(|memory| {
                memory.scope == "global"
                    || memory.profile_id == profile_id
                    || (!memory.domain.is_empty()
                        && domain_lc.contains(&memory.domain.to_lowercase()))
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
        let reminders = stmt
            .query_map([], |row| {
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
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(reminders)
    }

    pub fn save_secretary_reminder(
        &self,
        input: &SaveSecretaryReminder,
    ) -> Result<SecretaryReminder> {
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
        let reminders = stmt
            .query_map([], |row| {
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
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(reminders)
    }

    pub fn save_secretary_conversation(
        &self,
        conversation: &SecretaryConversation,
    ) -> Result<SecretaryConversation> {
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
                let messages = serde_json::from_str::<Vec<SecretaryMessage>>(&messages_json)
                    .unwrap_or_default();
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
        let conversations = stmt
            .query_map([], |row| {
                let messages_json: String = row.get(4)?;
                let messages = serde_json::from_str::<Vec<SecretaryMessage>>(&messages_json)
                    .unwrap_or_default();
                Ok(SecretaryConversation {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    profile_id: row.get(2)?,
                    transcript_path: row.get(3)?,
                    messages,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(conversations)
    }

    // --- App Settings ---

    pub fn get_app_settings(&self) -> Result<AppSettings> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute("INSERT OR IGNORE INTO app_settings (id) VALUES (1)", [])?;
        let mut stmt = conn.prepare(
            "SELECT page_size, note_page_size, todo_display, note_display, note_template, note_folder, language
             FROM app_settings WHERE id = 1",
        )?;
        stmt.query_row([], |row| {
            Ok(AppSettings {
                page_size: row.get(0)?,
                note_page_size: row.get(1)?,
                todo_display: row.get(2)?,
                note_display: row.get(3)?,
                note_template: row.get(4)?,
                note_folder: row.get(5)?,
                language: row.get(6)?,
            })
        })
    }

    pub fn save_app_settings(&self, s: &AppSettings) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT INTO app_settings (id, page_size, note_page_size, todo_display, note_display, note_template, note_folder, language)
             VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
               page_size = excluded.page_size,
               note_page_size = excluded.note_page_size,
               todo_display = excluded.todo_display,
               note_display = excluded.note_display,
               note_template = excluded.note_template,
               note_folder = excluded.note_folder,
               language = excluded.language",
            params![
                s.page_size,
                s.note_page_size,
                s.todo_display,
                s.note_display,
                s.note_template,
                s.note_folder,
                if s.language == "zh" { "zh" } else { "en" },
            ],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_db_dir(name: &str) -> std::path::PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("lazy_todo_db_{name}_{suffix}"))
    }

    #[test]
    fn todo_migration_preserves_existing_rows_with_empty_recurrence_and_reminders() {
        let dir = temp_db_dir("todo_migration");
        std::fs::create_dir_all(&dir).expect("create temp db dir");
        {
            let conn = Connection::open(dir.join("todos.db")).expect("open old db");
            conn.execute_batch(
                "CREATE TABLE todos (
                    id          INTEGER PRIMARY KEY AUTOINCREMENT,
                    title       TEXT NOT NULL,
                    description TEXT NOT NULL DEFAULT '',
                    priority    INTEGER NOT NULL DEFAULT 2,
                    completed   INTEGER NOT NULL DEFAULT 0,
                    deadline    TEXT,
                    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
                );",
            )
            .expect("create old todos schema");
            conn.execute(
                "INSERT INTO todos (title, description, priority, deadline)
                 VALUES ('Legacy task', 'Keep me', 2, '2026-05-01T09:00')",
                [],
            )
            .expect("insert legacy todo");
        }

        let db = Database::new(&dir).expect("migrate db");
        let todos = db.list_todos().expect("list todos");
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].title, "Legacy task");
        assert_eq!(todos[0].recurrence, "none");
        assert_eq!(todos[0].recurrence_weekday, None);
        assert_eq!(todos[0].recurrence_month_day, None);
        assert_eq!(todos[0].reminder_minutes_before, None);
        assert_eq!(todos[0].reminder_state, "none");

        let conn = db.conn.lock().expect("db lock poisoned");
        let occurrence_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM todo_occurrences", [], |row| {
                row.get(0)
            })
            .expect("count occurrences");
        assert_eq!(occurrence_count, 0);
        drop(conn);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn recurrence_calculation_clamps_month_end_and_leap_day() {
        assert_eq!(
            next_recurrence_deadline(
                "2026-01-31T09:30",
                "monthly",
                Some("2026-01-31T09:30"),
                None,
                None
            ),
            Some("2026-02-28T09:30".to_string())
        );
        assert_eq!(
            next_recurrence_deadline(
                "2024-02-29T08:00",
                "yearly",
                Some("2024-02-29T08:00"),
                None,
                None
            ),
            Some("2025-02-28T08:00".to_string())
        );
    }

    #[test]
    fn recurrence_calculation_uses_explicit_weekday_and_month_day() {
        assert_eq!(
            next_recurrence_deadline("2026-05-01T09:30", "weekly", None, Some(1), None),
            Some("2026-05-04T09:30".to_string())
        );
        assert_eq!(
            next_recurrence_deadline("2026-01-15T09:30", "monthly", None, None, Some(31)),
            Some("2026-01-31T09:30".to_string())
        );
        assert_eq!(
            next_recurrence_deadline("2026-01-31T09:30", "monthly", None, None, Some(31)),
            Some("2026-02-28T09:30".to_string())
        );
        assert_eq!(
            next_recurrence_deadline("2026-02-28T09:30", "monthly", None, None, Some(31)),
            Some("2026-03-31T09:30".to_string())
        );
    }

    #[test]
    fn recurring_completion_records_occurrence_and_advances_deadline() {
        let dir = temp_db_dir("todo_recurring_completion");
        let db = Database::new(&dir).expect("create db");
        let todo = db
            .add_todo(
                "Daily standup",
                "",
                2,
                Some("2026-05-01T09:00"),
                Some("daily"),
                None,
                None,
                Some(15),
            )
            .expect("add todo");

        let advanced = db.toggle_todo(todo.id).expect("complete occurrence");
        assert!(!advanced.completed);
        assert_eq!(advanced.deadline.as_deref(), Some("2026-05-02T09:00"));
        assert_eq!(advanced.recurrence, "daily");
        assert_eq!(advanced.last_reminded_deadline, None);

        let conn = db.conn.lock().expect("db lock poisoned");
        let occurrence_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM todo_occurrences WHERE todo_id = ?1 AND due_at = ?2",
                params![todo.id, "2026-05-01T09:00"],
                |row| row.get(0),
            )
            .expect("count occurrences");
        assert_eq!(occurrence_count, 1);
        drop(conn);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn one_off_completion_and_recurrence_edits_remain_compatible() {
        let dir = temp_db_dir("todo_recurrence_edits");
        let db = Database::new(&dir).expect("create db");
        let one_off = db
            .add_todo(
                "One off",
                "",
                2,
                Some("2026-05-01T09:00"),
                None,
                None,
                None,
                None,
            )
            .expect("add one-off");
        let completed = db.toggle_todo(one_off.id).expect("complete one-off");
        assert!(completed.completed);
        assert_eq!(completed.deadline.as_deref(), Some("2026-05-01T09:00"));

        let recurring = db
            .add_todo(
                "Weekly report",
                "",
                2,
                Some("2026-05-01T09:00"),
                Some("weekly"),
                None,
                None,
                None,
            )
            .expect("add recurring");
        let monthly = db
            .update_todo(
                recurring.id,
                None,
                None,
                None,
                None,
                false,
                Some("monthly"),
                None,
                None,
                None,
            )
            .expect("change recurrence");
        assert_eq!(monthly.recurrence, "monthly");
        assert_eq!(
            monthly.recurrence_anchor.as_deref(),
            Some("2026-05-01T09:00")
        );

        let non_recurring = db
            .update_todo(
                recurring.id,
                None,
                None,
                None,
                None,
                false,
                Some("none"),
                None,
                None,
                None,
            )
            .expect("remove recurrence");
        assert_eq!(non_recurring.recurrence, "none");
        assert_eq!(non_recurring.recurrence_anchor, None);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn reminders_report_due_overdue_and_avoid_duplicates() {
        let dir = temp_db_dir("todo_reminders");
        let db = Database::new(&dir).expect("create db");
        let now = Local::now().naive_local();
        let future_deadline = format_todo_datetime(now + Duration::minutes(20));
        let overdue_deadline = format_todo_datetime(now - Duration::minutes(5));

        let due = db
            .add_todo(
                "Due reminder",
                "",
                2,
                Some(&future_deadline),
                None,
                None,
                None,
                Some(30),
            )
            .expect("add due reminder");
        let overdue = db
            .add_todo(
                "Overdue reminder",
                "",
                2,
                Some(&overdue_deadline),
                None,
                None,
                None,
                Some(10),
            )
            .expect("add overdue reminder");

        let reminders = db.list_due_todo_reminders().expect("list due reminders");
        assert!(reminders
            .iter()
            .any(|todo| todo.id == due.id && todo.reminder_state == "due"));
        assert!(reminders
            .iter()
            .any(|todo| todo.id == overdue.id && todo.reminder_state == "overdue"));

        let reminded = db.mark_todo_reminded(due.id).expect("mark reminded");
        assert_eq!(reminded.reminder_state, "reminded");
        let reminders = db
            .list_due_todo_reminders()
            .expect("list due reminders after mark");
        assert!(!reminders.iter().any(|todo| todo.id == due.id));

        db.delete_todo(overdue.id).expect("delete overdue");
        assert!(!db
            .list_due_todo_reminders()
            .expect("list reminders after delete")
            .iter()
            .any(|todo| todo.id == overdue.id));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn toolbox_database_query_returns_rows_and_blocks_writes() {
        let dir = temp_db_dir("toolbox_database_query");
        let db = Database::new(&dir).expect("create db");
        db.insert_note("Query me", "Database toolbox", "blue")
            .expect("insert note");

        let result = db
            .query_database_readonly("SELECT title, color FROM sticky_notes ORDER BY id DESC", 10)
            .expect("query notes");
        assert_eq!(result.columns, vec!["title", "color"]);
        assert_eq!(result.row_count, 1);
        assert_eq!(result.rows[0], vec!["Query me", "blue"]);
        assert!(!result.truncated);

        let file_result = Database::query_database_file_readonly(
            Path::new(&db.db_path()),
            "SELECT COUNT(*) FROM sticky_notes",
            10,
        )
        .expect("query notes by file path");
        assert_eq!(file_result.columns, vec!["COUNT(*)"]);
        assert_eq!(file_result.rows[0], vec!["1"]);

        let error = db
            .query_database_readonly("DELETE FROM sticky_notes", 10)
            .expect_err("write query should be rejected");
        assert!(error.contains("Only read-only SQL queries"));

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn sticky_notes_can_be_pinned_and_sort_first() {
        let dir = temp_db_dir("sticky_note_pinned");
        let db = Database::new(&dir).expect("create db");
        let first = db
            .insert_note("First", "Unpinned note.", "yellow")
            .expect("insert first");
        let second = db
            .insert_note("Second", "Pinned note.", "blue")
            .expect("insert second");

        let pinned = db
            .set_note_pinned(second.id, true)
            .expect("pin second note");
        assert!(pinned.pinned);

        let notes = db.list_notes().expect("list notes");
        assert_eq!(notes[0].id, second.id);
        assert!(notes[0].pinned);
        assert_eq!(notes[1].id, first.id);
        assert!(!notes[1].pinned);

        let unpinned = db
            .set_note_pinned(second.id, false)
            .expect("unpin second note");
        assert!(!unpinned.pinned);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn persists_agent_session_and_messages() {
        let dir = temp_db_dir("agent_session");
        let db = Database::new(&dir).expect("create db");
        let session = AgentSession {
            session_id: "session-test".to_string(),
            session_type: 1,
            agent_ids: vec!["secretary".to_string()],
            session_title: "Chat with Personal Secretary".to_string(),
            memory_enabled: true,
            messages: Vec::new(),
            created_at: String::new(),
            updated_at: String::new(),
        };

        let saved = db.save_agent_session(&session).expect("save session");
        db.append_agent_message(&AgentMessage {
            message_id: "message-user".to_string(),
            session_id: saved.session_id.clone(),
            sender_type: 1,
            agent_id: None,
            content: "hello".to_string(),
            turn_index: 1,
            stream_status: "final".to_string(),
            error_text: String::new(),
            created_at: String::new(),
        })
        .expect("append user message");
        db.append_agent_message(&AgentMessage {
            message_id: "message-agent".to_string(),
            session_id: saved.session_id.clone(),
            sender_type: 2,
            agent_id: Some("secretary".to_string()),
            content: "hello, Walter".to_string(),
            turn_index: 1,
            stream_status: "final".to_string(),
            error_text: String::new(),
            created_at: String::new(),
        })
        .expect("append agent message");
        db.append_agent_message(&AgentMessage {
            message_id: "message-error".to_string(),
            session_id: saved.session_id.clone(),
            sender_type: 2,
            agent_id: Some("secretary".to_string()),
            content: String::new(),
            turn_index: 2,
            stream_status: "error".to_string(),
            error_text: "stream failed".to_string(),
            created_at: String::new(),
        })
        .expect("append error message");

        let loaded = db.get_agent_session("session-test").expect("load session");
        assert_eq!(loaded.agent_ids, vec!["secretary"]);
        assert_eq!(loaded.messages.len(), 3);
        assert_eq!(loaded.messages[0].content, "hello");
        assert_eq!(loaded.messages[1].agent_id.as_deref(), Some("secretary"));
        assert_eq!(loaded.messages[2].stream_status, "error");
        assert_eq!(loaded.messages[2].error_text, "stream failed");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn resets_and_deletes_agent_sessions_with_related_session_data() {
        let dir = temp_db_dir("agent_session_lifecycle");
        let db = Database::new(&dir).expect("create db");
        let session = db
            .save_agent_session(&AgentSession {
                session_id: "session-lifecycle".to_string(),
                session_type: 1,
                agent_ids: vec!["secretary".to_string()],
                session_title: "Lifecycle".to_string(),
                memory_enabled: true,
                messages: Vec::new(),
                created_at: String::new(),
                updated_at: String::new(),
            })
            .expect("save session");
        db.append_agent_message(&AgentMessage {
            message_id: "message-lifecycle".to_string(),
            session_id: session.session_id.clone(),
            sender_type: 2,
            agent_id: Some("secretary".to_string()),
            content: "hello".to_string(),
            turn_index: 1,
            stream_status: "final".to_string(),
            error_text: String::new(),
            created_at: String::new(),
        })
        .expect("append message");
        db.save_agent_conversation_summary(&AgentConversationSummary {
            summary_id: "summary-lifecycle".to_string(),
            session_id: session.session_id.clone(),
            agent_id: Some("secretary".to_string()),
            title: "summary".to_string(),
            summary: "A short summary.".to_string(),
            topics: vec!["testing".to_string()],
            created_at: String::new(),
            updated_at: String::new(),
        })
        .expect("save summary");
        db.save_agent_tool_action(&AgentToolAction {
            action_id: "action-lifecycle".to_string(),
            session_id: Some(session.session_id.clone()),
            agent_id: Some("secretary".to_string()),
            tool_name: "read_note".to_string(),
            arguments: serde_json::json!({}),
            preview: serde_json::json!({}),
            status: "pending".to_string(),
            created_at: String::new(),
            updated_at: String::new(),
        })
        .expect("save action");
        db.save_agent_memory_proposal(&AgentMemoryProposal {
            proposal_id: "proposal-lifecycle".to_string(),
            source_session_id: Some(session.session_id.clone()),
            source_agent_id: Some("secretary".to_string()),
            source_message_id: Some("message-lifecycle".to_string()),
            proposed_text: "Remember this.".to_string(),
            status: "pending".to_string(),
            created_at: String::new(),
            updated_at: String::new(),
        })
        .expect("save proposal");
        db.insert_agent_builtin_tool_audit(
            "audit-lifecycle",
            Some(&session.session_id),
            Some("secretary"),
            "read_note",
            None,
            &serde_json::json!({}),
            "completed",
            &serde_json::json!({}),
        )
        .expect("insert audit");

        let reset = db
            .reset_agent_session(&session.session_id)
            .expect("reset session");
        assert!(reset.messages.is_empty());
        {
            let conn = db.conn.lock().expect("db lock poisoned");
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM agent_tool_actions WHERE session_id = ?1",
                    params![&session.session_id],
                    |row| row.get(0),
                )
                .expect("count actions");
            assert_eq!(count, 0);
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM agent_memory_proposals WHERE source_session_id = ?1",
                    params![&session.session_id],
                    |row| row.get(0),
                )
                .expect("count proposals");
            assert_eq!(count, 0);
        }

        db.delete_agent_session(&session.session_id)
            .expect("delete session");
        assert!(db.get_agent_session(&session.session_id).is_err());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn persists_agent_identity_and_memory() {
        let dir = temp_db_dir("agent_memory");
        let db = Database::new(&dir).expect("create db");

        let identity = db
            .save_agent_user_identity(&SaveAgentUserIdentity {
                display_name: "Walter".to_string(),
                preferred_language: "zh-CN".to_string(),
                communication_style: "direct and practical".to_string(),
                roles: vec!["builder".to_string()],
                goals: vec!["ship the agents module".to_string()],
                boundaries: "Ask before changing local data.".to_string(),
                important_facts: "Prefers local-first tools.".to_string(),
                enabled: true,
            })
            .expect("save identity");
        assert_eq!(identity.display_name, "Walter");
        assert_eq!(identity.roles, vec!["builder"]);

        let global = db
            .save_agent_memory(&SaveAgentMemory {
                memory_id: Some("memory-global".to_string()),
                content: "Walter wants Agents to have soul and memory.".to_string(),
                scope: "global".to_string(),
                agent_id: None,
                status: Some("active".to_string()),
                pinned: Some(true),
                source_session_id: None,
                source_agent_id: None,
                source_message_id: None,
            })
            .expect("save global memory");
        let scoped = db
            .save_agent_memory(&SaveAgentMemory {
                memory_id: Some("memory-secretary".to_string()),
                content: "Secretary should be proactive.".to_string(),
                scope: "agent".to_string(),
                agent_id: Some("secretary".to_string()),
                status: Some("active".to_string()),
                pinned: Some(false),
                source_session_id: None,
                source_agent_id: None,
                source_message_id: None,
            })
            .expect("save scoped memory");

        let memories = db
            .relevant_agent_memories("secretary", 10)
            .expect("list relevant");
        assert_eq!(memories.len(), 2);
        assert_eq!(memories[0].memory_id, global.memory_id);
        assert!(memories
            .iter()
            .any(|memory| memory.memory_id == scoped.memory_id));

        let archived = db
            .set_agent_memory_status("memory-secretary", "archived")
            .expect("archive memory");
        assert_eq!(archived.status, "archived");
        let memories = db
            .relevant_agent_memories("secretary", 10)
            .expect("list relevant");
        assert_eq!(memories.len(), 1);
        assert_eq!(memories[0].memory_id, "memory-global");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn finds_recent_agent_messages_for_previous_context() {
        let dir = temp_db_dir("agent_previous_context");
        let db = Database::new(&dir).expect("create db");
        for session_id in ["old-session", "current-session"] {
            db.save_agent_session(&AgentSession {
                session_id: session_id.to_string(),
                session_type: 1,
                agent_ids: vec!["secretary".to_string()],
                session_title: session_id.to_string(),
                memory_enabled: true,
                messages: Vec::new(),
                created_at: String::new(),
                updated_at: String::new(),
            })
            .expect("save session");
        }
        db.append_agent_message(&AgentMessage {
            message_id: "previous-message".to_string(),
            session_id: "old-session".to_string(),
            sender_type: 1,
            agent_id: None,
            content: "remember my launch checklist".to_string(),
            turn_index: 1,
            stream_status: "final".to_string(),
            error_text: String::new(),
            created_at: String::new(),
        })
        .expect("append previous");
        db.append_agent_message(&AgentMessage {
            message_id: "current-message".to_string(),
            session_id: "current-session".to_string(),
            sender_type: 1,
            agent_id: None,
            content: "do not include this".to_string(),
            turn_index: 1,
            stream_status: "final".to_string(),
            error_text: String::new(),
            created_at: String::new(),
        })
        .expect("append current");

        let messages = db
            .recent_agent_messages_for_context("secretary", "current-session", 4)
            .expect("previous messages");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].message_id, "previous-message");
        let _ = std::fs::remove_dir_all(dir);
    }
}
