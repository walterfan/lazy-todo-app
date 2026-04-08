# Lazy Todo App — Runbook

<!-- maintained-by: human+ai -->

## Prerequisites

| Tool | Minimum Version | Check Command |
|------|----------------|---------------|
| Rust | 1.70+ | `rustc --version` |
| Node.js | 18+ | `node --version` |
| npm | 9+ | `npm --version` |
| Tauri CLI | 2.x | `npx tauri --version` |

### Platform-Specific Dependencies

**macOS**: Xcode Command Line Tools (`xcode-select --install`)

**Linux**: System libraries required by Tauri:
```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

**Windows**: [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) + WebView2 runtime

## Setup

```bash
# Clone
git clone https://github.com/walterfan/lazy-todo-app.git
cd lazy-todo-app

# Install frontend dependencies
npm install

# (Optional) Install pre-commit hooks
pip install pre-commit
pre-commit install
```

## Development

```bash
# Start dev mode (Vite dev server + Tauri with hot reload)
npm run tauri dev

# Start with custom DB location
LAZY_TODO_DB_DIR=/tmp/test-db npm run tauri dev
```

This will:
1. Start Vite dev server on `http://localhost:1420`
2. Compile the Rust backend (first run takes ~60s for dependency compilation)
3. Open the Tauri window with hot-reload enabled

### Frontend Only (no Tauri)

```bash
npm run dev        # Vite dev server at http://localhost:1420
npm run build      # Production build → dist/
npm run preview    # Preview production build
```

### Backend Only (type-check)

```bash
cd src-tauri
cargo check        # Type-check without building
cargo clippy       # Lint check
cargo test         # Run tests (if any)
```

## Building for Production

```bash
# Build distributable app bundle
npm run tauri build
```

Output locations:
- **macOS**: `src-tauri/target/release/bundle/dmg/` (`.dmg` installer)
- **Linux**: `src-tauri/target/release/bundle/deb/` and `appimage/`
- **Windows**: `src-tauri/target/release/bundle/msi/` and `nsis/`

## Database Management

### Location

| Platform | Default Path |
|----------|-------------|
| macOS | `~/Library/Application Support/com.fanyamin.lazytodoapp/todos.db` |
| Linux | `~/.local/share/com.fanyamin.lazytodoapp/todos.db` |
| Windows | `%APPDATA%\com.fanyamin.lazytodoapp\todos.db` |

Override with: `LAZY_TODO_DB_DIR=/custom/path`

### Inspect Database

```bash
# macOS example
sqlite3 ~/Library/Application\ Support/com.fanyamin.lazytodoapp/todos.db

# List tables
.tables

# View todos
SELECT * FROM todos ORDER BY priority;

# View today's Pomodoro sessions
SELECT * FROM pomodoro_sessions WHERE date(completed_at) = date('now');

# Check settings
SELECT * FROM pomodoro_settings;
```

### Backup

```bash
cp ~/Library/Application\ Support/com.fanyamin.lazytodoapp/todos.db ~/Desktop/todos-backup.db
```

### Reset (delete all data)

```bash
rm ~/Library/Application\ Support/com.fanyamin.lazytodoapp/todos.db
# Tables are auto-recreated on next app launch
```

## Type-Checking

```bash
# Frontend TypeScript check
npx tsc --noEmit

# Backend Rust check
cd src-tauri && cargo check

# Both (as pre-commit does)
npx tsc --noEmit && cd src-tauri && cargo clippy
```

## Debugging

### Frontend

- Open browser DevTools in the Tauri window: right-click → "Inspect Element" (available in dev mode)
- Console logs from React hooks appear in DevTools console
- Network tab won't show IPC calls (they're not HTTP); use `console.log` in hooks

### Backend

- Rust `println!` and `eprintln!` output appears in the terminal running `npm run tauri dev`
- For detailed logging, add `env_logger` or `tracing` crate (not currently included)

### Common Issues

| Symptom | Cause | Fix |
|---------|-------|-----|
| `libsqlite3-sys` build fails | Sandboxed environment blocking build script | Run with full permissions or ensure C compiler is available |
| Window doesn't appear | Window was hidden to tray | Click tray icon or check tray menu → "Show/Hide" |
| Pomodoro timer resets when switching tabs | Component was conditionally rendered (unmounted) | Already fixed: uses CSS `display: none` instead of conditional render |
| `Permission denied` on DB path | Custom `LAZY_TODO_DB_DIR` directory doesn't exist or lacks write permission | Create the directory manually: `mkdir -p $LAZY_TODO_DB_DIR` |
| Tray icon looks blurry | Missing `@2x` icon variant | Ensure `icons/128x128@2x.png` exists as 256x256 |
| Notifications don't appear | OS notification permissions not granted | Check system notification settings for the app |

## Pre-Commit Hooks

```bash
# Install
pip install pre-commit
pre-commit install

# Manual run
pre-commit run --all-files
```

Checks performed:
1. `cargo clippy` — Rust linting
2. `cargo test` — Rust unit tests
3. `tsc --noEmit` — TypeScript type checking

---
<!-- PKB-metadata
last_updated: 2026-04-07
commit: 4c09050
updated_by: human+ai
-->
