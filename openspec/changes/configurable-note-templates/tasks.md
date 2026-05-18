## 1. Data Model And Settings

- [ ] 1.1 Add failing Rust tests for settings migration and note mirror metadata.
- [ ] 1.2 Extend Rust and TypeScript app settings with explicit template file paths.
- [ ] 1.3 Add guarded SQLite migrations for `app_settings.note_template_files_json` and `sticky_notes.file_path`.
- [ ] 1.4 Update sticky note models and database CRUD to read/write mirror file paths.

## 2. Template Backend

- [ ] 2.1 Add failing Rust tests for built-in Daily Note and first-H1 Markdown template parsing.
- [ ] 2.2 Implement note template response types and `list_note_templates` Tauri command.
- [ ] 2.3 Expand supported date placeholders in template title and body.
- [ ] 2.4 Register the new Tauri command.

## 3. Markdown Folder Mirror

- [ ] 3.1 Add failing Rust tests for create/update/delete mirror behavior.
- [ ] 3.2 Write new note mirrors into the configured notes folder after SQLite insert.
- [ ] 3.3 Update the stored mirror path on note updates and keep the same file path.
- [ ] 3.4 Remove the mirrored file on note deletion when present.
- [ ] 3.5 Preserve manual selected-note export behavior.

## 4. Frontend

- [ ] 4.1 Add TypeScript types and hook methods for note templates.
- [ ] 4.2 Update Settings to edit an explicit list of template file paths.
- [ ] 4.3 Update NoteEditor to show a template select list and fill title/body from the selection.
- [ ] 4.4 Wire templates from App into the Notes tab and keep Daily Note as the default option.

## 5. Documentation And Verification

- [ ] 5.1 Update English and Chinese i18n strings.
- [ ] 5.2 Update the data/API documentation for settings, note metadata, and commands.
- [ ] 5.3 Run OpenSpec validation.
- [ ] 5.4 Run frontend typecheck and Rust tests.
- [ ] 5.5 Check edited frontend files with IDE lints.
