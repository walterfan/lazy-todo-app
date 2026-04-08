# Lazy Todo App — Architecture

<!-- maintained-by: human+ai -->

## C4 Model

### Level 1: System Context

The Lazy Todo App is a standalone desktop application. It has no external service dependencies — all data is stored locally. The only external interactions are OS-level: system tray, native notifications, and file system (SQLite).

```mermaid
C4Context
    title System Context — Lazy Todo App

    Person(user, "User", "Desktop user managing tasks, notes, and focus time")
    System(app, "Lazy Todo App", "Cross-platform desktop productivity app<br/>Tauri v2 + React + Rust + SQLite")
    System_Ext(os, "Operating System", "System tray, notifications, file system")

    Rel(user, app, "Uses", "GUI interaction")
    Rel(app, os, "System tray, notifications,<br/>SQLite file I/O")
```

### Level 2: Container

The application consists of two runtime containers packaged into a single binary:

```mermaid
C4Container
    title Container Diagram — Lazy Todo App

    Person(user, "User")

    Container_Boundary(tauri, "Tauri Application") {
        Container(webview, "React Frontend", "React 18 + TypeScript", "UI rendering, timer logic,<br/>user interactions")
        Container(rust, "Rust Backend", "Rust + Tauri v2", "IPC command handlers,<br/>state management, system tray")
        ContainerDb(sqlite, "SQLite Database", "rusqlite (bundled)", "todos, sticky_notes,<br/>pomodoro_settings,<br/>pomodoro_sessions")
    }

    System_Ext(os, "OS Services", "Tray, Notifications")

    Rel(user, webview, "Interacts via", "GUI")
    Rel(webview, rust, "Calls", "Tauri invoke() IPC")
    Rel(rust, sqlite, "Reads/Writes", "rusqlite")
    Rel(rust, os, "System tray,<br/>notifications", "Tauri plugins")
```

**Communication pattern**: The frontend NEVER accesses the file system or database directly. All data flows through Tauri's `invoke()` IPC mechanism, which serializes/deserializes JSON between the webview and Rust.

### Level 3: Component

#### Rust Backend Components

```mermaid
graph TB
    subgraph "src-tauri/src/"
        LIB["lib.rs<br/><i>App setup, tray, command registration</i>"]
        DB["db.rs<br/><i>Database struct, all SQL queries</i>"]

        subgraph "commands/"
            CMD_TODO["todo.rs<br/><i>list, add, toggle, update, delete</i>"]
            CMD_NOTE["note.rs<br/><i>list, add, update, delete</i>"]
            CMD_POMO["pomodoro.rs<br/><i>settings, sessions, stats, tooltip</i>"]
            CMD_APP["app.rs<br/><i>get_db_path</i>"]
        end

        subgraph "models/"
            MOD_TODO["todo.rs<br/><i>Todo, CreateTodo, UpdateTodo</i>"]
            MOD_NOTE["note.rs<br/><i>StickyNote, CreateNote, UpdateNote</i>"]
            MOD_POMO["pomodoro.rs<br/><i>PomodoroSettings, DayStat</i>"]
        end
    end

    LIB -->|registers| CMD_TODO
    LIB -->|registers| CMD_NOTE
    LIB -->|registers| CMD_POMO
    LIB -->|registers| CMD_APP
    LIB -->|initializes| DB

    CMD_TODO -->|uses| DB
    CMD_NOTE -->|uses| DB
    CMD_POMO -->|uses| DB
    CMD_APP -->|uses| DB

    CMD_TODO -->|types| MOD_TODO
    CMD_NOTE -->|types| MOD_NOTE
    CMD_POMO -->|types| MOD_POMO
```

#### React Frontend Components

