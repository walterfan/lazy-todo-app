## ADDED Requirements

### Requirement: External CLI tool registration
The system SHALL allow users to register approved external CLI commands as Agent tools with explicit metadata, argument schemas, execution policy, and permission settings.

#### Scenario: User registers CLI tool
- **WHEN** the user registers an external CLI tool such as `helper` or `skill`
- **THEN** the system stores its display name, executable path or command name, allowed subcommands, argument schema, working directory policy, environment policy, timeout, output limits, and enabled status

#### Scenario: CLI executable is unavailable
- **WHEN** a registered CLI executable cannot be found or executed
- **THEN** the system marks the tool unavailable and does not expose it to Agent conversations

### Requirement: Structured CLI arguments
The system SHALL execute external CLI tools only from registered schemas and validated arguments.

#### Scenario: Agent calls registered CLI
- **WHEN** an Agent requests an external CLI tool with arguments matching the registered schema
- **THEN** the system builds the process invocation from structured arguments without shell interpolation

#### Scenario: Agent supplies invalid argument
- **WHEN** an Agent requests an external CLI tool with unknown, malformed, or disallowed arguments
- **THEN** the system rejects the call and records a validation error

### Requirement: CLI permission controls
The system SHALL let users enable or disable external CLI tools globally, per Agent, and per session.

#### Scenario: CLI tool disabled for session
- **WHEN** an external CLI tool is disabled for the current session
- **THEN** the system does not expose that tool schema to the Agent

#### Scenario: CLI requires confirmation
- **WHEN** a registered CLI tool is marked as write, destructive, networked, or sensitive
- **THEN** the system creates a proposed CLI action and executes it only after user confirmation

### Requirement: CLI execution sandbox policy
The system SHALL constrain external CLI execution using registered working directories, environment allowlists, timeouts, and output limits.

#### Scenario: CLI runs with allowed environment
- **WHEN** the system executes a confirmed or read-only CLI tool call
- **THEN** it passes only configured environment variables, applies the configured working directory, enforces timeout, and captures bounded stdout and stderr

#### Scenario: CLI exceeds timeout
- **WHEN** a CLI process exceeds its configured timeout
- **THEN** the system terminates the process, records a timeout result, and returns a bounded error to the Agent conversation

### Requirement: CLI output handling
The system SHALL return external CLI output to Agents as bounded tool results with exit status, stdout, stderr, and truncation metadata.

#### Scenario: CLI succeeds
- **WHEN** a CLI process exits successfully
- **THEN** the system records the exit code, bounded stdout, bounded stderr, duration, and whether output was truncated

#### Scenario: CLI fails
- **WHEN** a CLI process exits with a non-zero status
- **THEN** the system records the failure and returns a safe summarized error result to the Agent

### Requirement: CLI audit logging
The system SHALL persist audit records for every external CLI tool call.

#### Scenario: CLI call is audited
- **WHEN** an Agent requests an external CLI tool
- **THEN** the system records session ID, Agent ID, tool ID, command metadata, validated arguments, confirmation status, exit code, bounded output, timestamps, and duration

#### Scenario: Sensitive values are logged
- **WHEN** CLI arguments or environment variables contain registered sensitive values
- **THEN** the audit record masks those values before storage or display

### Requirement: External CLI management UI
The system SHALL provide Settings UI to manage registered external CLI tools.

#### Scenario: User manages external tools
- **WHEN** the user opens Agent tool settings
- **THEN** the UI allows listing, adding, editing, testing, enabling, disabling, and deleting external CLI tool registrations

#### Scenario: User tests CLI registration
- **WHEN** the user tests a CLI registration
- **THEN** the system validates executable availability, argument schema, working directory, environment policy, and timeout behavior without exposing the tool until the registration is valid
