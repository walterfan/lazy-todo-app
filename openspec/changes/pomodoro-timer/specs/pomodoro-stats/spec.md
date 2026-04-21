## ADDED Requirements

### Requirement: Record completed pomodoro sessions
The system SHALL insert a row into `pomodoro_sessions` each time a work phase completes successfully (countdown reaches 00:00). The record SHALL include the completion timestamp and the work duration in minutes.

#### Scenario: Record a completed pomodoro
- **WHEN** a 25-minute work countdown reaches 00:00
- **THEN** a new row is inserted with `completed_at` = current datetime and `duration_min` = 25

#### Scenario: Paused and resumed pomodoro still records
- **WHEN** a work phase is paused, resumed, and eventually completes
- **THEN** a session is recorded with the configured work duration (not elapsed wall time)

### Requirement: Display today's pomodoro count
The system SHALL display the number of pomodoros completed today prominently in the timer panel. The count SHALL update in real-time when a new pomodoro is completed.

#### Scenario: Show today's count
- **WHEN** user has completed 3 pomodoros today
- **THEN** the timer panel displays "Today: 3 🍅"

#### Scenario: Count resets at midnight
- **WHEN** the date changes to the next day
- **THEN** today's count starts at 0

### Requirement: Display 7-day pomodoro history
The system SHALL display a mini bar chart showing the number of pomodoros per day for the past 7 days (including today). Each bar SHALL be labeled with the day abbreviation. The chart SHALL be compact and fit below the timer.

#### Scenario: 7-day chart with data
- **WHEN** user views the Pomodoro tab
- **THEN** a bar chart shows 7 bars (Mon-Sun or last 7 dates)
- **AND** each bar height is proportional to the pomodoro count for that day
- **AND** today's bar is visually highlighted

#### Scenario: Days with no pomodoros
- **WHEN** a day in the 7-day range has 0 completed pomodoros
- **THEN** that day's bar has minimal/zero height but the label is still shown

### Requirement: Backend commands for statistics
The system SHALL expose Tauri commands:
- `get_today_pomodoro_count` → `i64` (count for today)
- `get_weekly_pomodoro_stats` → `Vec<DayStat>` (count per day for last 7 days)
- `record_pomodoro_session(duration_min: i64)` → `()`

#### Scenario: Get today's count from backend
- **WHEN** frontend invokes `get_today_pomodoro_count`
- **THEN** backend returns the count of rows in `pomodoro_sessions` where `date(completed_at)` = today

#### Scenario: Get weekly stats from backend
- **WHEN** frontend invokes `get_weekly_pomodoro_stats`
- **THEN** backend returns an array of 7 objects `{ date: string, count: i64 }` for the past 7 days, filling 0 for days with no data

### Requirement: Backend commands for settings
The system SHALL expose Tauri commands:
- `get_pomodoro_settings` → `PomodoroSettings`
- `save_pomodoro_settings(settings: PomodoroSettings)` → `()`

The settings table SHALL be initialized with defaults (25/5/15/4) if no row exists.

#### Scenario: Get settings when none exist
- **WHEN** frontend invokes `get_pomodoro_settings` for the first time
- **THEN** backend inserts default settings (work=25, short_break=5, long_break=15, rounds=4) and returns them

#### Scenario: Save updated settings
- **WHEN** frontend invokes `save_pomodoro_settings` with work=30
- **THEN** the settings row is updated with work_minutes=30
