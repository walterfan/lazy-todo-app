## Why

Users often have todos that repeat on a regular cadence, such as daily habits, weekly reviews, monthly bills, and yearly renewals. Today every occurrence must be recreated manually, and deadlines do not provide a dedicated reminder flow, so recurring work is easy to miss or tedious to maintain.

## What Changes

- Add repeat configuration for todos with daily, weekly, monthly, and yearly recurrence.
- Track the next due occurrence for a repeating todo and generate the next occurrence when the current occurrence is completed.
- Preserve normal one-off todos as the default behavior with no migration impact for existing tasks.
- Add reminder metadata for todos so the app can surface due and upcoming work before the deadline.
- Add UI controls for repeat cadence, reminder timing, next due date, and recurrence status.
- Add list behavior that highlights overdue, due-today, upcoming, and repeating tasks without hiding existing priority/deadline behavior.
- Add local persistence for recurrence and reminder configuration in SQLite.

## Capabilities

### New Capabilities

- `recurring-tasks`: Defines repeatable todo behavior, supported cadences, next occurrence calculation, completion behavior, editing behavior, and display expectations.
- `task-reminders`: Defines local reminder metadata, due/upcoming reminder states, reminder visibility, and reminder delivery expectations for todo deadlines and recurring tasks.

### Modified Capabilities

- None. Todo behavior exists in code, but there are no archived baseline todo specs under `openspec/specs/` yet.

## Impact

- Frontend: update todo creation, editing, list display, filtering/search, and status indicators in `src/components/AddTodo.tsx`, `src/components/TodoItem.tsx`, `src/components/TodoList.tsx`, `src/hooks/useTodos.ts`, and `src/types/todo.ts`.
- Backend: update todo models and Tauri commands in `src-tauri/src/models/todo.rs`, `src-tauri/src/commands/todo.rs`, and `src-tauri/src/db.rs`.
- Database: add safe, additive SQLite columns or companion tables for recurrence cadence, recurrence anchor/next due date, reminder lead time, reminder status, and occurrence metadata.
- Scheduling/UI: integrate reminders with app startup/list refresh and optionally with the existing Tauri notification plugin if desktop reminder delivery is implemented in the first slice.
- Tests: add Rust persistence/recurrence calculation tests and focused frontend type coverage through existing TypeScript checks.
- Docs: update `doc/05-data-and-api.md`, `doc/06-workflows.md`, and `AGENTS.md` if new commands or persistent fields become part of the durable project contract.
