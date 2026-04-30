## ADDED Requirements

### Requirement: Standard Agent plugin directory
The system SHALL define each Agent plugin as a static directory with required files `manifest.json`, `system_prompt.md`, `config.json`, `avatar.png`, and `README.md`, plus optional `rag_knowledge.md`.

#### Scenario: Valid plugin directory is present
- **WHEN** the plugin engine scans a directory containing all required files with valid names
- **THEN** the system recognizes the directory as a candidate Agent plugin

#### Scenario: Required file is missing
- **WHEN** the plugin engine scans a directory missing any required plugin file
- **THEN** the system marks the plugin invalid and reports the missing file

### Requirement: Bundled initial plugin folders
The system SHALL include a bundled `plugins/` folder with `secretary/` and `confucius/` subfolders that follow the standard Agent plugin directory structure.

#### Scenario: Bundled plugins are scanned
- **WHEN** the plugin engine scans the bundled `plugins/` folder
- **THEN** it validates `secretary/` and `confucius/` using the same rules as external plugins

#### Scenario: Bundled plugin structure is invalid
- **WHEN** a bundled Agent subfolder is missing a required plugin file
- **THEN** the system reports the same validation diagnostic used for external plugins

### Requirement: Manifest schema validation
The system SHALL validate `manifest.json` for required metadata including plugin ID, name, version, author, description, tags, creation/update dates, minimum app version, RAG support flag, and multi-Agent support flag.

#### Scenario: Manifest is valid
- **WHEN** `manifest.json` contains all required fields with valid values
- **THEN** the system loads the manifest metadata for display and runtime use

#### Scenario: Manifest field is invalid
- **WHEN** `manifest.json` has a missing field, malformed field, unsupported app version, or duplicate plugin ID
- **THEN** the system rejects the plugin and displays a validation diagnostic

### Requirement: Plugin identifier and naming rules
The system SHALL require plugin IDs to be globally unique lowercase identifiers using letters, numbers, and underscores, with a maximum length of 32 characters.

#### Scenario: Plugin ID is well formed
- **WHEN** a plugin ID matches the allowed naming pattern and is unique in the local plugin set
- **THEN** the system accepts the plugin ID

#### Scenario: Plugin ID conflicts
- **WHEN** two installed plugins declare the same plugin ID
- **THEN** the system marks the later conflicting plugin invalid and reports the conflict

### Requirement: System prompt file
The system SHALL read `system_prompt.md` as the Agent persona, thinking style, speech style, boundaries, and multi-Agent interaction rules.

#### Scenario: Prompt is loaded
- **WHEN** a valid Agent plugin is activated
- **THEN** the system includes the plugin system prompt in that Agent's LLM prompt assembly

### Requirement: Runtime config schema
The system SHALL validate `config.json` for supported runtime parameters including temperature, top_p, rag_top_k, embedding_dim, response_style, and ban_topics.

#### Scenario: Config values are valid
- **WHEN** config values are within supported ranges and types
- **THEN** the system stores them as runtime defaults for that Agent

#### Scenario: Config values are unsafe
- **WHEN** config values are out of range, malformed, or unsupported
- **THEN** the system marks the plugin invalid and reports the invalid field

### Requirement: Plugin packaging
The system SHALL support Agent plugins packaged as a folder or ZIP archive named with plugin ID and version.

#### Scenario: ZIP package is installed
- **WHEN** the user installs a ZIP package containing exactly one valid plugin folder
- **THEN** the system extracts it into the local plugin directory and validates it

#### Scenario: ZIP package attempts unsafe paths
- **WHEN** a ZIP package contains absolute paths, parent-directory paths, or files outside the plugin folder
- **THEN** the system rejects the package and does not write unsafe files
