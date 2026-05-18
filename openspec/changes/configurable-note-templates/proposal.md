## Why

Notes currently support only one inline body template and manual export to a folder. Users need reusable Markdown templates that can populate both title and body, plus automatic Markdown files in a configured folder so notes are available outside the app.

## What Changes

- Add a built-in Daily Note template as the default note creation option.
- Allow users to configure an explicit list of Markdown template file paths in settings.
- Parse user template files by using the first H1 heading as the note title and the remaining Markdown as the note body.
- Keep SQLite as the canonical note store while automatically mirroring notes to Markdown files in the configured notes folder.
- Preserve the existing manual selected-note export behavior.

## Capabilities

### New Capabilities
- `note-template-storage`: Covers configurable note templates, built-in daily note generation, and automatic Markdown folder mirroring for sticky notes.

### Modified Capabilities
- None.

## Impact

- Affects app settings models and SQLite schema migrations in Rust and TypeScript.
- Adds or updates Tauri note commands for template listing and Markdown file mirroring.
- Updates the Notes editor and Settings panel UI.
- Adds Rust tests for template parsing, settings migration, and note mirror behavior.
