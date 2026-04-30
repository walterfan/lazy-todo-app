## Why

The Lazy Todo App helps users manage tasks and memos, but lacks a tool to help them stay focused while working on those tasks. A Pomodoro timer addresses this by structuring work into focused intervals (typically 25 min) with short breaks (5 min), scientifically proven to improve concentration and prevent burnout. With daily pomodoro statistics, users can also track their productivity trends over time — all within the same desktop app they already use for task management.

## What Changes

- Add a **Pomodoro Timer** as a new tab alongside Todos and Sticky Notes
- Circular countdown timer display with animated progress ring — compact and beautiful
- Automatic cycling between **Work** and **Break** phases, with an optional long break after N pomodoros
- **Configurable durations**: work time, short break, long break, and pomodoros-until-long-break
- Settings persisted in SQLite so they survive restarts
- Each completed pomodoro is recorded in the database with a timestamp
- **Daily statistics view** showing pomodoro count per day (today prominently, plus a 7-day history bar chart)
- **System notification** when a phase ends (work → break or break → work) via Tauri notification API
- Timer state visible in the **tray tooltip** (e.g., "🍅 Working 12:34" or "☕ Break 3:21")

## Capabilities

### New Capabilities
- `pomodoro-timer`: Core timer logic with work/break cycling, configurable durations, start/pause/reset controls, and phase-end notifications
- `pomodoro-stats`: Daily pomodoro completion records persisted in SQLite, with aggregation for today and recent 7-day history

### Modified Capabilities

## Impact

- **Rust backend**: New `pomodoro_sessions` and `pomodoro_settings` tables in SQLite; new Tauri commands for recording sessions, querying stats, and reading/saving settings
- **Frontend**: New `PomodoroTimer` component with circular progress UI, `PomodoroStats` component for daily chart, `PomodoroSettings` inline config; new tab in the tab bar
- **Dependencies**: `tauri-plugin-notification` for system notifications when a phase completes
- **Tray**: Tooltip updated dynamically to show current timer state
- **Existing code**: Tab type extended from 2 to 3 tabs; no changes to Todo or Notes functionality
