# Lazy Todo App — Workflows

<!-- maintained-by: human+ai -->

## Workflow 1: Todo Lifecycle

A todo item flows through creation, optional editing, completion, and deletion.

```mermaid
stateDiagram-v2
    [*] --> Created: User fills AddTodo form
    Created --> Active: Saved to SQLite
    Active --> Editing: User clicks edit
    Editing --> Active: Save changes
    Active --> Completed: User toggles checkbox
    Completed --> Active: User un-toggles
    Active --> Deleted: User clicks delete
    Completed --> Deleted: User clicks delete
    Deleted --> [*]
```

### Code Path

1. **Create**: `AddTodo.tsx` → `useTodos.addTodo()` → `invoke("add_todo", CreateTodo)` → `commands/todo.rs::add_todo` → `db.rs::add_todo`
2. **List**: On mount, `useTodos` calls `invoke("list_todos")` → `db.rs::list_todos` (ordered by `completed ASC, priority ASC, deadline ASC NULLS LAST`)
3. **Toggle**: `TodoItem.tsx` checkbox → `useTodos.toggleTodo(id)` → `invoke("toggle_todo")` → `db.rs::toggle_todo` (flips `completed` column)
4. **Edit**: `TodoItem.tsx` inline fields → `useTodos.updateTodo(UpdateTodo)` → `invoke("update_todo")` → `db.rs::update_todo` (partial update)
5. **Delete**: `TodoItem.tsx` delete button → `useTodos.deleteTodo(id)` → `invoke("delete_todo")` → `db.rs::delete_todo`

### Countdown Timer

`useCountdown.ts` runs a `setInterval(1000ms)` that recalculates remaining time for each todo with a `deadline`. Visual states:
- **Normal**: white text
- **< 1 hour**: orange text
- **Overdue**: red text

## Workflow 2: Sticky Note Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Draft: User opens NoteEditor
    Draft --> Saved: Submit (title + content + color)
    Saved --> Viewing: Displayed in NoteList grid
    Viewing --> Expanded: User clicks card
    Expanded --> Editing: User clicks edit in expanded view
    Editing --> Saved: Save changes
    Viewing --> Deleted: User clicks delete
    Expanded --> Deleted: User clicks delete
    Deleted --> [*]
```

### Code Path

1. **Create**: `NoteEditor.tsx` → `useNotes.addNote(CreateNote)` → `invoke("add_note")` → `db.rs::insert_note`
2. **List**: `useNotes` calls `invoke("list_notes")` → `db.rs::list_notes` (ordered by `updated_at DESC`)
3. **Edit**: `NoteCard.tsx` inline editing → `useNotes.updateNote(UpdateNote)` → `invoke("update_note")` → `db.rs::update_note` (also updates `updated_at`)
4. **Delete**: `NoteCard.tsx` → `useNotes.deleteNote(id)` → `invoke("delete_note")` → `db.rs::delete_note`
5. **Tray shortcut**: System tray "New Note" menu → `lib.rs` emits `tray-new-note` event → `App.tsx` switches to Notes tab and focuses editor

### Markdown Rendering

`NoteCard.tsx` renders note content via `MarkdownPreview.tsx`, which uses `react-markdown` with `remark-gfm` for GitHub Flavored Markdown (tables, strikethrough, task lists, etc.).

## Workflow 3: Pomodoro Cycle

A Pomodoro session cycles through work and break phases automatically.

```mermaid
stateDiagram-v2
    [*] --> Idle: App loaded
    Idle --> Work: User clicks Start
    Work --> WorkPaused: User clicks Pause
    WorkPaused --> Work: User clicks Resume
    Work --> ShortBreak: Timer reaches 0 (rounds < max)
    Work --> LongBreak: Timer reaches 0 (rounds = max)
    ShortBreak --> Work: Break timer reaches 0
    LongBreak --> Idle: Long break timer reaches 0 (cycle reset)
    Work --> Idle: User clicks Reset
    ShortBreak --> Idle: User clicks Reset
    LongBreak --> Idle: User clicks Reset
```

### Phase Transition Detail

Managed by `usePomodoro.ts`:

1. Timer runs via `setInterval(100ms)` for smooth countdown
2. When `remainingMs <= 0`:
   - Record the completed work session: `invoke("record_pomodoro_session")`
   - Send system notification: `sendNotification()` via `@tauri-apps/plugin-notification`
   - Set `alertPhase` state → triggers `PomodoroAlert.tsx` modal
   - Call `getCurrentWindow().show()` + `setFocus()` to bring window to foreground
   - Play Web Audio chime (ascending C-major arpeggio: C5→E5→G5→C6, twice)
   - Auto-advance to next phase (work → short break → work → ... → long break → reset)
3. Tray tooltip updated with current timer status: `invoke("update_tray_tooltip")`

### Alert Dismissal

User clicks "OK" on the `PomodoroAlert` overlay or clicks the overlay backdrop → `dismissAlert()` clears `alertPhase` → modal closes → next phase timer starts automatically.

## Workflow 4: System Tray Interaction

```mermaid
flowchart TD
    A[User closes window] -->|CloseRequested event| B[Window hides instead of closing]
    B --> C[App lives in system tray]
    C -->|Left-click tray icon| D{Window visible?}
    D -->|Yes| E[Hide window]
    D -->|No| F[Show + focus window]
    C -->|Right-click tray icon| G[Context menu]
    G --> H[Show/Hide]
    G --> I[New Note]
    G --> J[Quit]
    I --> K[Show window + switch to Notes tab + emit tray-new-note]
    J --> L[app.exit 0]
```

### Code Path

- Tray setup: `lib.rs::setup_tray()` builds `TrayIconBuilder` with menu items
- Close intercept: `lib.rs` `.on_window_event` catches `CloseRequested`, calls `api.prevent_close()` + `window.hide()`
- Menu handler: `lib.rs` `.on_menu_event` matches menu item IDs (`show_hide`, `new_note`, `quit`)
- Left-click toggle: `lib.rs` `.on_tray_icon_event` checks `TrayIconEvent::Click`

## Workflow 5: App Startup

```mermaid
flowchart TD
    A[main.rs] --> B[lib::run]
    B --> C[Tauri Builder]
    C --> D[Init plugins: shell, notification]
    C --> E[setup callback]
    E --> F{LAZY_TODO_DB_DIR env var set?}
    F -->|Yes| G[Use custom path]
    F -->|No| H[Use app_data_dir]
    G --> I[Database::new — create dir + open SQLite + create tables]
    H --> I
    I --> J[app.manage database — inject into Tauri state]
    J --> K[setup_tray — icon, menu, event handlers]
    K --> L[Register 16 invoke commands]
    L --> M[Frontend loads in webview]
    M --> N[App.tsx mounts — fetches db_path, listens for tray events]
    N --> O[Default tab: Todos — useTodos fetches list]
```

---
<!-- PKB-metadata
last_updated: 2026-04-07
commit: 4c09050
updated_by: human+ai
-->
