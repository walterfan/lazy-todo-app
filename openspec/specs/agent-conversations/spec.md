# agent-conversations Specification

## Purpose
TBD - created by archiving change refactor-secretary-to-agents. Update Purpose after archive.
## Requirements
### Requirement: Agent sessions
The system SHALL create local conversation sessions with a session type, title, selected Agent participants, timestamps, and persisted metadata.

#### Scenario: Single-Agent session is created
- **WHEN** the user starts a conversation with one enabled Agent
- **THEN** the system creates a single-Agent session with that Agent as a participant

#### Scenario: Multi-Agent session is created
- **WHEN** the user starts a conversation with multiple enabled Agents
- **THEN** the system creates a group session with all selected Agents as participants

### Requirement: Speaker-attributed messages
The system SHALL persist every user and Agent message with sender type, session ID, optional Agent ID, content, timestamps, and ordering metadata.

#### Scenario: Agent replies
- **WHEN** an Agent produces a reply
- **THEN** the system persists the message with the Agent ID and displays the Agent as the speaker

### Requirement: Single-Agent dispatch
The system SHALL dispatch a user message to the selected Agent using that Agent's prompt, config, RAG snippets, memory, selected app context, and recent session history.

#### Scenario: User sends message in single-Agent session
- **WHEN** the user sends a message in a single-Agent session
- **THEN** the system streams and persists one reply from the selected Agent

#### Scenario: User chats with a built-in Agent
- **WHEN** the user selects Personal Secretary or Confucius and sends a message
- **THEN** the system streams and persists one reply using the selected built-in Agent identity and prompt

### Requirement: Multi-Agent group dispatch
The system SHALL dispatch a user message to each selected enabled Agent in a group session and stream each Agent's reply with attribution.

#### Scenario: User sends message in group session
- **WHEN** the user sends a message in a group session with three enabled Agents
- **THEN** the system requests responses from all three Agents and displays each response under its Agent identity

#### Scenario: Agent does not support group chat
- **WHEN** a selected Agent has multi-Agent support disabled
- **THEN** the system prevents that Agent from being added to a group session or reports why it cannot participate

### Requirement: Shared conversation context
The system SHALL provide group-session Agents with the shared user message history and prior Agent messages within configured token limits.

#### Scenario: Second Agent responds after first Agent
- **WHEN** an Agent response is generated after another Agent has already replied in the same turn
- **THEN** the system may include the prior Agent reply in the later Agent's context when within configured limits

### Requirement: Streaming events
The system SHALL stream Agent responses through Tauri events keyed by session, turn, and Agent ID.

#### Scenario: Stream chunk arrives
- **WHEN** a streaming chunk arrives for an Agent response
- **THEN** the UI appends the chunk to the matching Agent message draft

#### Scenario: Stream fails for one Agent
- **WHEN** one Agent response stream fails in a group session
- **THEN** the system records the error for that Agent without discarding successful replies from other Agents

### Requirement: Transcript export
The system SHALL export Agent conversation transcripts with speaker names, Agent IDs, timestamps, and session metadata.

#### Scenario: User exports a group session
- **WHEN** the user exports a group session
- **THEN** the exported transcript preserves each Agent's speaker attribution and message order

