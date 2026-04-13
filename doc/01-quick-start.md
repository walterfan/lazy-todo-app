# Quick Start

<!-- maintained-by: human+ai -->

This guide helps you get the Lazy Todo App running locally in under 5 minutes.

## Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Node.js | ≥ 18 | [nodejs.org](https://nodejs.org/) |
| pnpm | ≥ 9 | `npm install -g pnpm` |
| Rust | stable | [rustup.rs](https://rustup.rs/) |
| Tauri CLI | ≥ 2.x | `cargo install tauri-cli --version "^2"` |

> **macOS extra**: Xcode Command Line Tools (`xcode-select --install`)
>
> **Linux extra**: see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

## 1. Clone & Install

```bash
git clone https://github.com/user/lazy-todo-app.git
cd lazy-todo-app
pnpm install
```

## 2. Run in Development Mode

```bash
# Start the Tauri dev server (frontend hot-reload + native shell)
pnpm tauri dev
```

This will:
1. Start the Vite dev server on `http://localhost:1420`
2. Compile the Rust backend
3. Open the Tauri window

## 3. Build for Production

```bash
pnpm tauri build
```

The installer/bundle is output to `src-tauri/target/release/bundle/`.

## 4. Run Tests

```bash
# Frontend unit tests
pnpm test

# Rust backend tests
cd src-tauri && cargo test
```

## 5. Verify It Works

1. The app window should open with the todo list UI.
2. Try adding a new todo item.
3. Mark it as complete.
4. Restart the app — your data should persist (stored in local SQLite via `src-tauri`).

## Common Issues

| Symptom | Fix |
|---------|-----|
| `error: failed to run custom build command for webkit2gtk-sys` | Install GTK dev libs: `sudo apt install libgtk-3-dev libwebkit2gtk-4.1-dev` |
| Port 1420 already in use | Kill the process or set `VITE_PORT` in `.env` |
| Rust compilation slow on first run | Normal — subsequent builds use incremental compilation |

## Next Steps

- [Architecture](02-architecture.md) — understand how the pieces fit together
- [Workflows](06-workflows.md) — day-to-day development workflows
- [Build](08-build.md) — detailed build configuration

---
<!-- PKB-metadata
last_updated: 2025-07-14
updated_by: human+ai
-->
