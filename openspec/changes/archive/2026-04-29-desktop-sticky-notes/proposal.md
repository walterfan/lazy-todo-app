## Why

The current Lazy Todo App only supports structured todo items with priority and deadlines. Users often need a quick, free-form way to jot down thoughts, meeting notes, or code snippets — like physical sticky notes on a desk. Adding a desktop sticky note feature with Markdown support turns the app into a more versatile personal productivity tool. System tray integration lets users keep notes accessible without cluttering the desktop, making it behave like a native sticky-notes companion app.

## What Changes

- Add a new **Sticky Notes** feature alongside the existing Todo list, allowing users to create, edit, and delete free-form Markdown memos
- Each sticky note has a title, Markdown body, color label, and timestamps
- Sticky notes are persisted in the existing SQLite database via a new `sticky_notes` table
- Add a **Markdown renderer** in the frontend to display formatted note content (headings, lists, code blocks, links, etc.)
- Integrate **system tray** support so the app can minimize to the OS taskbar/menu bar area instead of fully closing
- Add a tray icon with a context menu (Show/Hide window, New Note, Quit)
- Add a **tab-based UI** or sidebar navigation to switch between the Todo list view and the Sticky Notes view
- Support toggling between edit mode (raw Markdown textarea) and preview mode (rendered Markdown)

## Capabilities

### New Capabilities
- `sticky-notes`: CRUD operations for free-form Markdown memo notes with color labels, persisted in SQLite
- `markdown-renderer`: Frontend Markdown rendering with support for common syntax (headings, lists, code, links, emphasis)
- `system-tray`: Minimize-to-tray behavior with tray icon and context menu (Show, Hide, New Note, Quit)

### Modified Capabilities

## Impact

- **Rust backend**: New `sticky_notes` table in SQLite, new Tauri commands for note CRUD (`list_notes`, `add_note`, `update_note`, `delete_note`), system tray setup in `lib.rs`
- **Frontend**: New `StickyNote` components, Markdown rendering library (e.g., `react-markdown`), navigation/tab UI to switch between Todos and Notes views
- **Dependencies**: `tauri-plugin-notification` or tray APIs from Tauri v2, `react-markdown` + `remark-gfm` for frontend rendering
- **Tauri config**: Tray icon configuration, updated capabilities for tray permissions, window close/hide behavior override
- **Existing code**: Minimal impact — Todo functionality stays unchanged; `App.tsx` gains a navigation layer wrapping the existing Todo view
