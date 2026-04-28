## ADDED Requirements

### Requirement: Resolve effective LLM configuration
The system SHALL resolve `LLM_BASE_URL`, `LLM_MODEL`, and `LLM_API_KEY` from process environment variables before falling back to saved application configuration.

#### Scenario: Environment variables are present
- **WHEN** the user opens the Secretary module and the process environment contains `LLM_BASE_URL`, `LLM_MODEL`, or `LLM_API_KEY`
- **THEN** the system uses the environment value for each present setting instead of the saved value

#### Scenario: Environment variables are absent
- **WHEN** the user opens the Secretary module and one or more LLM environment variables are not present
- **THEN** the system uses the saved configuration value for each missing setting

### Requirement: Save fallback LLM configuration
The system SHALL allow the user to save fallback LLM base URL and model values in application configuration.

#### Scenario: User saves provider settings
- **WHEN** the user enters an LLM base URL and model in the Secretary configuration UI and saves
- **THEN** the system persists those values for later app sessions

### Requirement: Protect API key display
The system SHALL mask API key values in the UI and SHALL NOT expose API key values through normal configuration read responses.

#### Scenario: Configuration is displayed
- **WHEN** the Secretary configuration UI renders current settings
- **THEN** any available API key is shown only as a masked or presence-only value

### Requirement: Validate LLM configuration before secretary request
The system SHALL prevent secretary requests when the effective base URL, model, or API key is missing.

#### Scenario: Missing required setting
- **WHEN** the user attempts to send a secretary message without a complete effective LLM configuration
- **THEN** the system rejects the request with an actionable error message identifying the missing setting
