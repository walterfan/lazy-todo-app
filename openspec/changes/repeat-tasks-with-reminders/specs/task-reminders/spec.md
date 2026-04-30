## ADDED Requirements

### Requirement: Configure todo reminders
The system SHALL allow users to configure a reminder for a todo based on a lead time before the todo deadline.

#### Scenario: Add reminder to one-off todo
- **WHEN** the user creates or edits a todo with a reminder lead time and deadline
- **THEN** the system persists the reminder configuration with the todo

#### Scenario: Add reminder to recurring todo
- **WHEN** the user creates or edits a recurring todo with a reminder lead time
- **THEN** the system applies the reminder to the current and future due occurrences

### Requirement: Detect reminder states
The system SHALL identify todos whose reminders are upcoming, due, already reminded, or overdue based on the current time and the todo deadline.

#### Scenario: Reminder becomes due
- **WHEN** the current time reaches the reminder time for an incomplete todo
- **THEN** the system marks the reminder as due for display and notification

#### Scenario: Todo is overdue
- **WHEN** the current time is later than the todo deadline and the todo is incomplete
- **THEN** the system marks the todo as overdue even if the reminder was not previously delivered

### Requirement: Deliver local reminders while app is running
The system SHALL surface due reminders while the desktop app is running through in-app state and, when available, desktop notification delivery.

#### Scenario: App is running when reminder is due
- **WHEN** a reminder becomes due while the app is open
- **THEN** the user receives an in-app reminder state and a desktop notification if notification permission is available

#### Scenario: App starts after reminder time
- **WHEN** the app starts and an incomplete todo has a missed reminder or overdue deadline
- **THEN** the todo list surfaces the missed reminder or overdue state

### Requirement: Avoid duplicate reminder notifications
The system SHALL prevent repeated notification delivery for the same todo occurrence after a reminder has already been delivered.

#### Scenario: Refresh after notification
- **WHEN** the app refreshes todos after a reminder notification has already been delivered for the current occurrence
- **THEN** the system does not deliver the same notification again

#### Scenario: Recurring task advances
- **WHEN** a recurring todo advances to the next occurrence
- **THEN** the system resets reminder delivery state for the new occurrence

### Requirement: Clear reminder state on completion or deletion
The system SHALL clear active reminder state when a one-off todo is completed or deleted, and reset reminder state when a recurring todo advances.

#### Scenario: Complete one-off reminded todo
- **WHEN** the user completes a one-off todo with an active reminder
- **THEN** the reminder state no longer appears for that todo

#### Scenario: Delete reminded todo
- **WHEN** the user deletes a todo with reminder configuration
- **THEN** the system removes reminder state associated with that todo
