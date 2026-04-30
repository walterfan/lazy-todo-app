## ADDED Requirements

### Requirement: User can create a sticky note
The system SHALL allow users to create a new sticky note with a title and Markdown content body. The system SHALL assign a default color of "yellow" if no color is specified. The system SHALL persist the note to the SQLite database.

#### Scenario: Create a note with title and content
- **WHEN** user fills in a title and Markdown content and clicks "Add Note"
- **THEN** the system creates a new sticky note in the database with the provided title, content, default color "yellow", and current timestamp for both `created_at` and `updated_at`
- **AND** the new note appears in the sticky notes list

#### Scenario: Create a note with a selected color
- **WHEN** user fills in a title and content and selects a color (e.g., "blue") before clicking "Add Note"
- **THEN** the system creates the note with the selected color "blue"

#### Scenario: Create a note with empty title
- **WHEN** user submits a note with an empty title but non-empty content
- **THEN** the system creates the note with an empty title (title is optional for quick memos)

### Requirement: User can list all sticky notes
The system SHALL display all sticky notes in reverse chronological order (newest first) based on `updated_at`. Each note SHALL show its title, a preview of its rendered Markdown content, its color label, and its last-updated time.

#### Scenario: Display notes list
- **WHEN** user navigates to the Sticky Notes tab
- **THEN** the system fetches and displays all notes ordered by `updated_at` descending
- **AND** each note card shows the title, a truncated rendered preview, color indicator, and relative time since last update

#### Scenario: No notes exist
- **WHEN** user navigates to Sticky Notes and no notes exist in the database
- **THEN** the system displays an empty state message encouraging the user to create their first note

### Requirement: User can view a sticky note in full
The system SHALL allow users to expand a note card to see the full rendered Markdown content in preview mode.

#### Scenario: View full note content
- **WHEN** user clicks on a note card
- **THEN** the note expands to show the full rendered Markdown content in preview mode

### Requirement: User can edit a sticky note
The system SHALL allow users to edit the title, content, and color of an existing sticky note. The system SHALL update the `updated_at` timestamp on save.

#### Scenario: Edit note content
- **WHEN** user clicks the "Edit" button on a note
- **THEN** the note switches to edit mode showing the raw Markdown in a textarea and the title in an input field
- **AND** when the user modifies content and clicks "Save"
- **THEN** the system updates the note in the database with the new content and a refreshed `updated_at` timestamp

#### Scenario: Change note color
- **WHEN** user selects a different color for a note
- **THEN** the system updates the note's color in the database and the UI reflects the new color immediately

#### Scenario: Cancel editing
- **WHEN** user clicks "Cancel" during editing
- **THEN** changes are discarded and the note returns to preview mode with its original content

### Requirement: User can delete a sticky note
The system SHALL allow users to delete a sticky note permanently after confirmation.

#### Scenario: Delete a note
- **WHEN** user clicks the "Delete" button on a note and confirms the deletion
- **THEN** the system removes the note from the database and it disappears from the list

#### Scenario: Cancel deletion
- **WHEN** user clicks "Delete" but cancels the confirmation
- **THEN** the note remains unchanged

### Requirement: Sticky note color options
The system SHALL support the following color options for notes: "yellow", "green", "blue", "pink", "purple", "orange". Each color SHALL be represented visually on the note card as a background tint or accent.

#### Scenario: Color visual display
- **WHEN** a note has color "green"
- **THEN** the note card displays with a green-tinted background or green accent border

### Requirement: Sticky notes backend commands
The system SHALL expose the following Tauri commands for sticky note operations, all returning `Result<T, String>`:
- `list_notes` → `Vec<StickyNote>`
- `add_note(input: CreateNote)` → `StickyNote`
- `update_note(input: UpdateNote)` → `StickyNote`
- `delete_note(id: i64)` → `()`

#### Scenario: Backend list_notes returns ordered notes
- **WHEN** the frontend invokes `list_notes`
- **THEN** the backend returns all sticky notes from the database ordered by `updated_at` descending

#### Scenario: Backend add_note persists and returns
- **WHEN** the frontend invokes `add_note` with title and content
- **THEN** the backend inserts a new row into `sticky_notes` and returns the created `StickyNote` with its generated `id`
