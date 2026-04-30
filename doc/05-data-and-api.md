# Lazy Todo App — Data & API Reference

<!-- maintained-by: human+ai -->

## Database

- **Engine**: SQLite via `rusqlite` with the `bundled` feature
- **File**: `todos.db`
- **Schema creation**: `db.rs::Database::new()` initializes all tables on startup
- **Configuration precedence**: `LAZY_TODO_DB_DIR` -> `~/.config/lazy-todo-app/config.json` -> Tauri app data dir

### Table: `todos`

| Column | Type | Default | Notes |
|--------|------|---------|-------|
| `id` | INTEGER | AUTOINCREMENT | Primary key |
| `title` | TEXT | — | Required |
| `description` | TEXT | `''` | Required |
| `priority` | INTEGER | `2` | `1=high`, `2=medium`, `3=low` |
| `completed` | INTEGER | `0` | Boolean flag |
| `deadline` | TEXT | NULL | ISO 8601 datetime |
| `recurrence` | TEXT | `'none'` | `none`, `daily`, `weekly`, `monthly`, or `yearly` |
| `recurrence_anchor` | TEXT | NULL | Original due timestamp used for calendar-safe monthly/yearly recurrence |
| `reminder_minutes_before` | INTEGER | NULL | Lead time before `deadline` for local reminders |
| `last_reminded_at` | TEXT | NULL | Last local reminder delivery attempt |
| `last_reminded_deadline` | TEXT | NULL | Deadline occurrence that was last reminded |
| `created_at` | TEXT | `datetime('now')` | UTC timestamp |

### Table: `todo_occurrences`

Completion history for recurring todo occurrences.

| Column | Type | Default | Notes |
|--------|------|---------|-------|
| `id` | INTEGER | AUTOINCREMENT | Primary key |
| `todo_id` | INTEGER | — | Parent todo row |
| `due_at` | TEXT | — | Due timestamp that was completed |
| `completed_at` | TEXT | `datetime('now')` | Completion timestamp |

### Table: `sticky_notes`

| Column | Type | Default | Notes |
|--------|------|---------|-------|
| `id` | INTEGER | AUTOINCREMENT | Primary key |
| `title` | TEXT | `''` | Required |
| `content` | TEXT | `''` | Markdown body |
| `color` | TEXT | `'yellow'` | One of the supported note colors |
| `created_at` | TEXT | `datetime('now')` | UTC timestamp |
| `updated_at` | TEXT | `datetime('now')` | Updated on each edit |

### Table: `pomodoro_settings`

Singleton row (`id = 1`) for timer configuration.

| Column | Type | Default | Notes |
|--------|------|---------|-------|
| `id` | INTEGER | 1 | `CHECK (id = 1)` |
| `work_minutes` | INTEGER | `25` | Focus duration |
| `short_break_min` | INTEGER | `5` | Short break duration |
| `long_break_min` | INTEGER | `15` | Long break duration |
| `rounds_per_cycle` | INTEGER | `4` | Work rounds before long break |
| `milestones_json` | TEXT | `'[]'` | Serialized `PomodoroMilestone[]` |

### Table: `pomodoro_sessions`

| Column | Type | Default | Notes |
|--------|------|---------|-------|
| `id` | INTEGER | AUTOINCREMENT | Primary key |
| `completed_at` | TEXT | `datetime('now')` | Completion time |
| `duration_min` | INTEGER | — | Completed work-session length |

### Table: `app_settings`

Singleton row (`id = 1`) for UI preferences.

| Column | Type | Default | Notes |
|--------|------|---------|-------|
| `id` | INTEGER | 1 | `CHECK (id = 1)` |
| `page_size` | INTEGER | `50` | Shared list size preference |
| `todo_display` | TEXT | `'list'` | `list` or `grid` |
| `note_display` | TEXT | `'grid'` | `list` or `grid` |
| `note_template` | TEXT | `''` | Default Markdown template |
| `note_folder` | TEXT | `''` | Label/category hint for notes |

## Tauri Commands (IPC API)

All commands are called from the frontend through `invoke()` and return `Result<T, String>` on the Rust side.

### Todo Commands

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `list_todos` | — | `Todo[]` | Returns todos ordered by completion, priority, then deadline |
| `add_todo` | `CreateTodo` | `Todo` | Inserts a new todo |
| `toggle_todo` | `id` | `Todo` | Completes one-off todos or advances recurring todos to the next due occurrence |
| `update_todo` | `UpdateTodo` | `Todo` | Applies partial updates |
| `delete_todo` | `id` | `()` | Deletes the row |
| `list_due_todo_reminders` | — | `Todo[]` | Returns due or overdue reminders not yet delivered for the current occurrence |
| `mark_todo_reminded` | `id` | `Todo` | Records reminder delivery for the current todo occurrence |

### Note Commands

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `list_notes` | — | `StickyNote[]` | Returns notes ordered by `updated_at DESC` |
| `add_note` | `CreateNote` | `StickyNote` | Inserts a note |
| `update_note` | `UpdateNote` | `StickyNote` | Persists inline or window edits |
| `delete_note` | `id` | `()` | Deletes a note |

### Pomodoro Commands

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `get_pomodoro_settings` | — | `PomodoroSettings` | Loads timer settings and milestones |
| `save_pomodoro_settings` | `PomodoroSettings` | `()` | Upserts the singleton settings row |
| `record_pomodoro_session` | `duration_min` | `()` | Records a completed work interval |
| `get_today_pomodoro_count` | — | `i64` | Counts sessions completed today |
| `get_weekly_pomodoro_stats` | — | `DayStat[]` | Seven-day rolling stats |
| `update_tray_tooltip` | `text` | `()` | Updates the tray tooltip |

### App Commands

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `get_db_path` | — | `String` | Returns the active SQLite path |
| `get_app_settings` | — | `AppSettings` | Loads persisted UI preferences |
| `save_app_settings` | `AppSettings` | `()` | Persists preferences |
| `quit_app` | — | `()` | Exits the application |
| `open_note_window` | `note_id`, `title` | `()` | Creates or focuses a note-specific webview |

### Agent Built-In Tools

Agent built-in tools are exposed to the LLM as OpenAI-style function schemas and are executed through `src-tauri/src/commands/agents.rs`. Read tools complete immediately and are audited; write tools create pending actions that require user confirmation.

| Tool | Input | Output | Description |
|------|-------|--------|-------------|
| `web_fetch` | `url`, optional `max_chars` | Page metadata and extracted text | Fetches public HTTP/HTTPS text-like web pages for Agent translation, analysis, or summarization. Blocks credentials, localhost/private/link-local targets, non-text responses, oversized responses, and unsafe redirects. |

## Shared Types

### TypeScript: App Settings

```typescript
export type DisplayStyle = "list" | "grid";

export interface AppSettings {
  page_size: number;
  todo_display: DisplayStyle;
  note_display: DisplayStyle;
  note_template: string;
  note_folder: string;
}
```

### Rust: Pomodoro Milestone

```rust
pub struct PomodoroMilestone {
    pub name: String,
    pub deadline: String,
    pub status: String, // active | completed | cancelled
}
```

## Events

| Event | Direction | Payload | Purpose |
|-------|-----------|---------|---------|
| `tray-new-note` | Rust -> Frontend | `()` | Tells the main window to switch to the Notes tab and focus the editor |

---
<!-- PKB-metadata
last_updated: 2026-04-30
commit: local
updated_by: codex
-->
