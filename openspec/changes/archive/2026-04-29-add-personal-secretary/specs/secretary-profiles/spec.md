## ADDED Requirements

### Requirement: Select secretary role, domain, and persona
The system SHALL allow the user to select a secretary role, enter a domain, and choose a secretary persona before starting a conversation.

#### Scenario: User prepares a profile
- **WHEN** the user chooses a built-in role, enters a domain, and selects a persona
- **THEN** the system stores the selected role, domain, and persona as the active secretary profile context

### Requirement: Provide built-in role behaviors
The system SHALL provide built-in roles for domain question answering, domain question asking, idea criticism, and idea generation.

#### Scenario: User selects domain question answering
- **WHEN** the user selects the domain question answering role
- **THEN** the secretary behavior is instructed to answer questions within the selected domain

#### Scenario: User selects domain question asking
- **WHEN** the user selects the domain question asking role
- **THEN** the secretary behavior is instructed to ask the user questions within the selected domain

#### Scenario: User selects idea criticism
- **WHEN** the user selects the idea criticism role
- **THEN** the secretary behavior is instructed to critique ideas within the selected domain

#### Scenario: User selects idea generation
- **WHEN** the user selects the idea generation role
- **THEN** the secretary behavior is instructed to raise new ideas within the selected domain

### Requirement: Attach skills to a profile
The system SHALL allow the user to select one or more loaded skills for the active secretary profile.

#### Scenario: User selects skills
- **WHEN** the user selects loaded skills for the active profile
- **THEN** the selected skills are included in the active profile context for future secretary messages

### Requirement: Persist secretary profiles
The system SHALL persist secretary profile metadata so that role, domain, persona, and selected skill references survive app restarts.

#### Scenario: App restarts after profile creation
- **WHEN** the user creates a secretary profile and restarts the app
- **THEN** the profile is available for selection with its role, domain, persona, and skill references intact