```mermaid
graph TB
    subgraph "src/"
        APP["App.tsx<br/><i>Tab navigation, tray events</i>"]

        subgraph "hooks/"
            H_TODO["useTodos.ts"]
            H_COUNT["useCountdown.ts"]
            H_NOTE["useNotes.ts"]
            H_POMO["usePomodoro.ts"]
            H_STAT["usePomodoroStats.ts"]
        end

        subgraph "components/ — Todos"
            C_ADD["AddTodo.tsx"]
            C_ITEM["TodoItem.tsx"]
            C_LIST["TodoList.tsx"]
        end

        subgraph "components/ — Notes"
            C_EDIT["NoteEditor.tsx"]
            C_CARD["NoteCard.tsx"]
            C_NLIST["NoteList.tsx"]
            C_MD["MarkdownPreview.tsx"]
        end

        subgraph "components/ — Pomodoro"
            C_PANEL["PomodoroPanel.tsx"]
            C_RING["PomodoroRing.tsx"]
            C_CTRL["PomodoroControls.tsx"]
            C_SET["PomodoroSettings.tsx"]
            C_STATS["PomodoroStats.tsx"]
            C_ALERT["PomodoroAlert.tsx"]
        end
    end

    APP --> C_ADD
    APP --> C_LIST
    APP --> C_EDIT
    APP --> C_NLIST
    APP --> C_PANEL

    C_LIST --> C_ITEM
    C_ITEM --> H_COUNT
    C_NLIST --> C_CARD
    C_CARD --> C_MD

    C_PANEL --> C_RING
    C_PANEL --> C_CTRL
    C_PANEL --> C_SET
    C_PANEL --> C_STATS
    C_PANEL --> C_ALERT

    APP --> H_TODO
    APP --> H_NOTE
    C_PANEL --> H_POMO
    C_PANEL --> H_STAT
```

### Level 4: Code

#### Database Schema (SQLite)

```sql
-- Table: todos
CREATE TABLE IF NOT EXISTS todos (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    title       TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    priority    INTEGER NOT NULL DEFAULT 2,  -- 1=High, 2=Medium, 3=Low
    completed   INTEGER NOT NULL DEFAULT 0,
    deadline    TEXT,                         -- ISO 8601
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Table: sticky_notes
CREATE TABLE IF NOT EXISTS sticky_notes (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    title       TEXT NOT NULL DEFAULT '',
    content     TEXT NOT NULL DEFAULT '',
    color       TEXT NOT NULL DEFAULT 'yellow',
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Table: pomodoro_settings (singleton row, id=1)
CREATE TABLE IF NOT EXISTS pomodoro_settings (
    id               INTEGER PRIMARY KEY CHECK (id = 1),
    work_minutes     INTEGER NOT NULL DEFAULT 25,
    short_break_min  INTEGER NOT NULL DEFAULT 5,
    long_break_min   INTEGER NOT NULL DEFAULT 15,
    rounds_per_cycle INTEGER NOT NULL DEFAULT 4
);

-- Table: pomodoro_sessions
CREATE TABLE IF NOT EXISTS pomodoro_sessions (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    completed_at TEXT NOT NULL DEFAULT (datetime('now')),
    duration_min INTEGER NOT NULL
);
```

#### IPC Data Flow (Sequence)

```mermaid
sequenceDiagram
    participant UI as React Component
    participant Hook as React Hook
    participant IPC as Tauri invoke()
    participant Cmd as Rust Command
    participant DB as SQLite (db.rs)

    UI->>Hook: User action (e.g. add todo)
    Hook->>IPC: invoke("add_todo", { title, priority, ... })
    IPC->>Cmd: Deserialize JSON → CreateTodo struct
    Cmd->>DB: db.add_todo(title, desc, priority, deadline)
    DB-->>Cmd: Result<Todo>
    Cmd-->>IPC: Serialize Todo → JSON
    IPC-->>Hook: Promise<Todo> resolves
    Hook-->>UI: State update → re-render
```

## Key Architectural Decisions

| Decision | Rationale |
|----------|-----------|
| Frontend timer via `setInterval` | Avoids Rust async complexity; Web Audio API available for alerts |
| CSS visibility toggle (not conditional render) for Pomodoro tab | Prevents timer reset when switching tabs |
| Single `db.rs` file for all queries | Project is small enough; avoids over-abstraction |
| `rusqlite` with `bundled` feature | No external SQLite dependency needed at runtime |
| System tray via Tauri API | Native integration; close-to-tray pattern |
| Web Audio API for alert chime | No external audio files needed; works cross-platform |
| `LAZY_TODO_DB_DIR` env var for DB path | Simple override without config file complexity |

---
<!-- PKB-metadata
last_updated: 2026-04-07
commit: 4c09050
updated_by: human+ai
-->
