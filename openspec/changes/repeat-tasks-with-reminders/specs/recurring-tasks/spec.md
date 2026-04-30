## ADDED Requirements

### Requirement: Create a recurring todo
The system SHALL allow users to create a todo with no recurrence or with a daily, weekly, monthly, or yearly recurrence cadence.

#### Scenario: Create daily recurring todo
- **WHEN** the user creates a todo with recurrence set to daily and a deadline
- **THEN** the system persists the todo as recurring with the deadline as the first due occurrence

#### Scenario: Create weekly recurring todo for a selected weekday
- **WHEN** the user creates a todo with recurrence set to weekly and selects a weekday
- **THEN** the system persists the selected weekday and advances future occurrences to that weekday

#### Scenario: Create monthly recurring todo for a selected day
- **WHEN** the user creates a todo with recurrence set to monthly and selects a day of month
- **THEN** the system persists the selected day and advances future occurrences to that day, clamping to month end when needed

#### Scenario: Create one-off todo
- **WHEN** the user creates a todo without selecting a recurrence cadence
- **THEN** the system persists the todo as a one-off todo with existing todo behavior

### Requirement: Edit recurrence settings
The system SHALL allow users to change or remove recurrence settings on an existing todo without losing title, description, priority, or deadline data.

#### Scenario: Change weekly task to monthly
- **WHEN** the user edits a weekly recurring todo and changes the cadence to monthly
- **THEN** the system updates the recurrence cadence and keeps the todo in the list with the recalculated future schedule

#### Scenario: Change recurrence schedule details
- **WHEN** the user edits the weekday for a weekly todo or the day of month for a monthly todo
- **THEN** the system updates the stored schedule details and uses them for future occurrences

#### Scenario: Remove recurrence
- **WHEN** the user removes recurrence from a recurring todo
- **THEN** the system treats the todo as a one-off todo from that point forward

### Requirement: Complete a recurring occurrence
The system SHALL record completion of the current recurring occurrence and advance the todo to the next due occurrence instead of permanently completing the todo.

#### Scenario: Complete daily recurring todo
- **WHEN** the user completes a daily recurring todo due today
- **THEN** the system records the completed occurrence and updates the todo deadline to the next daily due date

#### Scenario: Complete yearly recurring todo
- **WHEN** the user completes a yearly recurring todo
- **THEN** the system records the completed occurrence and updates the todo deadline to the next yearly due date

### Requirement: Preserve one-off completion behavior
The system SHALL preserve existing completion behavior for todos with no recurrence.

#### Scenario: Complete non-recurring todo
- **WHEN** the user completes a one-off todo
- **THEN** the system marks the todo completed and does not generate a new due occurrence

### Requirement: Calculate calendar-safe next occurrences
The system SHALL calculate recurring due dates for daily, weekly, monthly, and yearly cadences using calendar-safe local date rules.

#### Scenario: Monthly recurrence from month end
- **WHEN** a monthly recurring todo is anchored on a day that does not exist in the target month
- **THEN** the system uses the last valid day of the target month for the next occurrence

#### Scenario: Yearly recurrence from leap day
- **WHEN** a yearly recurring todo is anchored on February 29 and the next year is not a leap year
- **THEN** the system uses February 28 for the next occurrence

### Requirement: Display recurring task state
The system SHALL show recurring tasks in the todo list with enough visual state to distinguish them from one-off todos.

#### Scenario: Recurring todo in list
- **WHEN** the todo list contains a recurring task
- **THEN** the task row shows its recurrence cadence and current due state

#### Scenario: Recurring todo schedule in list
- **WHEN** the todo list contains a weekly or monthly recurring task with explicit schedule details
- **THEN** the task row shows the selected weekday or day of month alongside the cadence

#### Scenario: Search includes recurring tasks
- **WHEN** the user searches todos
- **THEN** recurring todos are included in search results using the same title and description matching as one-off todos
