# Lazy Todo App — Workflows

<!-- maintained-by: human+ai -->

## Workflow 1: Todo Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Draft
    Draft --> Active: add_todo
    Active --> Editing: inline edit
    Editing --> Active: update_todo
    Active --> Completed: toggle_todo
    Active --> Active: toggle_todo recurring occurrence
    Completed --> Active: toggle_todo
    Active --> Deleted: delete_todo
    Completed --> Deleted: delete_todo
    Deleted --> [*]
```

### Code Path

1. `AddTodo.tsx` submits to `useTodos.addTodo()`.
2. `useTodos` calls `invoke("add_todo", { input })`.
3. `commands/todo.rs` validates and forwards to `db.rs`.
4. `TodoList.tsx` renders active tasks in list or grid mode based on `settings.todo_display`.
5. `TodoItem.tsx` uses `useCountdown.ts` to show deadline status in real time.
6. Recurring todos keep `deadline` as the next visible due occurrence; completing one records `todo_occurrences` history and advances the deadline.
7. `useTodos.ts` checks due reminders while the app is running, marks delivered occurrences, and falls back to in-app reminder state if desktop notifications are unavailable.

## Workflow 2: Sticky Note Creation and Pop-Out

```mermaid
flowchart TD
    A["NoteEditor.tsx"] -->|add_note| B["SQLite sticky_notes row"]
    B --> C["NoteList.tsx renders card"]
    C -->|Open in window| D["invoke open_note_window"]
    D --> E["commands/app.rs"]
    E --> F["Create or focus note window"]
    F --> G["Load note window route"]
    G --> H["NoteWindow.tsx"]
```

### Notes

- `NoteCard.tsx` can still edit inline in the main window.
- `NoteWindow.tsx` loads the note by ID using `list_notes` and persists edits through `update_note`.
- Markdown is rendered by `MarkdownPreview.tsx`.
- External links are intercepted in `src/main.tsx` and opened through `@tauri-apps/plugin-shell`.

## Workflow 3: Pomodoro Session and Milestones

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Work: Start
    Work --> Paused: Pause
    Paused --> Work: Resume
    Work --> ShortBreak: session complete
    Work --> LongBreak: last round complete
    ShortBreak --> Work: break complete
    LongBreak --> Idle: cycle complete
    Work --> Idle: Reset or Skip
    ShortBreak --> Idle: Reset
    LongBreak --> Idle: Reset
```

### Phase Handling

- `usePomodoro.ts` keeps the timer alive even when the Pomodoro tab is hidden.
- Finishing a work phase records a session via `record_pomodoro_session`.
- `PomodoroMilestones.tsx` surfaces up to three active milestones and lets the user mark them as `completed`, `cancelled`, or restored to `active`.
- The backend stores milestone state inside `pomodoro_settings.milestones_json`.
- `update_tray_tooltip` keeps the tray hover text synchronized with the timer.

## Workflow 4: Settings Persistence

```mermaid
sequenceDiagram
    participant UI as "SettingsPanel.tsx"
    participant Hook as "useSettings.ts"
    participant IPC as "invoke()"
    participant Cmd as "commands/app.rs"
    participant DB as "db.rs"

    UI->>Hook: onUpdate(partial settings)
    Hook->>Hook: merge with current state
    Hook->>IPC: save_app_settings
    IPC->>Cmd: save_app_settings(settings)
    Cmd->>DB: upsert app_settings row
    DB-->>Cmd: OK
    Cmd-->>Hook: OK
    Hook-->>UI: optimistic state already visible
```

### Stored Preferences

- `page_size`
- `todo_display`
- `note_display`
- `note_template`
- `note_folder`

These settings are loaded on app startup via `get_app_settings`.

## Workflow 5: System Tray and Window Lifecycle

```mermaid
flowchart TD
    A["User closes main window"] --> B["CloseRequested intercepted"]
    B --> C["Hide main window"]
    C --> D["Tray remains active"]
    D -->|Left click| E["Toggle main window visibility"]
    D -->|Menu: New Note| F["Show main window"]
    F --> G["Emit tray-new-note"]
    G --> H["App.tsx switches to Notes tab and autofocuses editor"]
    D -->|Menu: Quit| I["Exit application"]
```

## Workflow 6: App Startup

```mermaid
flowchart TD
    A["main.rs"] --> B["Run application"]
    B --> C["Resolve DB directory"]
    C --> D{"Env var set?"}
    D -->|Yes| E["Use LAZY_TODO_DB_DIR"]
    D -->|No| F{"Config file exists?"}
    F -->|Yes| G["Use config.json override"]
    F -->|No| H["Use app_data_dir"]
    E --> I["Initialize database"]
    G --> I
    H --> I
    I --> J["Setup tray"]
    J --> K["register invoke commands"]
    K --> L["Frontend boot"]
    L --> M["Load startup settings"]
```

---
<!-- PKB-metadata
last_updated: 2026-04-12
commit: f9ba186
updated_by: human+ai
-->
