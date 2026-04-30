# agent-plugin-runtime Specification

## Purpose
TBD - created by archiving change refactor-secretary-to-agents. Update Purpose after archive.
## Requirements
### Requirement: Bundled initial Agents
The system SHALL provide two initial Agents named Personal Secretary and Confucius by scanning bundled standard plugin folders.

#### Scenario: Agents module loads for the first time
- **WHEN** the user opens the Agents module with no external plugins installed
- **THEN** the system scans the bundled `plugins/` folder and lists Personal Secretary and Confucius as available Agents

#### Scenario: Confucius is selected
- **WHEN** the user selects Confucius
- **THEN** the system uses a Confucian persona prompt centered on `仁`, `礼`, moderation, self-cultivation, education, and classical Chinese wisdom

### Requirement: Configurable plugin directory
The system SHALL allow users to configure a local Agent plugin directory that the plugin engine scans for installed Agent plugins.

#### Scenario: Plugin directory is configured
- **WHEN** the user saves a plugin directory path
- **THEN** the system persists the path locally and uses it for subsequent plugin scans

#### Scenario: Plugin directory is unavailable
- **WHEN** the configured plugin directory cannot be read
- **THEN** the system reports the access error without deleting existing plugin metadata

### Requirement: Plugin discovery and refresh
The system SHALL discover plugins on app startup and on explicit refresh without requiring an app restart.

#### Scenario: New plugin folder is added
- **WHEN** the user adds a valid plugin folder to the configured plugin directory and refreshes plugins
- **THEN** the system adds the plugin to the available Agent list

#### Scenario: Plugin folder changes
- **WHEN** a plugin file hash or version changes during refresh
- **THEN** the system reloads metadata and marks dependent RAG indexes for rebuild when needed

### Requirement: Plugin lifecycle state
The system SHALL track plugin lifecycle states including discovered, valid, invalid, loaded, enabled, disabled, active in session, and uninstalled.

#### Scenario: User disables plugin
- **WHEN** the user disables an installed plugin
- **THEN** the system keeps plugin metadata and data but excludes the Agent from new conversation selection

#### Scenario: User uninstalls plugin
- **WHEN** the user uninstalls a plugin
- **THEN** the system removes the plugin files from the app-managed plugin directory and marks the plugin uninstalled

### Requirement: Plugin isolation
The system SHALL isolate each plugin's configuration, prompt, avatar, README, RAG knowledge, validation state, and local private data by plugin ID.

#### Scenario: Two plugins have different knowledge files
- **WHEN** two enabled plugins both include `rag_knowledge.md`
- **THEN** the system indexes and retrieves each plugin's knowledge independently

### Requirement: Validation diagnostics
The system SHALL keep validation errors visible for invalid plugins.

#### Scenario: Invalid plugin is scanned
- **WHEN** plugin validation fails
- **THEN** the system lists the plugin as invalid with actionable diagnostics

### Requirement: Built-in personal Agent migration
The system SHALL provide a default personal Agent representing migrated Secretary behavior when previous Secretary data exists.

#### Scenario: Secretary data exists
- **WHEN** the Agents module starts for the first time and Secretary persona/profile data exists
- **THEN** the system creates or maps a default personal Agent using the existing Secretary configuration

