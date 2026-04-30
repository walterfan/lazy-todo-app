# agent-built-in-tools Specification

## Purpose
TBD - created by archiving change refactor-secretary-to-agents. Update Purpose after archive.
## Requirements
### Requirement: Built-in tool registry
The system SHALL provide an app-owned built-in tool registry for Agents, with explicit tool names, JSON argument schemas, result schemas, permissions, and safety class.

#### Scenario: Agent asks for available tools
- **WHEN** an Agent conversation is prepared
- **THEN** the system exposes only enabled built-in tools and their schemas to the LLM

#### Scenario: Unknown tool is requested
- **WHEN** an Agent requests a tool that is not registered or enabled
- **THEN** the system rejects the tool call and records an unsupported-tool result

### Requirement: Notes tools
The system SHALL support `read_note` and `write_note` tools for Sticky Note access through app-owned persistence paths.

#### Scenario: Agent reads a note
- **WHEN** an Agent calls `read_note` with a valid note ID within the selected context scope
- **THEN** the system returns the note title, content, color, and timestamps

#### Scenario: Agent writes a note
- **WHEN** an Agent calls `write_note` with note changes
- **THEN** the system creates a proposed note write action and applies it only after user confirmation

### Requirement: Todo tools
The system SHALL support `read_todo_list`, `add_todo_item`, and `change_todo_item` tools through the existing Todo persistence path.

#### Scenario: Agent reads todo list
- **WHEN** an Agent calls `read_todo_list`
- **THEN** the system returns todo items allowed by the current session context settings

#### Scenario: Agent adds todo item
- **WHEN** an Agent calls `add_todo_item` with title, optional description, priority, and optional deadline
- **THEN** the system creates a proposed todo add action and applies it only after user confirmation

#### Scenario: Agent changes todo item
- **WHEN** an Agent calls `change_todo_item` with a valid todo ID and changed fields
- **THEN** the system creates a proposed todo change action and applies it only after user confirmation

### Requirement: Milestone tools
The system SHALL support `read_milestones` and `change_milestone` tools through the existing Pomodoro milestone persistence path.

#### Scenario: Agent reads milestones
- **WHEN** an Agent calls `read_milestones`
- **THEN** the system returns configured milestones allowed by the current session context settings

#### Scenario: Agent changes milestone
- **WHEN** an Agent calls `change_milestone` with a valid milestone index and changed fields
- **THEN** the system creates a proposed milestone change action and applies it only after user confirmation

### Requirement: File tools
The system SHALL support `read_file` and `write_file` tools only within configured safe file roots.

#### Scenario: Agent reads allowed file
- **WHEN** an Agent calls `read_file` for a path under an allowed file root
- **THEN** the system returns bounded file content and metadata

#### Scenario: Agent reads disallowed file
- **WHEN** an Agent calls `read_file` for a path outside allowed file roots
- **THEN** the system rejects the tool call and reports a scope violation

#### Scenario: Agent writes file
- **WHEN** an Agent calls `write_file` for a path under an allowed file root
- **THEN** the system creates a proposed file write action and applies it only after user confirmation

### Requirement: Tool call lifecycle
The system SHALL track tool calls from request through validation, proposed action, confirmation, execution, result delivery, and audit logging.

#### Scenario: Read tool completes
- **WHEN** an Agent read tool call validates and executes successfully
- **THEN** the system appends the tool result to the Agent turn context and records the call in the session log

#### Scenario: Write tool is confirmed
- **WHEN** the user confirms a proposed write/change tool call
- **THEN** the system executes the mutation through the existing app persistence path and returns the result to the conversation

#### Scenario: Write tool is rejected
- **WHEN** the user rejects a proposed write/change tool call
- **THEN** the system records the rejection and does not mutate user data

### Requirement: Tool permissions
The system SHALL allow users to enable or disable tool categories for an Agent session.

#### Scenario: Tool category disabled
- **WHEN** note tools are disabled for the current session
- **THEN** the system does not expose `read_note` or `write_note` to the Agent

#### Scenario: Write tools disabled
- **WHEN** write/change tools are disabled for the current session
- **THEN** the system exposes read tools only and rejects write/change tool calls

