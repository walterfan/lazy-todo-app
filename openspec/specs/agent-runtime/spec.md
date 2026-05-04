# agent-runtime Specification

## Purpose
TBD - created by archiving change refactor-secretary-to-agents. Update Purpose after archive.
## Requirements
### Requirement: Bundled initial Agents
The system SHALL provide two initial Agents named Personal Secretary and Confucius by scanning bundled standard Agent folders.

#### Scenario: Agents module loads for the first time
- **WHEN** the user opens the Agents module with no external Agents installed
- **THEN** the system scans the bundled `agents/` folder and lists Personal Secretary and Confucius as available Agents

#### Scenario: Confucius is selected
- **WHEN** the user selects Confucius
- **THEN** the system uses a Confucian persona prompt centered on `仁`, `礼`, moderation, self-cultivation, education, and classical Chinese wisdom

### Requirement: Configurable Agent directory
The system SHALL allow users to configure a local Agent package directory that the Agent engine scans for installed Agent packages.

#### Scenario: Agent directory is configured
- **WHEN** the user saves an Agent directory path
- **THEN** the system persists the path locally and uses it for subsequent Agent scans

#### Scenario: Agent directory is unavailable
- **WHEN** the configured agent directory cannot be read
- **THEN** the system reports the access error without deleting existing agent metadata

### Requirement: Agent discovery and refresh
The system SHALL discover Agents on app startup and on explicit refresh without requiring an app restart.

#### Scenario: New agent folder is added
- **WHEN** the user adds a valid Agent folder to the configured Agent directory and refreshes Agents
- **THEN** the system adds the Agent to the available Agent list

#### Scenario: Agent folder changes
- **WHEN** an Agent file hash or version changes during refresh
- **THEN** the system reloads metadata and marks dependent RAG indexes for rebuild when needed

### Requirement: Agent lifecycle state
The system SHALL track Agent lifecycle states including discovered, valid, invalid, loaded, enabled, disabled, active in session, and uninstalled.

#### Scenario: User disables agent
- **WHEN** the user disables an installed Agent
- **THEN** the system keeps Agent metadata and data but excludes the Agent from new conversation selection

#### Scenario: User uninstalls agent
- **WHEN** the user uninstalls an Agent
- **THEN** the system removes the Agent files from the app-managed Agent directory and marks the Agent uninstalled

### Requirement: Agent isolation
The system SHALL isolate each Agent's configuration, prompt, avatar, README, RAG knowledge, validation state, and local private data by agent ID.

#### Scenario: Two Agents have different knowledge files
- **WHEN** two enabled Agents both include `rag_knowledge.md`
- **THEN** the system indexes and retrieves each Agent's knowledge independently

### Requirement: Validation diagnostics
The system SHALL keep validation errors visible for invalid Agents.

#### Scenario: Invalid Agent is scanned
- **WHEN** Agent validation fails
- **THEN** the system lists the Agent as invalid with actionable diagnostics

### Requirement: Built-in personal Agent migration
The system SHALL provide a default personal Agent representing migrated Secretary behavior when previous Secretary data exists.

#### Scenario: Secretary data exists
- **WHEN** the Agents module starts for the first time and Secretary persona/profile data exists
- **THEN** the system creates or maps a default personal Agent using the existing Secretary configuration

