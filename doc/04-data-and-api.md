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
| `created_at` | TEXT | `datetime('now')` | UTC timestamp |

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
| `toggle_todo` | `id` | `Todo` | Flips the completed flag |
| `update_todo` | `UpdateTodo` | `Todo` | Applies partial updates |
| `delete_todo` | `id` | `()` | Deletes the row |

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
last_updated: 2026-04-12
commit: f9ba186
updated_by: human+ai
-->
