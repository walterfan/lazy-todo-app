# Lazy Todo App — Repository Map

<!-- maintained-by: human+ai -->

## Directory Structure

```
lazy-todo-app/
├── CLAUDE.md                        # AI Agent architectural rules (Harness Engineering)
├── README.md                        # Project documentation
├── package.json                     # Node.js dependencies & scripts
├── tsconfig.json                    # TypeScript configuration
├── vite.config.ts                   # Vite bundler configuration
├── index.html                       # Vite entry HTML
│
├── src/                             # ── React Frontend ──
│   ├── main.tsx                     # React DOM entry point
│   ├── App.tsx                      # Root component — tab navigation, tray events
│   ├── App.css                      # Global styles (dark theme, all features)
│   ├── types/                       # TypeScript type definitions
│   │   ├── todo.ts                  # Todo, CreateTodo, UpdateTodo
│   │   ├── note.ts                  # StickyNote, CreateNote, UpdateNote, NoteColor
│   │   └── pomodoro.ts              # PomodoroSettings, DayStat, TimerPhase, TimerState
│   ├── hooks/                       # React hooks (Tauri IPC + logic)
│   │   ├── useTodos.ts              # CRUD operations for todos
│   │   ├── useCountdown.ts          # Real-time deadline countdown
│   │   ├── useNotes.ts              # CRUD operations for sticky notes
│   │   ├── usePomodoro.ts           # Timer logic, phase transitions, alerts
│   │   └── usePomodoroStats.ts      # Daily/weekly Pomodoro statistics
│   └── components/                  # React UI components
│       ├── AddTodo.tsx              # New todo form (priority + deadline picker)
│       ├── TodoItem.tsx             # Single todo row (countdown, inline edit)
│       ├── TodoList.tsx             # Todo list with pending/completed grouping
│       ├── NoteEditor.tsx           # New note form (title, content, color)
│       ├── NoteCard.tsx             # Individual sticky note (expand, edit, delete)
│       ├── NoteList.tsx             # Grid layout of NoteCards
│       ├── MarkdownPreview.tsx      # Markdown renderer (react-markdown)
│       ├── PomodoroPanel.tsx        # Pomodoro tab layout container
│       ├── PomodoroRing.tsx         # SVG circular progress indicator
│       ├── PomodoroControls.tsx     # Start / Pause / Reset buttons
│       ├── PomodoroSettings.tsx     # Duration & cycle configuration form
│       ├── PomodoroStats.tsx        # Today count + 7-day bar chart (SVG)
│       └── PomodoroAlert.tsx        # Modal dialog with Web Audio chime
│
├── src-tauri/                       # ── Rust Backend ──
│   ├── Cargo.toml                   # Rust dependencies
│   ├── tauri.conf.json              # Tauri app config (window, bundle, build)
│   ├── capabilities/
│   │   └── default.json             # Tauri permissions (tray, menu, event, notification)
│   ├── icons/                       # App icons (32, 128, 256, 512, ICO)
│   └── src/
│       ├── main.rs                  # Rust binary entry (calls lib::run)
│       ├── lib.rs                   # Tauri builder setup, tray, command registration
│       ├── db.rs                    # SQLite database (schema creation + all queries)
│       ├── models/
│       │   ├── mod.rs               # Module declarations
│       │   ├── todo.rs              # Todo, CreateTodo, UpdateTodo structs
│       │   ├── note.rs              # StickyNote, CreateNote, UpdateNote structs
│       │   └── pomodoro.rs          # PomodoroSettings, DayStat structs
│       └── commands/
│           ├── mod.rs               # Module declarations
│           ├── todo.rs              # 5 Tauri commands for todo CRUD
│           ├── note.rs              # 4 Tauri commands for note CRUD
│           ├── pomodoro.rs          # 5 Tauri commands for pomodoro
│           └── app.rs               # 1 Tauri command (get_db_path)
│
├── openspec/                        # OpenSpec change proposals & specs
│   ├── changes/
│   │   ├── desktop-sticky-notes/    # Sticky notes feature proposal + specs
│   │   └── pomodoro-timer/          # Pomodoro feature proposal + specs
│   └── specs/
│
├── man/                             # Project Knowledge Base (this directory)
│
└── dist/                            # Vite build output (gitignored)
```

## Entry Points

| Context | File | Purpose |
|---------|------|---------|
| Rust binary | `src-tauri/src/main.rs` | Calls `lazy_todo_app_lib::run()` |
| Tauri setup | `src-tauri/src/lib.rs` | Builder config, tray, DB init, command registration |
| React app | `src/main.tsx` | `ReactDOM.createRoot` renders `<App />` |
| Root component | `src/App.tsx` | Tab navigation, event listeners |
| Vite entry | `index.html` | Loads `/src/main.tsx` |

## Key Configuration Files

| File | Purpose |
|------|---------|
| `CLAUDE.md` | AI agent architectural constraints |
| `package.json` | npm scripts: `dev`, `build`, `preview`, `tauri` |
| `src-tauri/Cargo.toml` | Rust crate definition + dependencies |
| `src-tauri/tauri.conf.json` | Window dimensions, bundle settings, build commands |
| `src-tauri/capabilities/default.json` | Tauri capability permissions |
| `tsconfig.json` | TypeScript compiler options |
| `vite.config.ts` | Vite plugins and dev server config |

## Naming Conventions

- **Rust files**: `snake_case.rs` (modules, models, commands)
- **React components**: `PascalCase.tsx` (one component per file)
- **React hooks**: `useCamelCase.ts` (prefixed with `use`)
- **Type files**: `camelCase.ts` (match domain entity names)
- **CSS**: single `App.css` file with BEM-like class naming (`.pomo-ring`, `.note-card`, `.tab-btn`)

---
<!-- PKB-metadata
last_updated: 2026-04-07
commit: 4c09050
updated_by: human+ai
-->
