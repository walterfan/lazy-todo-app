# Lazy Todo App — Project Overview

<!-- maintained-by: human+ai -->

## Purpose

Lazy Todo App is a cross-platform desktop productivity application that combines three tools into one lightweight window:

1. **Todo Management** — task tracking with priority levels and countdown timers
2. **Desktop Sticky Notes** — Markdown-formatted memos with color coding
3. **Pomodoro Timer** — configurable work/rest cycles with daily statistics and alerts

The project serves as a practical case study for [Harness Engineering](https://www.fanyamin.com/tech/harness-engineering.html) — demonstrating how a developer unfamiliar with Rust can direct AI agents to build a full Tauri application by providing architectural constraints rather than writing code directly.

## Tech Stack

| Layer | Technology | Role |
|-------|-----------|------|
| Frontend | React 18 + TypeScript | UI components, state hooks, timer logic |
| Backend | Rust + Tauri v2 | IPC commands, state management, system tray |
| Storage | SQLite via `rusqlite` (bundled) | Local persistence (4 tables) |
| Notifications | `tauri-plugin-notification` | Native OS alerts |
| Markdown | `react-markdown` + `remark-gfm` | Sticky note content rendering |
| Build | Vite (frontend) + Cargo (backend) | Development and production builds |

## Key Characteristics

- **Single-binary desktop app** — Tauri bundles a native webview; no Electron overhead.
- **Offline-first** — all data lives in a local SQLite file; no network dependency.
- **Configurable storage** — database directory overridable via `LAZY_TODO_DB_DIR` environment variable.
- **System tray integration** — the app minimizes to the OS tray on close; tray icon provides quick actions (show/hide, new note, quit).
- **Dark theme UI** — CSS custom properties throughout `App.css`.

## Deployment

This is a desktop application targeting macOS, Linux, and Windows. It is distributed as a native installer built by `npm run tauri build`. No server infrastructure is required.

| Platform | Default Data Directory |
|----------|----------------------|
| macOS | `~/Library/Application Support/com.fanyamin.lazytodoapp/` |
| Linux | `~/.local/share/com.fanyamin.lazytodoapp/` |
| Windows | `%APPDATA%\com.fanyamin.lazytodoapp\` |

## Project Status

- **Version**: 0.1.0
- **Identifier**: `com.fanyamin.lazytodoapp`
- **License**: Apache-2.0
- **Repository**: <https://github.com/walterfan/lazy-todo-app>

---
<!-- PKB-metadata
last_updated: 2026-04-07
commit: 4c09050
updated_by: human+ai
-->
