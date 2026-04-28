## ADDED Requirements

### Requirement: Start conversation with selected profile
The system SHALL start a secretary conversation using the active persona, role, domain, selected skills, and relevant memories as the secretary context.

#### Scenario: User starts a new conversation
- **WHEN** the user starts a conversation after selecting a persona, role, domain, and optional skills
- **THEN** the system creates a conversation using that profile context

### Requirement: Send user messages to LLM
The system SHALL send user messages to the configured OpenAI-compatible chat completion endpoint and display secretary responses.

#### Scenario: LLM request succeeds
- **WHEN** the user sends a message and the LLM endpoint returns a valid response
- **THEN** the system appends both the user message and secretary response to the conversation transcript

#### Scenario: LLM request uses memory
- **WHEN** the user sends a message and relevant active memories exist
- **THEN** the system includes selected memory context in the LLM request and records which memories were used

#### Scenario: LLM request fails
- **WHEN** the user sends a message and the LLM endpoint returns an error or cannot be reached
- **THEN** the system preserves the user message draft or transcript state and displays an actionable error

### Requirement: Stream secretary replies
The system SHALL stream LLM reply chunks into the Secretary transcript before the final response is persisted.

#### Scenario: Stream chunk arrives
- **WHEN** the LLM endpoint returns a streaming content chunk for the active secretary request
- **THEN** the system appends the chunk to a transient assistant message in the transcript

#### Scenario: Stream completes
- **WHEN** the LLM endpoint finishes a streaming response
- **THEN** the system replaces the transient pending messages with the persisted conversation containing the complete assistant response

#### Scenario: Stream fails
- **WHEN** a streaming request fails before completion
- **THEN** the system clears the in-flight streaming state, preserves the existing saved transcript, and displays the stream error

### Requirement: Save conversations to configured folder
The system SHALL allow the user to configure a conversation folder and save chat transcripts into that folder.

#### Scenario: User saves a conversation
- **WHEN** the user saves a conversation and the configured conversation folder is writable
- **THEN** the system writes the conversation transcript to that folder and records its metadata

#### Scenario: Conversation folder is unavailable
- **WHEN** the user saves a conversation and the configured folder is missing or not writable
- **THEN** the system does not discard the transcript and reports that the save failed

### Requirement: Preserve conversation metadata
The system SHALL persist conversation metadata including title, created time, updated time, profile reference, and transcript file path.

#### Scenario: Conversation is saved
- **WHEN** the system successfully saves a conversation transcript
- **THEN** the saved conversation appears in the conversation list with its metadata

### Requirement: Load saved conversations
The system SHALL allow the user to open a saved conversation from the conversation list.

#### Scenario: User opens saved conversation
- **WHEN** the user selects a saved conversation
- **THEN** the system loads and displays the saved transcript from its recorded file path
