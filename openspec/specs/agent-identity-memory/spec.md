# agent-identity-memory Specification

## Purpose
TBD - created by archiving change refactor-secretary-to-agents. Update Purpose after archive.
## Requirements
### Requirement: User identity profile
The system SHALL maintain a local user identity profile that Agents can use to understand who they are talking to.

#### Scenario: Agent conversation includes user identity
- **WHEN** an Agent reply is prepared
- **THEN** the prompt context includes the user's enabled identity profile fields within configured limits

#### Scenario: User edits identity profile
- **WHEN** the user updates their name, preferred language, communication style, roles, goals, boundaries, or important facts
- **THEN** future Agent conversations use the updated profile

### Requirement: Durable app memory
The system SHALL maintain local durable memories about user preferences, facts, goals, decisions, and relationship context separately from Agent personas.

#### Scenario: Memory is stored
- **WHEN** the user confirms or manually creates a memory
- **THEN** the system stores the memory with source metadata, timestamps, scope, and visibility status

#### Scenario: Memory is deleted
- **WHEN** the user deletes a memory
- **THEN** the system removes it from future Agent prompt context and relevant memory retrieval results

### Requirement: Memory proposal confirmation
The system SHALL require user confirmation before an Agent stores or updates durable memory.

#### Scenario: Agent proposes memory
- **WHEN** an Agent identifies a fact, preference, or goal worth remembering
- **THEN** the system shows a proposed memory with source context and stores it only after user confirmation

#### Scenario: User rejects proposed memory
- **WHEN** the user rejects a proposed memory
- **THEN** the system does not store that memory as durable app memory

### Requirement: Previous conversation recall
The system SHALL let Agents recall relevant previous conversations using persisted messages and conversation summaries.

#### Scenario: Agent recalls prior conversation
- **WHEN** a user starts or continues a conversation with an Agent
- **THEN** the system may include relevant previous conversation summaries and recent messages within configured limits

#### Scenario: Prior conversation is not relevant
- **WHEN** no previous conversation is relevant to the current Agent, topic, or user request
- **THEN** the system does not attach unrelated history to the prompt

### Requirement: Conversation summaries
The system SHALL create and update local summaries for Agent sessions so long conversations can be recalled without replaying full transcripts.

#### Scenario: Session summary is updated
- **WHEN** a session receives new user or Agent messages
- **THEN** the system updates or schedules an update to the local conversation summary

#### Scenario: Summary source is visible
- **WHEN** a summary is used in a future prompt
- **THEN** the UI can show which session and Agent produced the summary

### Requirement: Prompt memory assembly
The system SHALL assemble Agent prompts from Agent persona, user identity, durable memories, relevant previous conversation summaries, selected app context, RAG snippets, tool results, and current session history with explicit boundaries.

#### Scenario: Prompt is assembled
- **WHEN** the system prepares an Agent request
- **THEN** user identity, app memory, and previous conversation summaries are labeled as app-owned context and kept distinct from the Agent system prompt

#### Scenario: Prompt limits are reached
- **WHEN** memory and conversation context exceed configured token or character limits
- **THEN** the system prioritizes pinned memories, recent relevant summaries, selected context, and current conversation history

### Requirement: Memory transparency controls
The system SHALL show users what identity, memories, and previous conversations were used for an Agent response.

#### Scenario: User inspects Agent context
- **WHEN** the user opens the context details for an Agent reply
- **THEN** the UI lists the identity profile fields, memories, and previous conversation summaries that were attached

#### Scenario: User disables memory for a session
- **WHEN** the user disables memory recall for the current Agent session
- **THEN** the system does not attach durable memories or previous conversation summaries to subsequent Agent requests in that session

### Requirement: Local privacy and isolation
The system SHALL keep identity profile, durable memory, and conversation summaries in local storage and respect per-Agent and global visibility settings.

#### Scenario: Agent-specific memory is scoped
- **WHEN** a memory is scoped to one Agent
- **THEN** other Agents do not receive that memory unless the user changes the scope

#### Scenario: User clears memory
- **WHEN** the user clears app memory or conversation summaries
- **THEN** the system removes the selected local records and excludes them from future prompt context

