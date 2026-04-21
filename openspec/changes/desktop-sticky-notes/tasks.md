## 1. Database & Rust Models

- [x] 1.1 Add `sticky_notes` table creation SQL to `src-tauri/src/db.rs` (id, title, content, color, created_at, updated_at)
- [x] 1.2 Create `StickyNote`, `CreateNote`, `UpdateNote` structs in `src-tauri/src/models/` with serde derive
- [x] 1.3 Implement `db.rs` CRUD functions: `insert_note`, `list_notes`, `update_note`, `delete_note` using rusqlite

## 2. Tauri Backend Commands

- [x] 2.1 Create `src-tauri/src/commands/note.rs` with `list_notes`, `add_note`, `update_note`, `delete_note` commands (all `Result<T, String>`)
- [x] 2.2 Register the new note commands in the `invoke_handler` in `src-tauri/src/lib.rs`

## 3. System Tray Integration

- [x] 3.1 Add tray icon asset (32x32 PNG) to `src-tauri/icons/` for the system tray
- [x] 3.2 Implement system tray setup in `src-tauri/src/lib.rs` using `TrayIconBuilder` with icon and tooltip
- [x] 3.3 Create tray context menu with "Show/Hide", "New Note", separator, and "Quit" items
- [x] 3.4 Implement tray icon left-click handler to toggle main window visibility (show/hide)
- [x] 3.5 Implement tray menu event handlers: Show/Hide toggles window, New Note emits event to frontend, Quit exits app
- [x] 3.6 Override window close event to hide window instead of destroying the app (close-requested event)
- [x] 3.7 Update `src-tauri/capabilities/default.json` to include tray-related permissions

## 4. Frontend Dependencies & Types

- [x] 4.1 Install `react-markdown` and `remark-gfm` npm packages
- [x] 4.2 Create `StickyNote`, `CreateNote`, `UpdateNote` TypeScript types in `src/types/note.ts`

## 5. Frontend Hooks & API

- [x] 5.1 Create `src/hooks/useNotes.ts` hook with `loadNotes`, `addNote`, `updateNote`, `deleteNote` using `invoke()`
- [x] 5.2 Add event listener for the "new-note" tray event to switch to Notes tab and focus create form

## 6. Sticky Notes UI Components

- [x] 6.1 Create `src/components/NoteEditor.tsx` — form for creating a new sticky note (title, content textarea, color picker)
- [x] 6.2 Create `src/components/NoteCard.tsx` — individual note card with color accent, title, Markdown preview, edit/delete buttons
- [x] 6.3 Create `src/components/NoteDetail.tsx` — expanded note view with edit/preview toggle using `react-markdown` + `remark-gfm`
- [x] 6.4 Create `src/components/NoteList.tsx` — grid/list layout of NoteCard components with empty state
- [x] 6.5 Create `src/components/MarkdownPreview.tsx` — reusable Markdown renderer component wrapping `react-markdown`

## 7. Navigation & App Layout

- [x] 7.1 Add tab navigation component to `App.tsx` with "Todos" and "Sticky Notes" tabs
- [x] 7.2 Implement view switching state in `App.tsx` to conditionally render Todo view or Notes view
- [x] 7.3 Style the tab bar to match the existing app design (colors, spacing, active state)

## 8. Styling

- [x] 8.1 Add sticky note card styles with color variants (yellow, green, blue, pink, purple, orange) to `App.css`
- [x] 8.2 Style the Markdown rendered content (headings, code blocks, lists, tables, links) within note cards
- [x] 8.3 Style the note editor form (textarea height, color picker buttons)
- [x] 8.4 Add responsive grid layout for note cards

## 9. Tauri Configuration

- [x] 9.1 Update `src-tauri/tauri.conf.json` to configure tray icon path if needed
- [x] 9.2 Verify `Cargo.toml` has necessary Tauri features enabled for tray support

## 10. Testing & Verification

- [x] 10.1 Verify sticky note CRUD works end-to-end (create, list, edit, delete)
- [x] 10.2 Verify Markdown rendering with headings, lists, code blocks, links, and GFM tables
- [x] 10.3 Verify system tray icon appears on app launch
- [x] 10.4 Verify window close hides to tray instead of quitting
- [x] 10.5 Verify tray context menu actions (Show/Hide, New Note, Quit)
- [x] 10.6 Verify tab navigation switches between Todos and Sticky Notes without losing state
