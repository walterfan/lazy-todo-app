## ADDED Requirements

### Requirement: Agents navigation
The system SHALL expose an Agents module in the main app navigation replacing the user-facing Secretary module.

#### Scenario: User opens Agents
- **WHEN** the user selects the Agents navigation item
- **THEN** the system displays the Agents chat interface instead of the Secretary interface

### Requirement: Agent selection UI
The system SHALL allow users to select one enabled Agent for single chat or multiple enabled Agents for group chat.

#### Scenario: User selects one Agent
- **WHEN** the user selects one enabled Agent and starts a session
- **THEN** the chat UI enters single-Agent mode with that Agent identity visible

#### Scenario: User selects a built-in Agent
- **WHEN** the Agents chat window opens during the initial phase
- **THEN** the user can select Personal Secretary or Confucius and start chatting with the selected Agent

#### Scenario: User selects multiple Agents
- **WHEN** the user selects multiple enabled Agents and starts a session
- **THEN** the chat UI enters group mode with all participant identities visible

### Requirement: Virtual human presentation
The system SHALL display each Agent as a virtual human with name, avatar, description, tags, and README details.

#### Scenario: Agent card is displayed
- **WHEN** a valid Agent is listed in the UI
- **THEN** the system displays the Agent avatar, name, description, version, author, and tags

### Requirement: Plugin management UI
The system SHALL provide Settings or management UI for plugin directory configuration, refresh, install, uninstall, enable, disable, and validation diagnostics.

#### Scenario: User refreshes plugins
- **WHEN** the user clicks refresh in plugin management
- **THEN** the system rescans the plugin directory and updates the plugin list

#### Scenario: User disables plugin
- **WHEN** the user disables an enabled plugin
- **THEN** the UI marks it disabled and prevents it from being selected for new sessions

### Requirement: Local data controls
The system SHALL show local data status for conversations, plugin data, and RAG indexes and provide safe cleanup controls.

#### Scenario: User views plugin data
- **WHEN** the user opens an Agent plugin detail view
- **THEN** the system shows local conversation count, RAG index status, and validation status for that plugin

### Requirement: Secretary migration visibility
The system SHALL show whether previous Secretary data has been migrated into Agents.

#### Scenario: Migration succeeds
- **WHEN** Secretary data is migrated into a default personal Agent
- **THEN** the UI shows the personal Agent as available and does not present obsolete Secretary controls
