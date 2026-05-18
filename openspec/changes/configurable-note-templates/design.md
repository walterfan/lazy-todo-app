## Context

Lazy Todo App stores sticky notes in SQLite and exposes note CRUD through Rust Tauri commands. The frontend currently applies one settings-backed Markdown body template when the note editor expands, and selected notes can be manually exported to a configured folder. Users want multiple Markdown templates that populate both title and body, and they want notes automatically stored as Markdown files in a settings-selected folder while keeping the app's current SQLite behavior.

## Goals / Non-Goals

**Goals:**
- Keep SQLite as the canonical store for app note listing, editing, search, and window behavior.
- Add a built-in Daily Note template and an explicit settings-backed list of user template file paths.
- Parse user templates from Markdown using the first H1 as the note title and the remaining Markdown as the note body.
- Mirror notes to stable Markdown files in the configured note folder after create, update, and delete operations.
- Preserve the existing manual export command for selected notes.

**Non-Goals:**
- Replace SQLite with folder-primary Markdown storage.
- Import or sync edits made directly to mirrored Markdown files.
- Add a template-folder scanner or rich template language.
- Add a file picker dependency; paths remain text-configured in settings.

## Decisions

### 1. SQLite remains canonical

The app will continue using `sticky_notes` as the source of truth. Mirrored Markdown files are an interoperability output, not the authoritative store. This avoids changing existing search, pagination, pinning, note windows, and Agent context behavior.

Alternatives considered:
- Folder-primary Markdown notes: more portable, but it would require import/sync conflict handling and larger UI/API changes.
- Export-only folder use: simpler, but does not satisfy automatic storage into the configured folder.

### 2. Store template paths as JSON in app settings

Add `note_template_files_json` to `app_settings`, surfaced as `note_template_files: Vec<String>` in Rust and `string[]` in TypeScript. Keep the legacy `note_template` column for migration and compatibility, but stop using it for new UI.

Alternatives considered:
- One template folder: convenient, but the user selected explicit files.
- Separate template table: unnecessary for a small settings list.

### 3. Store each note mirror path

Add nullable `file_path` to `sticky_notes` so the app can update the same Markdown file when a note title changes. New notes without a configured folder still work in SQLite and receive no mirror path until they are saved with a configured folder.

Alternatives considered:
- Recompute file names from title on every update: simple but leaves stale files after title edits.
- Write all notes to timestamped files on every update: avoids collisions but produces duplicates.

### 4. Parse simple Markdown templates

Template listing will return a built-in Daily Note plus each configured file that can be read. For user files, the first `# Heading` line becomes the note title, and the remaining Markdown becomes the body. If a file has no H1, its file stem becomes the template label and title fallback while the whole file remains the body. The built-in and parsed templates support simple date placeholders such as `{{date}}`, `{{datetime}}`, and `{{weekday}}`.

Alternatives considered:
- YAML frontmatter: explicit and flexible, but the user selected first-heading parsing.
- JSON templates: structured, but less convenient for Markdown authoring.

## Risks / Trade-offs

- [Risk] Mirrored files can become stale if edited outside the app. Mitigation: document that SQLite remains canonical and mirrored files are app-written output.
- [Risk] File writes can fail due to permissions or missing folders. Mitigation: Tauri commands return `Result<T, String>` and only fail the note operation when the configured mirror cannot be written.
- [Risk] Deleting mirrored files could remove a user-edited file. Mitigation: only delete the stored path that was generated for that note, and ignore missing files.
- [Trade-off] Text path configuration avoids new dependencies but is less ergonomic than a native picker.

## Migration Plan

- Add SQLite columns with guarded `ALTER TABLE` statements for existing local databases.
- Map missing `note_template_files_json` to an empty list.
- Preserve existing `note_template` data in the database but do not surface it as a selectable template unless a future migration explicitly imports it.
- If rollback is needed, older builds ignore the added columns and continue using SQLite note content.

## Open Questions

- None. The user selected automatic SQLite-plus-Markdown mirroring and explicit template file paths.
