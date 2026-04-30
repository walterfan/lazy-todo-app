## Context

The Lazy Todo App is a Tauri v2 desktop app (Rust + React + TypeScript) with SQLite persistence. It currently has two tabs — Todos and Sticky Notes — with a system tray that hides/shows the window. All frontend-backend communication uses `invoke()`. The app already has a tab navigation pattern in `App.tsx`.

Adding a Pomodoro timer requires frontend timer logic, backend persistence for sessions/settings, system notifications, and a compact circular UI that fits the existing dark theme.

## Goals / Non-Goals

**Goals:**
- Provide a fully functional Pomodoro timer with work/break cycling
- Beautiful, compact circular progress ring that fits within the existing 700px-wide layout
- Configurable work, short break, long break durations, and pomodoros-per-cycle
- Persist settings and completed sessions in SQLite
- Show daily pomodoro counts with a 7-day mini bar chart
- System notification when a phase transitions
- Update tray tooltip with current timer state

**Non-Goals:**
- Linking pomodoros to specific todo items (future enhancement)
- Audio alerts (notifications only for v1)
- Detailed analytics beyond 7-day history
- Cloud sync of pomodoro data
- Multiple concurrent timers

## Decisions

### 1. Timer logic: frontend-only with backend persistence

**Decision**: The countdown timer runs entirely in the frontend using `setInterval`. The backend is only called to persist completed sessions and manage settings.

**Rationale**: A timer is inherently a UI concern — it drives visual updates every second. Running the timer in Rust would require constant IPC to update the frontend, adding latency and complexity. Frontend `setInterval` with `Date.now()` drift correction is reliable enough for a Pomodoro timer (precision to ~50ms is fine for minute-scale intervals).

**Alternatives considered**:
- Rust-side timer with events: High IPC overhead for 1/sec updates, over-engineered.
- Web Worker timer: Unnecessary for a desktop app that doesn't throttle background tabs.

### 2. Circular progress ring using SVG

**Decision**: Use an SVG `<circle>` with `stroke-dasharray`/`stroke-dashoffset` for the progress ring. Pure CSS + SVG, no additional library.

**Rationale**: A circular countdown is the most visually intuitive Pomodoro display. SVG circles with dash offset are lightweight, GPU-accelerated, and perfectly themeable with CSS variables. No need for a charting library for this.

**Alternatives considered**:
- Canvas-based timer: Harder to theme, more code for the same visual.
- Linear progress bar: Less visually compelling for a timer.
- Third-party timer component: Unnecessary dependency for a single SVG circle.

### 3. Notification via `tauri-plugin-notification`

**Decision**: Use `tauri-plugin-notification` to send native OS notifications when a phase ends.

**Rationale**: Tauri v2 has a first-party notification plugin that works across macOS, Windows, and Linux. It integrates cleanly — just `sendNotification({ title, body })` from the frontend.

**Alternatives considered**:
- Web Notification API: Blocked in Tauri webview context.
- Custom in-app toast: Not visible when the window is hidden to tray — OS notification is essential.

### 4. SQLite schema for sessions and settings

**Decision**: Two new tables in the existing `todos.db`.

**`pomodoro_settings`** — single-row config table:
```sql
CREATE TABLE IF NOT EXISTS pomodoro_settings (
    id              INTEGER PRIMARY KEY CHECK (id = 1),
    work_minutes    INTEGER NOT NULL DEFAULT 25,
    short_break_min INTEGER NOT NULL DEFAULT 5,
    long_break_min  INTEGER NOT NULL DEFAULT 15,
    rounds_per_cycle INTEGER NOT NULL DEFAULT 4
);
```

**`pomodoro_sessions`** — one row per completed pomodoro:
```sql
CREATE TABLE IF NOT EXISTS pomodoro_sessions (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    completed_at TEXT NOT NULL DEFAULT (datetime('now')),
    duration_min INTEGER NOT NULL
);
```

**Rationale**: Settings use a `CHECK (id = 1)` constraint to enforce single-row. Sessions are append-only with a timestamp, making daily aggregation a simple `GROUP BY date(completed_at)` query.

### 5. Daily stats: 7-day mini bar chart

**Decision**: Display today's count prominently with a small inline SVG bar chart for the past 7 days. No charting library.

**Rationale**: A 7-bar chart is trivial to render with SVG `<rect>` elements. Adding a charting library (recharts, chart.js) for 7 bars is overkill. The compact design fits within the timer panel without needing a separate page.

### 6. Tray tooltip update

**Decision**: Update the tray tooltip text from the frontend by calling a new Tauri command `update_tray_tooltip` when timer state changes.

**Rationale**: The tray is configured in Rust, but the timer state lives in React. A simple invoke command that calls `tray.set_tooltip()` bridges this cleanly.

## Risks / Trade-offs

- **[Risk] `setInterval` drift over long periods** → Mitigation: Use `Date.now()` to compute remaining time rather than decrementing a counter. This self-corrects any drift.
- **[Risk] Notification permission may be denied on some systems** → Mitigation: Gracefully degrade — timer still works without notifications, just log a warning.
- **[Trade-off] Settings in SQLite vs. config file** → SQLite keeps everything in one place and uses the existing `Database` struct. A config file would be simpler but adds a second persistence mechanism.
- **[Trade-off] No audio alert in v1** → Acceptable; system notification is sufficient and avoids audio file management. Can add later.
