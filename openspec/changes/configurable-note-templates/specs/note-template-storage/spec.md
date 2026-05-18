## ADDED Requirements

### Requirement: User can choose a note template
The system SHALL provide a template selection control when creating a new sticky note. The default option SHALL be a built-in Daily Note template.

#### Scenario: Default daily template is available
- **WHEN** the user opens the note creation UI
- **THEN** the template selection includes a Daily Note option selected by default

#### Scenario: Selecting a template fills a blank note draft
- **WHEN** the user selects a template before editing the note draft
- **THEN** the system fills the note title and body from the selected template

### Requirement: User can configure explicit template files
The system SHALL allow users to configure an explicit list of Markdown template file paths in settings. The system SHALL list readable configured files as selectable templates.

#### Scenario: Configured template file appears in selection
- **WHEN** settings contain a readable Markdown template file path
- **THEN** the note creation template selection includes that template

#### Scenario: Unreadable template file does not break notes
- **WHEN** settings contain an unreadable template file path
- **THEN** the system omits that file from the template selection and still shows the built-in Daily Note template

### Requirement: Template files define title and body with first heading
The system SHALL parse a configured Markdown template file by using the first H1 heading as the note title and the remaining Markdown as the note body.

#### Scenario: Markdown template has an H1 heading
- **WHEN** a configured template file contains a first line `# Meeting Notes` followed by body Markdown
- **THEN** the created note draft title is `Meeting Notes`
- **AND** the created note draft body contains the Markdown after that heading

#### Scenario: Markdown template has no H1 heading
- **WHEN** a configured template file has no H1 heading
- **THEN** the system uses the file stem as the template label and title fallback
- **AND** the created note draft body contains the full Markdown file content

### Requirement: Template placeholders are expanded
The system SHALL expand supported date placeholders in built-in and configured templates before filling a note draft.

#### Scenario: Date placeholder expands
- **WHEN** a selected template contains `{{date}}`
- **THEN** the note draft contains the current local date in place of `{{date}}`

### Requirement: Notes are mirrored to configured folder
The system SHALL keep SQLite as the canonical note store and SHALL write a Markdown mirror file for each note when a notes folder is configured in settings.

#### Scenario: New note writes Markdown file
- **WHEN** the user creates a note and settings include a notes folder
- **THEN** the system creates the note in SQLite
- **AND** the system writes a Markdown file for that note in the configured folder

#### Scenario: Updating a note updates same file
- **WHEN** the user updates a note that has a mirrored file path
- **THEN** the system updates the SQLite note
- **AND** the system updates the same Markdown file path

#### Scenario: Deleting a note removes mirrored file
- **WHEN** the user deletes a note that has a mirrored file path
- **THEN** the system deletes the SQLite note
- **AND** the system removes the mirrored Markdown file if it exists

#### Scenario: No configured folder keeps SQLite-only behavior
- **WHEN** the user creates or updates a note and no notes folder is configured
- **THEN** the system persists the note in SQLite without writing a Markdown mirror file

### Requirement: Manual selected-note export remains available
The system SHALL preserve the existing selected-note export flow and SHALL continue using the configured notes folder as the default export destination.

#### Scenario: Export selected notes
- **WHEN** the user selects notes and clicks Save to Folder
- **THEN** the system exports the selected notes to Markdown files using the configured notes folder unless an override folder is provided
