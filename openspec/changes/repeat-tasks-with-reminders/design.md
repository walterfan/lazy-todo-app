## Context

Lazy Todo App currently stores todos in a single SQLite `todos` table with title, description, priority, completion state, optional deadline, and creation time. The React UI creates, updates, toggles, and deletes one-off todos through Tauri commands, and the backend owns all persistence. There is already a Tauri notification plugin dependency in the project, but todo deadlines do not yet have a reminder lifecycle.

Recurring tasks and reminders touch the data model, command contracts, todo UI, and local notification behavior, so the implementation needs an explicit migration and scheduling approach.

## Goals / Non-Goals

**Goals:**

- Support daily, weekly, monthly, and yearly repeating todos.
- Keep existing one-off todo behavior working without requiring user action after migration.
- Let users configure reminders for one-off and recurring todos.
- Surface overdue, due-now, upcoming, and reminder-due states in the todo list.
- Persist recurrence configuration, reminder configuration, and completion history locally in SQLite.
- Use Tauri commands as the only frontend/backend boundary.

**Non-Goals:**

- Sync recurring tasks or reminders across devices.
- Run reminders while the desktop app is fully closed.
- Support complex recurrence rules such as "third Tuesday", weekdays-only, custom cron expressions, or exception dates in the first slice.
- Add cloud push notifications, email reminders, or calendar integration.
- Replace the existing todo list with a separate calendar view.

## Decisions

### Store recurrence on the todo row, store completion history separately

Add recurrence configuration fields to `todos` and add a `todo_occurrences` table for completed recurring occurrences. A repeating todo remains one active row. When the user completes an occurrence, the backend records the completion in `todo_occurrences`, advances the todo deadline/next due timestamp, and keeps the todo active unless recurrence is disabled or ends.

Alternatives considered:

- Create a new todo row for every future occurrence. This makes the list noisy and requires cleanup of future rows when the user edits the recurrence.
- Keep no occurrence history. This is simpler but loses useful completion evidence and makes future reporting harder.

### Use deadline as the visible due occurrence

The existing `deadline` field remains the user-visible due time for both one-off and repeating todos. Recurrence-specific fields track cadence and anchor information, while list sorting and countdown behavior continue to rely on the current deadline.

Alternatives considered:

- Add a separate `next_due_at` field and leave `deadline` only for one-off todos. This adds avoidable UI/API ambiguity because the existing app already understands deadlines.

### Implement simple cadence calculation in Rust

The backend computes the next occurrence for daily, weekly, monthly, and yearly recurrence. Monthly and yearly calculations clamp to the last valid day of the target month when needed, so a task anchored on January 31 can recur on February 28 or 29.

Alternatives considered:

- Use a new recurrence-rule dependency. This is more powerful than needed for the first slice and increases bundle/dependency surface.

### Reminders are local app reminders

Reminder configuration is stored per todo as a lead time before the current deadline. The app evaluates reminders on startup, todo refresh, todo mutation, and a lightweight foreground timer while the app is running. When a reminder becomes due, the UI surfaces it and the backend may emit a desktop notification through the existing notification plugin.

Alternatives considered:

- Register OS-level scheduled notifications. This would be better while the app is closed but is more platform-specific and harder to make reliable in the initial implementation.

### Additive migration only

SQLite changes must be additive: add guarded columns to `todos`, create `todo_occurrences`, and preserve existing rows as non-recurring todos with reminders disabled.

Alternatives considered:

- Rebuild the `todos` table. This is unnecessary and carries avoidable local data risk.

## Risks / Trade-offs

- [Monthly/yearly date edge cases] → Clamp invalid dates to the last valid calendar day and cover leap-year/month-end cases with Rust tests.
- [Reminder duplication] → Store `last_reminded_at` or equivalent reminder state so the same occurrence is not repeatedly notified on every refresh.
- [Timezone ambiguity] → Treat stored todo deadlines consistently with the app's existing ISO string behavior and use local-time calculations for user-facing reminder checks.
- [Recurring completion semantics may surprise users] → Make the completion action label/status clear for repeating todos, such as advancing to the next due occurrence instead of permanently completing the task.
- [App-closed reminders will not fire] → Document that first-slice reminders are local foreground/startup reminders, and show overdue reminders on next launch.

## Migration Plan

1. Add guarded SQLite migration helpers for recurrence/reminder columns and `todo_occurrences`.
2. Update Rust and TypeScript todo models with optional recurrence/reminder fields.
3. Keep old rows valid by defaulting recurrence to `none` and reminders to disabled.
4. Add recurrence/reminder command behavior behind the existing todo commands first.
5. Update UI forms and list rendering.
6. Add tests before broad manual verification.

Rollback is data-preserving: older builds will ignore new columns but may not understand advanced recurrence/reminder behavior. Do not remove existing todo columns.

## Open Questions

- Should reminder delivery in the first implementation use desktop notifications, in-app badges only, or both?
- Should users be able to pause a recurring task without deleting recurrence settings?
- Should completion history be visible in the UI immediately, or only stored for future stats/export?
