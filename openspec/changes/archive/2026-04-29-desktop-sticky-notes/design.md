## Context

The Lazy Todo App is a Tauri v2 desktop application with a Rust backend (SQLite via `rusqlite`) and a React 18 + TypeScript frontend. It currently provides Todo CRUD with priority and countdown features through a single-window UI. There is no system tray integration, no Markdown rendering, and no free-form note-taking capability.

The app uses `invoke()` from `@tauri-apps/api/core` for all frontend-to-backend communication, and all data is persisted in a `todos.db` SQLite file under the OS app data directory.

## Goals / Non-Goals

**Goals:**
- Allow users to create, view, edit, and delete free-form Markdown sticky notes
- Render Markdown content with common syntax support (headings, lists, code blocks, links, emphasis)
- Minimize the app to the system tray instead of closing, with a tray icon and context menu
- Provide a clean navigation between the existing Todo view and the new Notes view
- Keep the existing Todo functionality completely intact

**Non-Goals:**
- Rich text editor (WYSIWYG) — we use plain Markdown editing with preview toggle
- Cloud sync or cross-device sharing of notes
- Pinning sticky notes as always-on-top floating windows (future enhancement)
- Note categories, folders, or tagging system (keep it simple for v1)
- Export/import of notes
- Full-text search across notes (future enhancement)

## Decisions

### 1. SQLite table design for sticky notes

**Decision**: Add a new `sticky_notes` table alongside the existing `todos` table in the same database.

**Rationale**: Reuses the existing `Database` struct with its `Mutex<Connection>`. No need for a separate database file or connection pool. The schema includes a `color` field (string enum) for visual differentiation.

**Schema**:
```sql
CREATE TABLE IF NOT EXISTS sticky_notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL DEFAULT '',
    content TEXT NOT NULL DEFAULT '',
    color TEXT NOT NULL DEFAULT 'yellow',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

**Alternatives considered**:
- Separate SQLite file per note: Adds complexity for file management with no clear benefit.
- JSON file storage: Loses transactional guarantees and query capabilities.

### 2. Markdown rendering library

**Decision**: Use `react-markdown` with `remark-gfm` plugin for GitHub Flavored Markdown support.

**Rationale**: `react-markdown` is the most widely adopted React Markdown renderer (~10M weekly downloads), renders to React elements (no `dangerouslySetInnerHTML`), and supports plugin ecosystem. `remark-gfm` adds tables, strikethrough, task lists, and autolinks.

**Alternatives considered**:
- `marked` + `dangerouslySetInnerHTML`: Security risk from raw HTML injection; requires separate sanitization.
- `markdown-it`: Lower-level, requires more boilerplate to integrate with React.
- Custom renderer: Unnecessary complexity for standard Markdown.

### 3. System tray implementation

**Decision**: Use Tauri v2's built-in `tray` module (`tauri::tray::TrayIconBuilder`) configured in `lib.rs` during app setup.

**Rationale**: Tauri v2 provides native tray support across macOS, Windows, and Linux. The tray icon is configured in Rust with a context menu. Window close event is intercepted to hide instead of destroy.

**Behavior**:
- Clicking the tray icon toggles window visibility (show/hide)
- Context menu provides: "Show/Hide", "New Note", separator, "Quit"
- Window close button hides the window to tray instead of quitting the app
- "Quit" in context menu actually exits the app

**Alternatives considered**:
- `tauri-plugin-positioner` for tray window popup: Over-engineered for this use case; we only need show/hide of the main window.

### 4. Navigation between Todos and Notes

**Decision**: Add a tab bar at the top of the app to switch between "Todos" and "Sticky Notes" views. Use React state for view switching (no router needed).

**Rationale**: The app is a single-window tool. A simple tab/pill navigation keeps the UI lightweight without introducing `react-router`. Both views share the same window and layout structure.

**Alternatives considered**:
- Sidebar navigation: Takes horizontal space in a 900px window; tabs are more space-efficient.
- `react-router`: Overkill for two views in a desktop app with no URL concerns.

### 5. Note editing UX

**Decision**: Each note card has a toggle between "Edit" (raw Markdown textarea) and "Preview" (rendered Markdown) modes. Default view is Preview. Double-click or Edit button enters Edit mode.

**Rationale**: Keeps the UI simple while giving full control over Markdown content. Users familiar with Markdown can write quickly; the preview confirms formatting.

## Risks / Trade-offs

- **[Risk] System tray behavior differs across OS** → Mitigation: Tauri v2 abstracts most platform differences. Test on macOS and Windows. Linux tray support varies by desktop environment but Tauri handles common cases.
- **[Risk] Large Markdown content could slow rendering** → Mitigation: Notes are short memos by design. If needed, add lazy rendering or content length limits in future.
- **[Risk] Window close override may confuse users** → Mitigation: Show a brief notification on first minimize-to-tray ("App minimized to tray"). Provide clear "Quit" option in tray menu and consider adding a setting to choose close behavior.
- **[Trade-off] No syntax highlighting in code blocks** → Accept for v1. Can add `rehype-highlight` plugin later with minimal effort.
- **[Trade-off] Tab navigation instead of sidebar** → Limits future expansion to many sections, but sufficient for two views.
