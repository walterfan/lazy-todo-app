# agent-package-format Specification

## Purpose
TBD - created by archiving change refactor-secretary-to-agents. Update Purpose after archive.
## Requirements
### Requirement: Standard Agent package directory
The system SHALL define each Agent package as a static directory with required files `manifest.json`, `system_prompt.md`, `config.json`, `avatar.png`, and `README.md`, plus optional `rag_knowledge.md`.

#### Scenario: Valid Agent directory is present
- **WHEN** the Agent engine scans a directory containing all required files with valid names
- **THEN** the system recognizes the directory as a candidate Agent package

#### Scenario: Required file is missing
- **WHEN** the Agent engine scans a directory missing any required Agent file
- **THEN** the system marks the Agent invalid and reports the missing file

### Requirement: Bundled initial Agent folders
The system SHALL include a bundled `agents/` folder with `secretary/` and `confucius/` subfolders that follow the standard Agent package directory structure.

#### Scenario: Bundled Agents are scanned
- **WHEN** the Agent engine scans the bundled `agents/` folder
- **THEN** it validates `secretary/` and `confucius/` using the same rules as external Agents

#### Scenario: Bundled Agent structure is invalid
- **WHEN** a bundled Agent subfolder is missing a required Agent file
- **THEN** the system reports the same validation diagnostic used for external Agents

### Requirement: Manifest schema validation
The system SHALL validate `manifest.json` for required metadata including agent ID, name, version, author, description, tags, creation/update dates, minimum app version, RAG support flag, and multi-Agent support flag.

#### Scenario: Manifest is valid
- **WHEN** `manifest.json` contains all required fields with valid values
- **THEN** the system loads the manifest metadata for display and runtime use

#### Scenario: Manifest field is invalid
- **WHEN** `manifest.json` has a missing field, malformed field, unsupported app version, or duplicate agent ID
- **THEN** the system rejects the agent and displays a validation diagnostic

### Requirement: Agent identifier and naming rules
The system SHALL require agent IDs to be globally unique lowercase identifiers using letters, numbers, and underscores, with a maximum length of 32 characters.

#### Scenario: Agent ID is well formed
- **WHEN** an agent ID matches the allowed naming pattern and is unique in the local Agent set
- **THEN** the system accepts the agent ID

#### Scenario: Agent ID conflicts
- **WHEN** two installed Agents declare the same agent ID
- **THEN** the system marks the later conflicting Agent invalid and reports the conflict

### Requirement: System prompt file
The system SHALL read `system_prompt.md` as the Agent persona, thinking style, speech style, boundaries, and multi-Agent interaction rules.

#### Scenario: Prompt is loaded
- **WHEN** a valid Agent package is activated
- **THEN** the system includes the Agent system prompt in that Agent's LLM prompt assembly

### Requirement: Runtime config schema
The system SHALL validate `config.json` for supported runtime parameters including temperature, top_p, rag_top_k, embedding_dim, response_style, and ban_topics.

#### Scenario: Config values are valid
- **WHEN** config values are within supported ranges and types
- **THEN** the system stores them as runtime defaults for that Agent

#### Scenario: Config values are unsafe
- **WHEN** config values are out of range, malformed, or unsupported
- **THEN** the system marks the Agent invalid and reports the invalid field

### Requirement: Agent packaging
The system SHALL support Agent packages packaged as a folder or ZIP archive named with agent ID and version.

#### Scenario: ZIP package is installed
- **WHEN** the user installs a ZIP package containing exactly one valid Agent folder
- **THEN** the system extracts it into the local Agent directory and validates it

#### Scenario: ZIP package attempts unsafe paths
- **WHEN** a ZIP package contains absolute paths, parent-directory paths, or files outside the Agent folder
- **THEN** the system rejects the package and does not write unsafe files

