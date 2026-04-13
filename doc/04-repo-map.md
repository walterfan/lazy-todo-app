# Lazy Todo App — Repository Map

<!-- maintained-by: human+ai -->

## Directory Structure

```text
lazy-todo-app/
├── .github/
│   └── workflows/
│       └── release.yml              # GitHub Actions release pipeline for Tauri bundles
├── CLAUDE.md                        # AI agent architectural rules
├── README.md                        # End-user and project overview
├── package.json                     # Node.js dependencies and frontend scripts
├── tsconfig.json                    # TypeScript compiler configuration
├── vite.config.ts                   # Vite bundler configuration
├── index.html                       # Vite entry HTML
│
├── src/                             # React frontend
│   ├── main.tsx                     # Chooses App or NoteWindow based on URL query
│   ├── App.tsx                      # Main shell: navigation, search, tabs, settings
│   ├── App.css                      # Shared dark theme styles
│   ├── types/
│   │   ├── todo.ts
│   │   ├── note.ts
│   │   ├── pomodoro.ts
│   │   └── settings.ts             # AppSettings and display-mode types
│   ├── hooks/
│   │   ├── useTodos.ts
│   │   ├── useCountdown.ts
│   │   ├── useNotes.ts
│   │   ├── usePomodoro.ts
│   │   ├── usePomodoroStats.ts
│   │   └── useSettings.ts          # Loads and persists app settings
│   └── components/
│       ├── AddTodo.tsx
│       ├── TodoItem.tsx
│       ├── TodoList.tsx
│       ├── NoteEditor.tsx
│       ├── NoteCard.tsx
│       ├── NoteList.tsx
│       ├── NoteWindow.tsx          # Dedicated window for a single sticky note
│       ├── MarkdownPreview.tsx
│       ├── PomodoroPanel.tsx
│       ├── PomodoroRing.tsx
│       ├── PomodoroControls.tsx
│       ├── PomodoroSettings.tsx
│       ├── PomodoroMilestones.tsx  # Milestone cards and status actions
│       ├── PomodoroStats.tsx
│       ├── PomodoroAlert.tsx
│       └── SettingsPanel.tsx       # Display, notes, and storage preferences
│
├── src-tauri/                       # Rust backend
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json
│   ├── icons/
│   ├── gen/
│   │   └── schemas/                # Generated Tauri capability schemas
│   └── src/
│       ├── main.rs                 # Rust binary entry
│       ├── lib.rs                  # App builder, tray wiring, command registration
│       ├── db.rs                   # SQLite schema creation and all persistence queries
│       ├── models/
│       │   ├── mod.rs
│       │   ├── todo.rs
│       │   ├── note.rs
│       │   ├── pomodoro.rs         # Pomodoro settings, milestones, stats
│       │   └── settings.rs         # AppSettings persistence model
│       └── commands/
│           ├── mod.rs
│           ├── todo.rs
│           ├── note.rs
│           ├── pomodoro.rs
│           └── app.rs              # DB path, settings, quit, note window commands
│
├── doc/                             # Project Knowledge Base and Sphinx source
│   ├── index.md
│   ├── 00-overview.md ... 12-document.md
│   ├── ai-guide.md
│   ├── conf.py
│   ├── Makefile
│   ├── pyproject.toml
│   ├── requirements.txt
│   ├── _static/
│   ├── _templates/
│   ├── locale/
│   ├── adr/
│   └── changes/
│
├── openspec/                        # OpenSpec proposals and design artifacts
└── dist/                            # Frontend build output
```

## Entry Points

| Context | File | Purpose |
|---------|------|---------|
| Rust binary | `src-tauri/src/main.rs` | Calls `lazy_todo_app_lib::run()` |
| Tauri setup | `src-tauri/src/lib.rs` | Resolves DB directory, initializes tray, registers commands |
| React bootstrap | `src/main.tsx` | Routes between the main app and pop-out note windows |
| Main shell | `src/App.tsx` | Tabs, search, settings panel, tray event handling |
| Standalone note view | `src/components/NoteWindow.tsx` | Focused single-note window UI |
| Vite entry | `index.html` | Loads `/src/main.tsx` |

## Key Configuration Files

| File | Purpose |
|------|---------|
| `CLAUDE.md` | AI agent contribution constraints |
| `package.json` | npm scripts and frontend packages |
| `src-tauri/Cargo.toml` | Rust crate dependencies |
| `src-tauri/tauri.conf.json` | App identifier, bundle targets, build commands |
| `src-tauri/capabilities/default.json` | Tauri permissions |
| `.github/workflows/release.yml` | GitHub Release build automation |
| `doc/conf.py` | Sphinx, MyST, and i18n configuration |
| `doc/Makefile` | gettext and bilingual HTML build targets |

## Naming Conventions

- **Rust files**: `snake_case.rs`
- **React components**: `PascalCase.tsx`
- **React hooks**: `useCamelCase.ts`
- **Type files**: domain-oriented `.ts` files under `src/types/`
- **Documentation**: numbered PKB files under `doc/` with metadata footers

---
<!-- PKB-metadata
last_updated: 2026-04-12
commit: f9ba186
updated_by: human+ai
-->
