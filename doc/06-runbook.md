# Lazy Todo App — Runbook

<!-- maintained-by: human+ai -->

## Prerequisites

| Tool | Minimum Version | Check Command |
|------|-----------------|---------------|
| Rust | 1.70+ | `rustc --version` |
| Node.js | 18+ | `node --version` |
| npm | 9+ | `npm --version` |
| Python | 3.12+ | `python3 --version` |
| Poetry | 2.x | `poetry --version` |
| Tauri CLI | 2.x | `npx tauri --version` |

### Platform-Specific Dependencies

**macOS**

```bash
xcode-select --install
```

**Linux**

```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

**Windows**

- Microsoft C++ Build Tools
- WebView2 runtime

## Setup

```bash
git clone https://github.com/walterfan/lazy-todo-app.git
cd lazy-todo-app
npm install
```

Optional developer checks:

```bash
pip install pre-commit
pre-commit install
```

## Run the App

```bash
npm run tauri dev
```

Use a custom DB directory:

```bash
LAZY_TODO_DB_DIR=/tmp/lazy-todo-db npm run tauri dev
```

You can also persist a preferred DB directory in `~/.config/lazy-todo-app/config.json`:

```json
{
  "db_dir": "~/Documents/lazy-todo-db"
}
```

## Build Production Bundles

```bash
npm run tauri build
```

Bundle output:

- macOS: `src-tauri/target/release/bundle/dmg/`
- Linux: `src-tauri/target/release/bundle/deb/` and `src-tauri/target/release/bundle/appimage/`
- Windows: `src-tauri/target/release/bundle/msi/` and `src-tauri/target/release/bundle/nsis/`

Public GitHub release binaries are published by `.github/workflows/release.yml` when a version tag is pushed.

## Build the Documentation Site

```bash
cd doc
poetry install
poetry run make gettext
poetry run make intl-update
poetry run make html
```

Generated outputs:

- English: `doc/_build/en/html/`
- Chinese: `doc/_build/zh_CN/html/`

For a local preview server:

```bash
cd doc
poetry run make serve
```

## Inspect and Manage the Database

| Platform | Default Path |
|----------|--------------|
| macOS | `~/Library/Application Support/com.fanyamin.lazytodoapp/todos.db` |
| Linux | `~/.local/share/com.fanyamin.lazytodoapp/todos.db` |
| Windows | `%APPDATA%\com.fanyamin.lazytodoapp\todos.db` |

Inspect the DB:

```bash
sqlite3 ~/Library/Application\ Support/com.fanyamin.lazytodoapp/todos.db
.tables
SELECT * FROM app_settings;
SELECT * FROM pomodoro_settings;
SELECT * FROM sticky_notes ORDER BY updated_at DESC;
```

Back up the DB:

```bash
cp ~/Library/Application\ Support/com.fanyamin.lazytodoapp/todos.db ~/Desktop/todos-backup.db
```

Reset the DB:

```bash
rm ~/Library/Application\ Support/com.fanyamin.lazytodoapp/todos.db
```

## Verification Commands

```bash
npx tsc --noEmit
cd src-tauri && cargo check
cd src-tauri && cargo clippy
cd src-tauri && cargo test
```

## Debugging Notes

### Frontend

- Use Tauri devtools to inspect React state and console logs.
- Search/display issues usually involve `App.tsx`, `TodoList.tsx`, `NoteList.tsx`, or `SettingsPanel.tsx`.
- Pop-out note issues usually involve `src/main.tsx`, `NoteCard.tsx`, and `NoteWindow.tsx`.

### Backend

- Tray/startup behavior lives in `src-tauri/src/lib.rs`.
- Persistence behavior lives in `src-tauri/src/db.rs`.
- Settings and note-window commands live in `src-tauri/src/commands/app.rs`.

### Common Issues

| Symptom | Cause | Fix |
|---------|-------|-----|
| Main window seems gone | It was hidden to tray on close | Use the tray icon or tray menu to show it |
| Note window opens blank | Missing `noteId` or deleted note | Check the `open_note_window` payload and note existence |
| DB path is unexpected | Env or config override took precedence | Check `LAZY_TODO_DB_DIR` and `~/.config/lazy-todo-app/config.json` |
| Chinese docs do not change | Gettext catalogs were not refreshed | Run `poetry run make intl-update` before `poetry run make html` |
| Sphinx build fails on Mermaid | Missing docs dependencies | Re-run `poetry install` in `doc/` |

---
<!-- PKB-metadata
last_updated: 2026-04-12
commit: f9ba186
updated_by: human+ai
-->
