## ADDED Requirements

### Requirement: User can start a Pomodoro work session
The system SHALL display a circular countdown timer. When the user clicks "Start", the timer SHALL count down from the configured work duration (default 25 minutes). The progress ring SHALL animate smoothly, updating every second.

#### Scenario: Start a work session with default settings
- **WHEN** user clicks the "Start" button on the Pomodoro tab
- **THEN** the timer begins counting down from 25:00
- **AND** the circular progress ring animates from full to empty
- **AND** the phase label displays "Working"

#### Scenario: Start a work session with custom duration
- **WHEN** user has configured work time to 30 minutes and clicks "Start"
- **THEN** the timer begins counting down from 30:00

### Requirement: User can pause and resume the timer
The system SHALL allow pausing the timer. While paused, the countdown freezes and the progress ring stays at its current position. Resume SHALL continue from where it was paused.

#### Scenario: Pause running timer
- **WHEN** the timer is running and user clicks "Pause"
- **THEN** the countdown freezes at the current remaining time
- **AND** the button changes to "Resume"

#### Scenario: Resume paused timer
- **WHEN** the timer is paused and user clicks "Resume"
- **THEN** the countdown resumes from the paused time

### Requirement: User can reset the timer
The system SHALL allow resetting the timer to the beginning of the current phase. Reset SHALL stop the timer and restore the full duration.

#### Scenario: Reset during work phase
- **WHEN** user clicks "Reset" during a work countdown
- **THEN** the timer stops and resets to the full work duration (e.g., 25:00)
- **AND** the progress ring returns to full

### Requirement: Timer automatically transitions between work and break phases
The system SHALL automatically cycle: Work → Short Break → Work → Short Break → ... → Long Break (after N work rounds). After a long break, the cycle restarts. When a phase ends, the next phase SHALL start automatically after a brief pause (3 seconds).

#### Scenario: Work phase ends, short break begins
- **WHEN** the work countdown reaches 00:00 and it is not the Nth round
- **THEN** the system records a completed pomodoro session
- **AND** transitions to "Short Break" phase with the short break duration
- **AND** sends a system notification: "Work complete! Time for a short break."

#### Scenario: Work phase ends after N rounds, long break begins
- **WHEN** the work countdown reaches 00:00 and it is the Nth round (e.g., 4th)
- **THEN** the system records a completed pomodoro session
- **AND** transitions to "Long Break" phase with the long break duration
- **AND** sends a system notification: "Great job! Time for a long break."

#### Scenario: Break phase ends, work begins
- **WHEN** a break countdown (short or long) reaches 00:00
- **THEN** the system transitions to the "Work" phase with the work duration
- **AND** sends a system notification: "Break over! Let's get back to work."

### Requirement: System notification on phase transition
The system SHALL send a native OS notification when any phase (work or break) completes. The notification SHALL include a descriptive title and body text.

#### Scenario: Notification when work ends
- **WHEN** the work countdown reaches 00:00
- **THEN** a system notification is sent with title "Pomodoro Complete" and body "Time for a break!"

#### Scenario: Notification when break ends
- **WHEN** the break countdown reaches 00:00
- **THEN** a system notification is sent with title "Break Over" and body "Ready to focus!"

### Requirement: User can configure timer durations
The system SHALL provide inline settings to configure: work duration (minutes), short break duration (minutes), long break duration (minutes), and number of work rounds per cycle. Settings SHALL be persisted in SQLite and loaded on app start.

#### Scenario: Change work duration
- **WHEN** user changes work duration from 25 to 30 minutes and saves
- **THEN** the setting is persisted in the database
- **AND** the next work phase uses 30 minutes

#### Scenario: Settings persist across app restarts
- **WHEN** user sets work duration to 45 minutes, closes, and reopens the app
- **THEN** the Pomodoro timer shows 45:00 as the work duration

### Requirement: Tray tooltip reflects timer state
The system SHALL update the system tray tooltip to show the current timer phase and remaining time while the timer is running.

#### Scenario: Tray shows running work timer
- **WHEN** the timer is in work phase with 12:34 remaining
- **THEN** the tray tooltip displays "🍅 Working 12:34"

#### Scenario: Tray shows idle state
- **WHEN** the timer is not running
- **THEN** the tray tooltip displays "Lazy Todo App"

### Requirement: Compact and beautiful timer display
The system SHALL display the timer as a circular progress ring with the remaining time in MM:SS format centered inside. The phase label ("Working" / "Short Break" / "Long Break") SHALL appear below the timer. The current round indicator (e.g., "2/4") SHALL be visible. The design SHALL use the app's dark theme colors.

#### Scenario: Visual display during work phase
- **WHEN** the timer is in work phase
- **THEN** the progress ring uses an accent color (e.g., tomato red for work)
- **AND** center text shows MM:SS countdown
- **AND** phase label shows "Working"
- **AND** round indicator shows current/total (e.g., "2/4")
