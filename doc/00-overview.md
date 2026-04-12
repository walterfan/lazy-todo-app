# Lazy Todo App — Project Overview

<!-- maintained-by: human+ai -->

## Purpose

Lazy Todo App is a cross-platform desktop productivity app that combines four closely related capabilities in a single Tauri desktop experience:

1. **Todo management** with priorities, deadlines, countdowns, search, and list/grid display options
2. **Sticky notes** with Markdown rendering, color themes, inline editing, and dedicated pop-out note windows
3. **Pomodoro focus timer** with configurable work/break cycles, milestone tracking, notifications, and weekly stats
4. **Application settings** for display preferences, note templates, storage hints, and persistence behavior

The project is also a practical [Harness Engineering](https://www.fanyamin.com/tech/harness-engineering.html) case study: the codebase is intentionally structured so AI agents can contribute safely by following explicit architecture rules.

## Tech Stack

| Layer | Technology | Role |
|-------|-----------|------|
| Desktop shell | Tauri v2 | Native windowing, tray integration, plugin wiring |
| Frontend | React 18 + TypeScript | Main UI, pop-out note view, local state, timer rendering |
| Backend | Rust | Tauri commands, database access, multi-window orchestration |
| Storage | SQLite via `rusqlite` | Todos, sticky notes, pomodoro state, app settings |
| Notifications | `tauri-plugin-notification` | Native work/break reminders |
| External link handling | `@tauri-apps/plugin-shell` | Open HTTP links outside the webview |
| Markdown | `react-markdown` + `remark-gfm` | Rich note rendering |
| Build | Vite + Cargo | Frontend bundling and native packaging |

## Key Characteristics

- **Single-binary desktop app**: the main window and note windows are packaged together with Tauri.
- **Offline-first**: the app has no backend service dependency; all data is local.
- **Multi-window note experience**: a note can be opened into its own focused window while still sharing the same SQLite store.
- **Configurable persistence**: database location is resolved from `LAZY_TODO_DB_DIR`, then `~/.config/lazy-todo-app/config.json`, then the platform app-data directory.
- **Tray-centric UX**: the main window hides on close, can be toggled from the tray, and exposes a quick "New Note" action.

## Deployment

The app targets macOS, Linux, and Windows. Production bundles are created with `npm run tauri build`, and the repo now includes a GitHub Actions release workflow in `.github/workflows/release.yml` so tagged builds can publish downloadable installers on GitHub Releases.

| Platform | Default Data Directory |
|----------|------------------------|
| macOS | `~/Library/Application Support/com.fanyamin.lazytodoapp/` |
| Linux | `~/.local/share/com.fanyamin.lazytodoapp/` |
| Windows | `%APPDATA%\com.fanyamin.lazytodoapp\` |

## Project Status

- **Version**: 0.1.0
- **Identifier**: `com.fanyamin.lazytodoapp`
- **License**: Apache-2.0
- **Repository**: <https://github.com/walterfan/lazy-todo-app>
- **Documentation root**: `doc/` with bilingual `Sphinx + MyST + sphinx-intl` output

---
<!-- PKB-metadata
last_updated: 2026-04-12
commit: f9ba186
updated_by: human+ai
-->
