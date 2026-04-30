## ADDED Requirements

### Requirement: Capture local memories
The system SHALL support saving durable memory entries for user preferences, project facts, recurring ideas, decisions, and profile-specific context.

#### Scenario: User saves memory manually
- **WHEN** the user marks a conversation detail as memory
- **THEN** the system stores a local memory entry with source conversation metadata

#### Scenario: Secretary proposes memory
- **WHEN** the secretary identifies a potentially useful durable fact
- **THEN** the system asks the user to confirm before storing it as memory

### Requirement: Retrieve relevant memories
The system SHALL include relevant active memory entries when assembling context for a secretary request.

#### Scenario: Message relates to saved memory
- **WHEN** the user sends a message related to existing active memories
- **THEN** the system includes relevant memories in the prompt context sent to the LLM

### Requirement: Manage memory lifecycle
The system SHALL allow the user to review, edit, pin, archive, and delete memory entries.

#### Scenario: User forgets a memory
- **WHEN** the user deletes a memory entry
- **THEN** the system removes that memory from future prompt context

#### Scenario: User archives a memory
- **WHEN** the user archives a memory entry
- **THEN** the system keeps it in history but excludes it from normal prompt context

#### Scenario: User pins a memory
- **WHEN** the user pins a memory entry
- **THEN** the system prioritizes that memory when assembling prompt context for the associated profile or domain

### Requirement: Scope memories
The system SHALL support memory scope by profile, domain, and global app context.

#### Scenario: Profile-specific memory exists
- **WHEN** a secretary request uses a profile with profile-specific memories
- **THEN** the system considers those memories before unrelated profile memories

### Requirement: Explain memory usage
The system SHALL show which memories were used for a response when memory context is included.

#### Scenario: Response uses memory
- **WHEN** the system includes memory entries in a secretary request
- **THEN** the Secretary module shows a list or count of memories used for that response

### Requirement: Keep memory local
The system SHALL store memories locally and SHALL only send selected relevant memory content to the configured LLM endpoint as part of a secretary request.

#### Scenario: Memory panel is open
- **WHEN** the user reviews memory settings
- **THEN** the system indicates that memories are local and user-controlled
