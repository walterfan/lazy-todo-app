## ADDED Requirements

### Requirement: Provide secretary suggestions
The system SHALL allow the secretary to suggest next steps, risks, decisions, and useful actions based on the active conversation, memory, domain, and skills.

#### Scenario: User asks for guidance
- **WHEN** the user asks what to do next
- **THEN** the secretary responds with relevant suggestions grounded in the active context

### Requirement: Ask clarifying questions
The system SHALL allow the secretary to ask clarifying questions when user intent, missing information, or decision criteria are unclear.

#### Scenario: User gives an ambiguous request
- **WHEN** the user asks for help but important details are missing
- **THEN** the secretary asks concise clarifying questions before giving a final recommendation

### Requirement: Identify follow-ups
The system SHALL identify possible follow-up items from conversations without automatically committing them.

#### Scenario: Conversation contains a follow-up
- **WHEN** the user discusses a future action, deadline, or pending decision
- **THEN** the secretary can present it as a proposed follow-up item for user review

### Requirement: Create user-confirmed reminders
The system SHALL create reminders only after the user confirms the reminder content and due time.

#### Scenario: User confirms proposed reminder
- **WHEN** the secretary proposes a reminder and the user confirms it
- **THEN** the system saves the reminder with title, optional notes, due time, source conversation, and status

#### Scenario: User rejects proposed reminder
- **WHEN** the secretary proposes a reminder and the user rejects it
- **THEN** the system does not save the reminder

### Requirement: Notify due reminders
The system SHALL surface due reminders in the Secretary module and SHOULD use available desktop notification support when notifications are enabled.

#### Scenario: Reminder becomes due
- **WHEN** a saved reminder reaches its due time
- **THEN** the system marks it as due and presents it to the user

### Requirement: Manage reminders
The system SHALL allow the user to list, complete, snooze, edit, and delete secretary reminders.

#### Scenario: User completes reminder
- **WHEN** the user marks a reminder complete
- **THEN** the system removes it from active reminder prompts while retaining its history
