## 1. Data Model And Migration

- [x] 1.1 Add recurrence and reminder fields to Rust todo models and matching TypeScript todo types.
- [x] 1.2 Add an additive SQLite migration for todo recurrence/reminder columns and a `todo_occurrences` completion history table.
- [x] 1.3 Update todo create, update, list, delete, and completion persistence code to read and write the new fields without breaking existing one-off todos.
- [x] 1.4 Add backend tests that verify migration preserves existing todos and default recurrence/reminder values are empty.

## 2. Recurrence Behavior

- [x] 2.1 Implement recurrence calculation utilities for daily, weekly, monthly, and yearly cadences.
- [x] 2.2 Handle month-end and leap-day next occurrence clamping with focused unit tests.
- [x] 2.3 Update todo completion so one-off todos complete normally and recurring todos record the completed occurrence, remain active, and advance to the next deadline.
- [x] 2.4 Add backend tests for recurring completion, recurrence edits, recurrence removal, and one-off completion compatibility.

## 3. Reminder Behavior

- [x] 3.1 Implement reminder state calculation for upcoming, due, already reminded, missed, and overdue todos.
- [x] 3.2 Add commands or command response fields needed by the frontend to fetch due reminders and display reminder state.
- [x] 3.3 Persist reminder delivery state per current occurrence so refreshes do not duplicate notifications.
- [x] 3.4 Reset reminder delivery state when a recurring todo advances to a new occurrence or a reminder configuration changes.
- [x] 3.5 Add backend tests for due reminders, missed reminders on app startup, duplicate notification prevention, completion cleanup, and deletion cleanup.

## 4. Frontend Todo UI

- [x] 4.1 Add recurrence controls to todo create/edit flows with daily, weekly, monthly, yearly, and none options.
- [x] 4.2 Add reminder lead-time controls to todo create/edit flows and validate that reminders require a deadline.
- [x] 4.3 Update todo list rendering to show recurring status, next due deadline, reminder due state, and overdue state.
- [x] 4.4 Update todo hooks and invoke calls to send and receive recurrence and reminder fields.
- [x] 4.5 Preserve existing filtering, pagination, search, pinning, and completion interactions for one-off and recurring todos.

## 5. Local Notification Integration

- [x] 5.1 Add a frontend polling or refresh loop that detects due reminders while the app is running.
- [x] 5.2 Use the existing Tauri notification capability when permission is available and keep in-app reminder state as the fallback.
- [x] 5.3 Mark reminders as delivered only after the app attempts to surface the current occurrence reminder.
- [x] 5.4 Surface missed reminders and overdue todos after app startup or todo list refresh.

## 6. Verification And Documentation

- [x] 6.1 Run Rust formatting and backend tests for todo persistence, recurrence, and reminders.
- [x] 6.2 Run TypeScript checking or the project frontend build to verify UI/type changes.
- [ ] 6.3 Manually verify creating, editing, completing, deleting, and listing one-off and recurring todos with reminders.
- [x] 6.4 Update user-facing docs or release notes for recurring todos and local reminder limitations.
