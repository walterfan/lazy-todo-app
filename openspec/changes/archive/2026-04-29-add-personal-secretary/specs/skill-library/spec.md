## ADDED Requirements

### Requirement: Configure skill folder
The system SHALL allow the user to choose and save a local folder used as the secretary skill library.

#### Scenario: User chooses a skill folder
- **WHEN** the user selects a local folder for skills
- **THEN** the system saves that folder path as the current skill library source

### Requirement: Load skills from folder
The system SHALL scan the configured skill folder for supported text skill files and make loaded skills available for profile selection.

#### Scenario: Supported skill files exist
- **WHEN** the user refreshes the skill library and the configured folder contains supported skill files
- **THEN** the system lists those skills with name, source path, and summary information

#### Scenario: Skill folder is empty
- **WHEN** the user refreshes the skill library and no supported skill files are found
- **THEN** the system displays an empty skill list without failing the Secretary module

### Requirement: Report skipped skill files
The system SHALL report unsupported, unreadable, or oversized skill files without blocking valid skills from loading.

#### Scenario: Mixed valid and invalid files
- **WHEN** the configured folder contains both valid skill files and files that cannot be loaded
- **THEN** the system loads valid skills and reports which files were skipped

### Requirement: Reload skill changes
The system SHALL allow the user to refresh the skill library so changed files are reflected in the selectable skill list.

#### Scenario: Skill file is updated
- **WHEN** a skill file changes on disk and the user refreshes the skill library
- **THEN** the system updates the loaded skill metadata and content used by secretary profiles
