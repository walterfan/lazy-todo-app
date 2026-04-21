## 1. Database Schema & Rust Models

- [x] 1.1 Add `pomodoro_settings` table (id, work_minutes, short_break_min, long_break_min, rounds_per_cycle) with single-row CHECK constraint to `db.rs`
- [x] 1.2 Add `pomodoro_sessions` table (id, completed_at, duration_min) to `db.rs`
- [x] 1.3 Create `src-tauri/src/models/pomodoro.rs` with `PomodoroSettings`, `DayStat` structs (serde)
- [x] 1.4 Implement `db.rs` functions: `get_pomodoro_settings`, `save_pomodoro_settings`, `record_pomodoro_session`, `get_today_pomodoro_count`, `get_weekly_pomodoro_stats`
- [x] 1.5 Register `pomodoro` module in `models/mod.rs`

## 2. Tauri Backend Commands

- [x] 2.1 Create `src-tauri/src/commands/pomodoro.rs` with commands: `get_pomodoro_settings`, `save_pomodoro_settings`, `record_pomodoro_session`, `get_today_pomodoro_count`, `get_weekly_pomodoro_stats`
- [x] 2.2 Add `update_tray_tooltip` command to update tray tooltip text from frontend
- [x] 2.3 Register pomodoro commands and `update_tray_tooltip` in `lib.rs` invoke_handler
- [x] 2.4 Register `pomodoro` module in `commands/mod.rs`

## 3. Notification Plugin

- [x] 3.1 Add `tauri-plugin-notification` to `Cargo.toml` dependencies
- [x] 3.2 Register notification plugin in `lib.rs` builder
- [x] 3.3 Install `@tauri-apps/plugin-notification` npm package
- [x] 3.4 Add notification permission to `capabilities/default.json`

## 4. Frontend Types & Hooks

- [x] 4.1 Create `src/types/pomodoro.ts` with `PomodoroSettings`, `DayStat`, `TimerPhase`, `TimerState` types
- [x] 4.2 Create `src/hooks/usePomodoro.ts` with timer logic (start, pause, reset, phase cycling, auto-transition), settings loading, and session recording via invoke
- [x] 4.3 Create `src/hooks/usePomodoroStats.ts` with today's count and weekly stats fetching

## 5. UI Components

- [x] 5.1 Create `src/components/PomodoroRing.tsx` — SVG circular progress ring with MM:SS center text, phase label, and round indicator
- [x] 5.2 Create `src/components/PomodoroControls.tsx` — Start/Pause/Resume/Reset buttons
- [x] 5.3 Create `src/components/PomodoroSettings.tsx` — inline config form for work/break durations and rounds-per-cycle
- [x] 5.4 Create `src/components/PomodoroStats.tsx` — today's count display + 7-day SVG mini bar chart
- [x] 5.5 Create `src/components/PomodoroPanel.tsx` — layout wrapper composing Ring, Controls, Stats, and Settings toggle

## 6. App Integration

- [x] 6.1 Add "Pomodoro" tab to `App.tsx` tab bar (extend Tab type to include "pomodoro")
- [x] 6.2 Render `PomodoroPanel` when pomodoro tab is active
- [x] 6.3 Wire tray tooltip update on timer state changes

## 7. Styling

- [x] 7.1 Add Pomodoro ring styles — circular SVG animation, color by phase (red for work, green for break), pulsing animation when timer ends
- [x] 7.2 Add control buttons styles — compact pill buttons matching app theme
- [x] 7.3 Add settings form styles — inline, compact, same input styling as existing forms
- [x] 7.4 Add stats bar chart styles — mini bars with day labels, highlight for today
- [x] 7.5 Add panel layout styles — centered vertical layout, compact spacing

## 8. Build & Verify

- [x] 8.1 Run `cargo check` to verify Rust compiles without errors
- [x] 8.2 Run `npx tsc --noEmit` to verify TypeScript compiles without errors
- [x] 8.3 Run `npx vite build` to verify frontend bundles successfully
