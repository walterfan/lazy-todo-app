## ADDED Requirements

### Requirement: Configure secretary persona
The system SHALL allow the user to configure a secretary persona with name, voice, values, interaction style, and boundaries.

#### Scenario: User creates persona
- **WHEN** the user enters persona details and saves them
- **THEN** the system persists the persona for use in secretary profiles

### Requirement: Apply persona to profile prompt
The system SHALL include the selected persona instructions when assembling the system prompt for a secretary profile.

#### Scenario: Conversation starts with persona
- **WHEN** the user starts a conversation with a profile that has a selected persona
- **THEN** the secretary behavior is instructed by that persona in addition to role, domain, and skills

### Requirement: Preserve secretary continuity
The system SHALL keep the secretary persona consistent across messages and saved conversations unless the user changes the persona.

#### Scenario: User continues conversation
- **WHEN** the user sends multiple messages in the same conversation
- **THEN** the secretary maintains the selected persona voice and interaction style

### Requirement: Show active persona context
The system SHALL display the active persona summary so the user can understand which secretary identity is currently speaking.

#### Scenario: Secretary panel is open
- **WHEN** a persona is active
- **THEN** the Secretary module shows the persona name and a concise summary of its voice or style

### Requirement: Support default personal secretary persona
The system SHALL provide a default personal secretary persona when the user has not created a custom persona.

#### Scenario: No custom persona exists
- **WHEN** the user opens the Secretary module for the first time
- **THEN** the system offers a default persona suitable for thoughtful, warm, practical, discreet personal-secretary support
