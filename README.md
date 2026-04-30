# Lazy Todo App

[English Version](README.md) | [Chinese Version](README_zh.md)

A cross-platform desktop productivity app built with **Tauri v2 + Rust + React + TypeScript**, combining todo management, sticky notes, Pomodoro focus tools, developer utilities, local AI Agents, and app settings in one native shell.

This project also serves as a practical [Harness Engineering](https://www.fanyamin.com/tech/harness-engineering.html) case study: AI agents contribute inside explicit architectural guardrails instead of writing code with no boundaries.

## Features

### Todo Management

- **Todo CRUD**: add, edit, complete, and delete tasks.
- **Priority and deadlines**: supports high/medium/low priority plus live countdowns.
- **Recurring tasks**: daily, weekly, monthly, and yearly tasks advance to the next due occurrence when completed, with explicit weekday and day-of-month controls.
- **Local reminders**: reminder lead times surface in the list and can trigger desktop notifications while the app is running.
- **Search and display modes**: task search plus list/grid rendering modes.

### Sticky Notes

- **Markdown notes**: supports GitHub Flavored Markdown rendering.
- **Multi-color cards**: supports multiple note colors and inline editing.
- **Pop-out windows**: individual notes can open in dedicated windows while sharing the same data store.
- **Tray shortcut creation**: quick note creation is available from the system tray.

### Pomodoro

- **Visual timer**: SVG ring progress and phase countdown.
- **Configurable cycles**: supports work, short break, long break, and round settings.
- **Milestones and stats**: supports milestone tracking, daily completions, and 7-day stats.
- **Alerts**: supports window alerts, sound cues, and native notifications.

### Toolbox

- **Conversion**: Base64, Hex ↔ ASCII, URL, HTML escape, number base, timestamp (seconds / milliseconds, batch input), and JWT encode/decode (HS256).
- **Checksum**: MD5, SHA-1, SHA-256, SHA-384, SHA-512 via Web Crypto.
- **Generation**: UUID v4, random strings with configurable charset, strong passwords.
- **Encryption**: AES-GCM / AES-CBC (128/192/256-bit keys) plus ROT13, Caesar, Atbash.
- **Fully client-side**: inputs and outputs never leave the app — no persistence, no network.

### AI Agents

- **Agent chat**: talk with bundled local Agents such as Personal Secretary and Confucius from the desktop app.
- **Plugin-based personas**: Agents are loaded from static plugin folders with manifest, prompt, config, avatar, README, and optional RAG knowledge files.
- **Local memory and identity**: manage user identity, durable memories, memory proposals, and previous conversation recall in local SQLite.
- **RAG knowledge**: per-Agent `rag_knowledge.md` files are chunked and retrieved only for the active Agent.
- **Safe tool actions**: Agents can propose todo, note, milestone, file, memory, and external CLI actions, with app-owned confirmation flows for writes or sensitive operations.
- **Plugin management**: refresh, enable, disable, install, inspect, uninstall, and rebuild RAG indexes from Settings.

### Settings and Desktop Experience

- **App settings**: supports page size, todo/note display modes, note templates, and note folder labels.
- **SQLite persistence**: todos, notes, pomodoro data, Agents data, and app settings are all stored locally in SQLite.
- **Configurable DB path**: supports environment-variable and local-config overrides for the database directory.
- **Tray behavior**: closing the main window hides it to the tray, with show/hide and quit actions.

## Tech Stack

| Layer | Technology | Purpose |
|---|---|---|
| Desktop shell | Tauri v2 | Native windows, tray integration, plugin wiring |
| Frontend | React 18 + TypeScript | Main UI, search, settings, countdowns, pop-out note windows |
| Backend | Rust | Tauri commands, window management, state, and DB access |
| Storage | SQLite via `rusqlite` | Local persistence for todos, sticky notes, Pomodoro, Agents, and settings |
| Notifications | `tauri-plugin-notification` | Native system alerts |
| External links | `@tauri-apps/plugin-shell` | Opens HTTP links outside the webview |
| Markdown | `react-markdown` + `remark-gfm` | Note content rendering |
| Build | Vite + Cargo | Frontend bundling and desktop packaging |

## Quick Start

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) `1.70+`
- [Node.js](https://nodejs.org/) `18+`
- macOS / Linux / Windows

### Install and Run

```bash
git clone https://github.com/walterfan/lazy-todo-app.git
cd lazy-todo-app
npm install

# Development mode
npm run tauri dev

# Production build
npm run tauri build
```

### Database Location

Default database path:

| Platform | Path |
|---|---|
| macOS | `~/Library/Application Support/com.fanyamin.lazytodoapp/todos.db` |
| Linux | `~/.local/share/com.fanyamin.lazytodoapp/todos.db` |
| Windows | `%APPDATA%\com.fanyamin.lazytodoapp\todos.db` |

Override with an environment variable:

```bash
LAZY_TODO_DB_DIR=/path/to/your/folder npm run tauri dev
```

You can also persist the override via local config:

```json
{
  "db_dir": "~/Documents/lazy-todo-db"
}
```

Config file location:

```text
~/.config/lazy-todo-app/config.json
```

## Documentation

The project knowledge base lives in `doc/` and uses **Sphinx + MyST + sphinx-intl** to generate bilingual documentation.

Published site: [https://walterfan.github.io/lazy-todo-app](https://walterfan.github.io/lazy-todo-app)

### Build Docs Locally

```bash
cd doc
poetry install
poetry run make html
```

Output directories:

- English: `doc/_build/en/html/`
- Chinese: `doc/_build/zh_CN/html/`

### Build the GitHub Pages Site

```bash
cd doc
poetry run make pages
```

Generated output:

- Site root: `doc/_build/site/`
- Landing page: `doc/_build/site/index.html`
- English site: `doc/_build/site/en/`
- Chinese site: `doc/_build/site/zh_CN/`

### Automatic Docs Publishing

The repo includes `/.github/workflows/docs.yml`, which publishes GitHub Pages when:

- Pushes land on `master` or `main` and change `doc/**`, `README.md`, or the docs workflow itself
- The workflow is triggered manually via `workflow_dispatch`

Enable this once in GitHub:

1. Open `Settings -> Pages`
2. Set `Source` to `GitHub Actions`

## Binary Releases

The repo includes `/.github/workflows/release.yml`, which builds Tauri installers and publishes them to GitHub Releases when you push a `v*` tag.

Recommended release helper:

```bash
./scripts/release_version.sh v0.1.1
```

Or:

```bash
npm run release:tag -- v0.1.1
```

This helper updates `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml`, creates a release commit, pushes the current branch, then creates and pushes the release tag.

Example:

```bash
git tag v0.1.1
git push origin v0.1.1
```

For the full release checklist and bilingual docs publish steps, see `doc/08-build.md`.

## Harness Engineering Notes

This project demonstrates how to guide AI coding with rules, constraints, and automation instead of asking a model to generate code with no boundaries.

### `AGENTS.md` as Architectural Guardrails

`AGENTS.md` defines where commands live, how frontend/backend boundaries work, how persistence is handled, and what Tauri commands must return.

### Pre-commit Checks

```bash
pip install pre-commit
pre-commit install
```

Typical checks:

- `cargo clippy`
- `cargo test`
- `tsc --noEmit`

### Tauri as a Natural Sandbox

The frontend can only reach the backend through `invoke()` and cannot directly touch the database or filesystem; the backend then relies on Rust's type system and ownership model to keep those boundaries safe.

## Tauri Commands

| Area | Command | Description |
|---|---|---|
| Todo | `list_todos` | List todos |
| Todo | `add_todo` | Add a todo |
| Todo | `toggle_todo` | Toggle completion |
| Todo | `update_todo` | Update a todo |
| Todo | `delete_todo` | Delete a todo |
| Todo | `list_due_todo_reminders` | List due and overdue todo reminders |
| Todo | `mark_todo_reminded` | Mark a reminder as delivered |
| Notes | `list_notes` | List notes |
| Notes | `add_note` | Add a note |
| Notes | `update_note` | Update a note |
| Notes | `delete_note` | Delete a note |
| Pomodoro | `get_pomodoro_settings` | Get pomodoro settings |
| Pomodoro | `save_pomodoro_settings` | Save pomodoro settings |
| Pomodoro | `record_pomodoro_session` | Record a completed session |
| Pomodoro | `get_today_pomodoro_count` | Get today's count |
| Pomodoro | `get_weekly_pomodoro_stats` | Get weekly stats |
| Pomodoro | `update_tray_tooltip` | Update tray tooltip |
| Agents | `list_agents` | List bundled and installed Agents |
| Agents | `refresh_agents` | Rescan Agent plugins |
| Agents | `start_agent_session` | Start a single-Agent chat |
| Agents | `start_agent_group_session` | Start a group Agent chat |
| Agents | `send_agent_message_stream` | Stream a single-Agent reply |
| Agents | `send_agent_group_message_stream` | Stream group Agent replies |
| Agents | `get_agent_plugin_detail` | Inspect Agent plugin metadata, README, diagnostics, and RAG status |
| Agents | `rebuild_agent_rag_index` | Rebuild RAG knowledge for one Agent |
| Agents | `list_agent_memories` | List local Agent memories |
| Agents | `confirm_agent_tool_action` | Confirm or reject proposed write/tool actions |
| Agents | `list_agent_external_cli_tools` | Manage registered external CLI tools |
| App | `get_db_path` | Get DB path |
| App | `get_app_settings` | Get app settings |
| App | `save_app_settings` | Save app settings |
| App | `quit_app` | Quit app |
| App | `open_note_window` | Open a dedicated note window |

## Related Links

- [从 Prompt Engineering 到 Harness Engineering：AI 编程的四次进化](https://www.fanyamin.com/tech/harness-engineering.html)
- [Harness Engineering: Leveraging Codex in an Agent-First World](https://openai.com/index/harness-engineering/)
- [Tauri v2 Documentation](https://v2.tauri.app/)

## License

Apache-2.0
