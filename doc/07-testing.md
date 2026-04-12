# Lazy Todo App — Testing Strategy

<!-- maintained-by: human+ai -->

## Test Layers

| Layer | Primary Checks | Notes |
|-------|----------------|------|
| TypeScript frontend | `npm run build`, `npx tsc --noEmit` | Validates React components, hooks, and shared types |
| Rust backend | `cargo check`, `cargo clippy`, `cargo test` | Validates Tauri commands, SQLite access, and model contracts |
| Desktop behavior | Manual smoke test | Tray behavior, notifications, note windows, and focus handling are OS-facing |
| Documentation | `poetry run make html` | Ensures Sphinx/MyST/i18n docs remain buildable |

## Core Regression Areas

### Todos

- Add, edit, toggle, and delete a todo.
- Verify search filters active tasks by title and description.
- Verify display mode changes between list and grid without losing data.

### Sticky Notes

- Create a note from the main window and from the tray shortcut.
- Edit title, content, and color inline.
- Open a note in a dedicated pop-out window and verify updates persist back to SQLite.
- Confirm external Markdown links open via the shell plugin instead of navigating inside the webview.

### Pomodoro

- Save timer durations and milestone settings.
- Start, pause, resume, skip, and reset a cycle.
- Verify the tray tooltip changes during active sessions.
- Confirm milestone status updates between `active`, `completed`, and `cancelled`.

### Settings and Storage

- Update page size, todo display, note display, note folder, and note template.
- Restart the app and confirm settings reload from `app_settings`.
- Verify database directory precedence: `LAZY_TODO_DB_DIR`, config file, then app data dir.

## Documentation Verification

```bash
cd doc
poetry install
poetry run make gettext
poetry run make intl-update
poetry run make html
```

The English source should build to `_build/en/html/`, the Chinese site should build to `_build/zh_CN/html/`, and the language switcher should be present on every page.

---
<!-- PKB-metadata
last_updated: 2026-04-12
commit: f9ba186
updated_by: human+ai
-->
