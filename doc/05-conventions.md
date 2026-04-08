# Lazy Todo App — Conventions

<!-- maintained-by: human+ai -->

## Code Style

### Rust

- **Edition**: 2021
- **Formatting**: Standard `rustfmt` defaults
- **Naming**: `snake_case` for files, functions, variables; `PascalCase` for structs
- **Error handling**: All Tauri commands return `Result<T, String>`, mapping internal errors via `.map_err(|e| e.to_string())`
- **Database access**: All SQL goes through `db.rs` methods; command handlers never construct SQL directly
- **State sharing**: `Database` struct wraps `Mutex<Connection>` and is injected via `app.manage()`; accessed in commands via `State<'_, Database>`
- **Serde**: All models derive `Serialize`/`Deserialize` for JSON IPC; input structs use `Option<T>` for partial updates

### TypeScript / React

- **React version**: 18 with function components only (no class components)
- **State**: `useState` and `useEffect` hooks; no external state library
- **Hooks**: Custom hooks prefixed with `use` encapsulate Tauri `invoke()` calls and local state
- **Components**: One component per file, `PascalCase` filenames
- **Types**: Separate `types/*.ts` files for domain types; interfaces preferred over type aliases for objects
- **Imports**: Named imports from `@tauri-apps/api/core` (`invoke`) and `@tauri-apps/api/event` (`listen`)
- **Strict mode**: TypeScript strict mode enabled via `tsconfig.json`

### CSS

- **Single file**: All styles in `src/App.css`
- **Theme**: Dark theme using CSS custom properties (`--bg-primary`, `--text-main`, etc.)
- **Naming**: BEM-like flat classes (`.pomo-ring`, `.note-card`, `.tab-btn.active`)
- **Layout**: Flexbox and CSS Grid; no CSS framework
- **Animations**: CSS `@keyframes` for alerts and transitions

## File Organization Rules

1. **Backend models** in `src-tauri/src/models/` — one file per domain entity
2. **Backend commands** in `src-tauri/src/commands/` — one file per feature area
3. **Frontend types** in `src/types/` — mirror backend models
4. **Frontend hooks** in `src/hooks/` — one per feature or concern
5. **Frontend components** in `src/components/` — one component per file
6. **All DB queries** centralized in `db.rs` — command handlers are thin wrappers

## IPC Contract

- Frontend calls backend **only** via `invoke()` from `@tauri-apps/api/core`
- Backend sends events to frontend via `window.emit()` (Tauri `Emitter` trait)
- All command return types must be JSON-serializable (`Serialize` in Rust, matching TypeScript interfaces)
- Errors are returned as plain strings (no structured error types currently)

## Error Handling

| Layer | Pattern |
|-------|---------|
| Rust DB | `rusqlite::Result<T>` propagated to commands |
| Rust commands | `.map_err(\|e\| e.to_string())` converts to `Result<T, String>` |
| Frontend hooks | `.catch(console.error)` — errors logged, UI shows stale data |
| Frontend UI | Loading states shown while data is fetched |

## Git Conventions

- **Branch**: `master` is the main branch
- **Pre-commit**: `cargo clippy`, `cargo test`, `tsc --noEmit` (via `pre-commit` framework)
- **Commit style**: Conventional commits recommended (`feat:`, `fix:`, `docs:`, etc.)

## Architectural Constraints (from CLAUDE.md)

These rules are enforced for AI agents contributing to the project:

1. All Tauri commands defined in `src-tauri/src/commands/`
2. State management uses Tauri's managed state (`tauri::State`)
3. Frontend calls backend ONLY through `@tauri-apps/api/core.invoke()`
4. All data persistence goes through Rust/SQLite
5. All Tauri commands return `Result<T, String>`

---
<!-- PKB-metadata
last_updated: 2026-04-07
commit: 4c09050
updated_by: human+ai
-->
