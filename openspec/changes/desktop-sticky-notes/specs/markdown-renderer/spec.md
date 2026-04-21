## ADDED Requirements

### Requirement: Render Markdown content to formatted HTML
The system SHALL render Markdown text into formatted, visually styled content within sticky note cards. The rendering SHALL support standard Markdown syntax including headings, paragraphs, bold, italic, strikethrough, ordered and unordered lists, links, inline code, and fenced code blocks.

#### Scenario: Render headings and paragraphs
- **WHEN** a note contains `# Title\n\nSome paragraph text`
- **THEN** the renderer displays "Title" as a heading element and "Some paragraph text" as a paragraph

#### Scenario: Render code blocks
- **WHEN** a note contains a fenced code block with triple backticks
- **THEN** the renderer displays the code in a monospaced, visually distinct block

#### Scenario: Render lists
- **WHEN** a note contains `- item 1\n- item 2\n- item 3`
- **THEN** the renderer displays an unordered bullet list with three items

#### Scenario: Render links
- **WHEN** a note contains `[example](https://example.com)`
- **THEN** the renderer displays a clickable link labeled "example"

### Requirement: Support GitHub Flavored Markdown extensions
The system SHALL support GFM extensions including tables, task lists (checkboxes), strikethrough, and autolinked URLs.

#### Scenario: Render a GFM table
- **WHEN** a note contains a pipe-delimited table in GFM format
- **THEN** the renderer displays a styled HTML table with headers and rows

#### Scenario: Render task list checkboxes
- **WHEN** a note contains `- [x] Done\n- [ ] Pending`
- **THEN** the renderer displays checkboxes (checked and unchecked) next to the items

### Requirement: Toggle between edit and preview modes
The system SHALL provide a toggle control on each note to switch between raw Markdown editing (textarea) and rendered preview. The default view for an existing note SHALL be preview mode.

#### Scenario: Switch from preview to edit
- **WHEN** user clicks the "Edit" button on a note in preview mode
- **THEN** the rendered Markdown is replaced by a textarea containing the raw Markdown source

#### Scenario: Switch from edit to preview
- **WHEN** user clicks "Preview" while in edit mode
- **THEN** the textarea is replaced by the rendered Markdown output reflecting the current content

### Requirement: Safe rendering without XSS
The system SHALL render Markdown using a React-native renderer (no `dangerouslySetInnerHTML`) to prevent cross-site scripting. Raw HTML tags in Markdown input SHALL NOT be rendered as executable HTML.

#### Scenario: HTML tags in Markdown are neutralized
- **WHEN** a note contains `<script>alert('xss')</script>`
- **THEN** the renderer does not execute the script and either escapes or strips the HTML tag
