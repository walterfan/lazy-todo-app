## ADDED Requirements

### Requirement: Read Todo context
The system SHALL allow the secretary to read selected Todo items as context for answering questions, making suggestions, asking questions, and proposing follow-ups.

#### Scenario: User includes active todos
- **WHEN** the user enables Todo context for a secretary request
- **THEN** the system includes relevant active Todo items with title, description, priority, deadline, and completion status

#### Scenario: User asks about priorities
- **WHEN** the user asks the secretary what to focus on and Todo context is enabled
- **THEN** the secretary can reason over Todo priority, deadline, and completion status

### Requirement: Read milestone context
The system SHALL allow the secretary to read Pomodoro milestones as context for planning, reminders, and progress discussion.

#### Scenario: User includes milestones
- **WHEN** the user enables milestone context for a secretary request
- **THEN** the system includes configured milestone name, deadline, and status

#### Scenario: Milestone is near deadline
- **WHEN** a milestone is active and near its deadline
- **THEN** the secretary can suggest focus, ask clarifying questions, or propose a reminder

### Requirement: Read Sticky Note context
The system SHALL allow the secretary to read selected Sticky Notes as context for discussion and suggestions.

#### Scenario: User includes notes
- **WHEN** the user enables note context for a secretary request
- **THEN** the system includes relevant note title, content, color, created time, and updated time

#### Scenario: User asks about prior notes
- **WHEN** the user asks the secretary to summarize or use notes and note context is enabled
- **THEN** the secretary can reason over selected note content

### Requirement: Control context inclusion
The system SHALL allow the user to control whether Todos, milestones, and notes are included in secretary requests.

#### Scenario: User disables notes
- **WHEN** note context is disabled
- **THEN** the system does not include Sticky Note content in the LLM request

#### Scenario: User selects specific context items
- **WHEN** the user selects specific Todos, milestones, or notes for a secretary request
- **THEN** the system includes only the selected items from those sources

### Requirement: Show context used
The system SHALL show which Todo, milestone, and note context was used for a secretary response.

#### Scenario: Response uses app context
- **WHEN** the system includes Todo, milestone, or note context in a secretary request
- **THEN** the Secretary module shows a list or count of app context items used for that response

### Requirement: Keep app context read-only by default
The system SHALL treat Todo, milestone, and note context as read-only during secretary requests unless the user explicitly confirms a supported app update.

#### Scenario: Secretary suggests a Todo update
- **WHEN** the secretary suggests changing a Todo or milestone
- **THEN** the system presents the suggestion without mutating the underlying app data automatically

### Requirement: Propose Sticky Note edits
The system SHALL allow the secretary to propose edits to selected Sticky Notes based on user instructions and note context.

#### Scenario: Secretary proposes note change
- **WHEN** the user asks the secretary to improve, summarize, reorganize, or update a selected Sticky Note
- **THEN** the system presents a proposed note edit containing the target note, changed fields, and before/after preview

### Requirement: Apply confirmed Sticky Note edits
The system SHALL apply secretary-proposed Sticky Note edits only after the user confirms the change.

#### Scenario: User confirms note edit
- **WHEN** the secretary proposes a Sticky Note edit and the user confirms it
- **THEN** the system updates the Sticky Note through the app's note persistence flow

#### Scenario: User rejects note edit
- **WHEN** the secretary proposes a Sticky Note edit and the user rejects it
- **THEN** the system leaves the Sticky Note unchanged

### Requirement: Preserve note edit audit metadata
The system SHALL record secretary note edit metadata in the conversation transcript when a proposed edit is accepted or rejected.

#### Scenario: Confirmed edit is recorded
- **WHEN** the user confirms a secretary-proposed Sticky Note edit
- **THEN** the conversation records the target note identifier, changed fields, confirmation time, and resulting note metadata
