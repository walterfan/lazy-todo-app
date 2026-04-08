# Lazy Todo App — Data & API Reference

<!-- maintained-by: human+ai -->

## Database

- **Engine**: SQLite (bundled via `rusqlite` with `bundled` feature)
- **File**: `todos.db` in the app data directory (or `LAZY_TODO_DB_DIR`)
- **Schema creation**: `db.rs::Database::new()` runs `CREATE TABLE IF NOT EXISTS` for all 4 tables on startup

### Table: `todos`

| Column | Type | Default | Notes |
|--------|------|---------|-------|
| `id` | INTEGER | AUTOINCREMENT | Primary key |
| `title` | TEXT | (required) | NOT NULL |
| `description` | TEXT | `''` | NOT NULL |
| `priority` | INTEGER | `2` | 1=High, 2=Medium, 3=Low |
| `completed` | INTEGER | `0` | Boolean (0/1) |
| `deadline` | TEXT | NULL | ISO 8601 datetime string |
| `created_at` | TEXT | `datetime('now')` | UTC timestamp |

**Query order**: `completed ASC, priority ASC, deadline ASC NULLS LAST`

### Table: `sticky_notes`

| Column | Type | Default | Notes |
|--------|------|---------|-------|
| `id` | INTEGER | AUTOINCREMENT | Primary key |
| `title` | TEXT | `''` | NOT NULL |
| `content` | TEXT | `''` | Markdown content, NOT NULL |
| `color` | TEXT | `'yellow'` | One of: yellow, green, blue, pink, purple, orange |
| `created_at` | TEXT | `datetime('now')` | UTC timestamp |
| `updated_at` | TEXT | `datetime('now')` | Updated on every edit |

**Query order**: `updated_at DESC`

### Table: `pomodoro_settings`

Singleton table — always exactly one row with `id = 1`.

| Column | Type | Default | Notes |
|--------|------|---------|-------|
| `id` | INTEGER | 1 | `CHECK (id = 1)` |
| `work_minutes` | INTEGER | `25` | Work phase duration |
| `short_break_min` | INTEGER | `5` | Short break duration |
| `long_break_min` | INTEGER | `15` | Long break duration |
| `rounds_per_cycle` | INTEGER | `4` | Work rounds before long break |

**Upsert strategy**: `INSERT ... ON CONFLICT(id) DO UPDATE SET ...`

### Table: `pomodoro_sessions`

| Column | Type | Default | Notes |
|--------|------|---------|-------|
| `id` | INTEGER | AUTOINCREMENT | Primary key |
| `completed_at` | TEXT | `datetime('now')` | When the session finished |
| `duration_min` | INTEGER | (required) | Duration of the work session |

**Stats queries**: Filter by `date(completed_at)` for daily/weekly aggregation.

## Tauri Commands (IPC API)

All commands are invoked from the frontend via `invoke("command_name", args)` and return `Result<T, String>`.

### Todo Commands (`commands/todo.rs`)

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `list_todos` | — | `Vec<Todo>` | All todos, sorted by priority and deadline |
| `add_todo` | `CreateTodo { title, description?, priority?, deadline? }` | `Todo` | Insert new todo, returns the created row |
| `toggle_todo` | `id: i64` | `Todo` | Flip `completed` flag, returns updated row |
| `update_todo` | `UpdateTodo { id, title?, description?, priority?, deadline? }` | `Todo` | Partial update, returns updated row |
| `delete_todo` | `id: i64` | `()` | Remove todo by ID |

### Note Commands (`commands/note.rs`)

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `list_notes` | — | `Vec<StickyNote>` | All notes, most recently updated first |
| `add_note` | `CreateNote { title?, content?, color? }` | `StickyNote` | Insert new note (defaults: empty title/content, yellow) |
| `update_note` | `UpdateNote { id, title?, content?, color? }` | `StickyNote` | Partial update, also bumps `updated_at` |
| `delete_note` | `id: i64` | `()` | Remove note by ID |

### Pomodoro Commands (`commands/pomodoro.rs`)

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `get_pomodoro_settings` | — | `PomodoroSettings` | Get current settings (auto-creates row if missing) |
| `save_pomodoro_settings` | `PomodoroSettings` | `()` | Upsert settings |
| `record_pomodoro_session` | `duration_min: i64` | `()` | Log a completed work session |
| `get_today_pomodoro_count` | — | `i64` | Count sessions where `date(completed_at) = date('now')` |
| `get_weekly_pomodoro_stats` | — | `Vec<DayStat>` | Last 7 days: `[{ date, count }]` |
| `update_tray_tooltip` | `text: String` | `()` | Set system tray hover text |

### App Commands (`commands/app.rs`)

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `get_db_path` | — | `String` | Full filesystem path to the SQLite database file |

## TypeScript Types

### Todo (`src/types/todo.ts`)

```typescript
interface Todo {
  id: number;
  title: string;
  description: string;
  priority: number;      // 1=High, 2=Medium, 3=Low
  completed: boolean;
  deadline: string | null; // ISO 8601
  created_at: string;
}

interface CreateTodo {
  title: string;
  description?: string;
  priority?: number;
  deadline?: string;
}

interface UpdateTodo {
  id: number;
  title?: string;
  description?: string;
  priority?: number;
  deadline?: string;
}
```

### Note (`src/types/note.ts`)

```typescript
type NoteColor = 'yellow' | 'green' | 'blue' | 'pink' | 'purple' | 'orange';

interface StickyNote {
  id: number;
  title: string;
  content: string;
  color: NoteColor;
  created_at: string;
  updated_at: string;
}
```

### Pomodoro (`src/types/pomodoro.ts`)

```typescript
interface PomodoroSettings {
  work_minutes: number;
  short_break_min: number;
  long_break_min: number;
  rounds_per_cycle: number;
}

interface DayStat { date: string; count: number; }
type TimerPhase = "work" | "short_break" | "long_break";

interface TimerState {
  phase: TimerPhase;
  remainingMs: number;
  totalMs: number;
  running: boolean;
  currentRound: number;
}
```

## Events

| Event | Direction | Payload | Purpose |
|-------|-----------|---------|---------|
| `tray-new-note` | Rust → Frontend | `()` | System tray "New Note" menu clicked |

---
<!-- PKB-metadata
last_updated: 2026-04-07
commit: 4c09050
updated_by: human+ai
-->
