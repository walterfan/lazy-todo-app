# Lazy Todo App — Architecture

<!-- maintained-by: human+ai -->

## C4 Model

### Level 1: System Context

Lazy Todo App is a local-first desktop productivity system. It does not depend on any hosted backend. Its only external integrations are operating-system capabilities such as windows, tray menus, notifications, and local file storage.

```mermaid
C4Context
    title System Context — Lazy Todo App

    Person(user, "User", "Desktop user managing tasks, notes, focus sessions, and preferences")
    System(app, "Lazy Todo App", "Cross-platform desktop productivity app")
    System_Ext(os, "Operating System", "Window manager, tray, notifications, local filesystem")

    Rel(user, app, "Uses", "Desktop UI")
    Rel(app, os, "Creates windows, stores SQLite DB, sends notifications")
```

### Level 2: Container

The app is packaged as a single desktop binary but runs with multiple cooperating runtime containers inside the Tauri shell.

```mermaid
C4Container
    title Container Diagram — Lazy Todo App

    Person(user, "User")

    Container_Boundary(tauri, "Tauri Application") {
        Container(main_ui, "Main Webview", "React 18 + TypeScript", "Tabs for todos, notes, pomodoro, settings")
        Container(note_ui, "Note Webview", "React 18 + TypeScript", "Focused single-note pop-out window")
        Container(rust, "Rust Backend", "Tauri + Rust", "IPC commands, tray control, window management, DB access")
        ContainerDb(sqlite, "SQLite Database", "rusqlite", "todos, sticky_notes, pomodoro_settings, pomodoro_sessions, app_settings")
    }

    System_Ext(os, "OS Services", "Tray, notifications, window manager, filesystem")

    Rel(user, main_ui, "Uses")
    Rel(user, note_ui, "Uses")
    Rel(main_ui, rust, "Calls", "Tauri invoke()")
    Rel(note_ui, rust, "Calls", "Tauri invoke()")
    Rel(rust, sqlite, "Reads/Writes")
    Rel(rust, os, "Uses")
```

**Boundary rule**: the frontend never talks directly to SQLite or the filesystem. Every state mutation crosses the `invoke()` boundary into Rust first.

### Level 3: Component

#### Rust Backend Components

```mermaid
graph TD
    LIB["lib.rs<br/>builder, tray, setup, command registration"]
    DB["db.rs<br/>schema creation and persistence"]

    subgraph Commands
        CMD_TODO["commands/todo.rs"]
        CMD_NOTE["commands/note.rs"]
        CMD_POMO["commands/pomodoro.rs"]
        CMD_APP["commands/app.rs"]
    end

    subgraph Models
        MOD_TODO["models/todo.rs"]
        MOD_NOTE["models/note.rs"]
        MOD_POMO["models/pomodoro.rs"]
        MOD_SETTINGS["models/settings.rs"]
    end

    LIB --> DB
    LIB --> CMD_TODO
    LIB --> CMD_NOTE
    LIB --> CMD_POMO
    LIB --> CMD_APP

    CMD_TODO --> DB
    CMD_NOTE --> DB
    CMD_POMO --> DB
    CMD_APP --> DB

    CMD_TODO --> MOD_TODO
    CMD_NOTE --> MOD_NOTE
    CMD_POMO --> MOD_POMO
    CMD_APP --> MOD_SETTINGS
```

#### React Frontend Components

```mermaid
graph TD
    MAIN["App.tsx<br/>main window shell"]
    NOTE_WINDOW["NoteWindow.tsx<br/>single-note window"]

    subgraph Hooks
        H_TODO["useTodos.ts"]
        H_NOTE["useNotes.ts"]
        H_POMO["usePomodoro.ts"]
        H_STATS["usePomodoroStats.ts"]
        H_SETTINGS["useSettings.ts"]
        H_COUNT["useCountdown.ts"]
    end

    subgraph Components
        TODO_LIST["TodoList + TodoItem"]
        NOTE_LIST["NoteList + NoteCard + NoteEditor"]
        POMO["PomodoroPanel + children"]
        SETTINGS["SettingsPanel"]
        MD["MarkdownPreview"]
    end

    MAIN --> H_TODO
    MAIN --> H_NOTE
    MAIN --> H_SETTINGS
    MAIN --> TODO_LIST
    MAIN --> NOTE_LIST
    MAIN --> POMO
    MAIN --> SETTINGS

    TODO_LIST --> H_COUNT
    NOTE_LIST --> MD
    NOTE_WINDOW --> MD
    NOTE_WINDOW --> H_NOTE
    POMO --> H_POMO
    POMO --> H_STATS
```

### Level 4: Code

#### Persistence Model

The SQLite schema currently uses five tables:

- `todos`
- `sticky_notes`
- `pomodoro_settings`
- `pomodoro_sessions`
- `app_settings`

Two singleton-style tables hold configuration:

- `pomodoro_settings` stores timer lengths plus `milestones_json`
- `app_settings` stores display preferences, note defaults, and storage-related UI hints

#### Representative Flow: Open a Sticky Note in Its Own Window

```mermaid
sequenceDiagram
    participant Card as NoteCard.tsx
    participant IPC as invoke("open_note_window")
    participant Cmd as commands/app.rs
    participant Win as WebviewWindowBuilder
    participant Bootstrap as src/main.tsx
    participant View as NoteWindow.tsx

    Card->>IPC: noteId, title
    IPC->>Cmd: open_note_window(note_id, title)
    Cmd->>Win: create or focus "note-{id}"
    Win-->>Bootstrap: load index.html?note={id}
    Bootstrap-->>View: render NoteWindow(noteId)
    View->>IPC: list_notes / update_note
```

## Key Architectural Decisions

| Decision | Rationale |
|----------|-----------|
| Keep all persistence in `db.rs` | Centralizes schema and SQL in one place for a small app |
| Persist app settings in SQLite | Settings survive restarts and reuse the same backend contract pattern as other features |
| Use a separate note window webview | Gives sticky notes a desktop-native feel without introducing a second frontend bundle |
| Hide the pomodoro tab instead of unmounting it | Prevents timer state from resetting when the user changes tabs |
| Store pomodoro milestones as JSON in `pomodoro_settings` | Keeps milestone data attached to the singleton timer config without another relational table |
| Resolve DB location from env, then config file, then app data | Supports both ad hoc overrides and persistent local configuration |
| Open external links with the shell plugin | Avoids in-webview navigation and keeps Markdown note links safe and predictable |

---
<!-- PKB-metadata
last_updated: 2026-04-12
commit: f9ba186
updated_by: human+ai
-->
