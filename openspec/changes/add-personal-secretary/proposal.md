## Why

The app is useful for managing work, but it does not yet provide a personal secretary that can remember what the user said, ask good questions, suggest next steps, and follow up later. Adding a Secretary module turns the desktop app into a local productivity companion with a configurable persona, domain skills, transparent memory, and reminder-oriented follow-up.

## What Changes

- Add a **Secretary** module as a new app tab alongside existing productivity tools
- Add LLM provider configuration for `LLM_BASE_URL`, `LLM_MODEL`, and `LLM_API_KEY`, with environment variables taking precedence over saved configuration
- Allow users to create or select a secretary profile made from role, domain, and one or more loaded skills
- Give each secretary profile a configurable persona: name, voice, values, interaction style, and response boundaries
- Support built-in roles for domain question answering, domain question asking, idea criticism, and idea generation
- Add local secretary memory for durable user preferences, project facts, recurring ideas, decisions, and profile-specific context
- Let users review, pin, edit, archive, and forget memories
- Let the secretary read selected app context from the user's Todo list, Pomodoro milestones, and Sticky Notes
- Let the secretary propose Sticky Note edits and apply them only after user confirmation
- Let the secretary give suggestions, ask clarifying questions, identify follow-ups, and propose reminders
- Let users confirm secretary-proposed reminders before they are saved
- Load skill files from a user-specified folder and expose them for profile selection
- Start and continue conversations using the selected LLM configuration and secretary profile
- Save conversations into a user-specified folder for later review or reuse
- Persist non-secret secretary settings, profile metadata, skill folder, memory entries, reminders, and conversation metadata in SQLite
- Avoid storing API keys in SQLite when an environment variable is available

## Capabilities

### New Capabilities
- `llm-configuration`: Provider configuration, environment-variable precedence, and validation for LLM connectivity
- `secretary-profiles`: Role, domain, persona, and skill selection used to shape secretary behavior
- `secretary-persona`: Configurable secretary identity, voice, values, interaction style, and continuity rules
- `secretary-memory`: Local memory capture, retrieval, review, editing, pinning, archiving, and forgetting
- `secretary-app-context`: Controlled access to selected Todo, milestone, and note context, plus user-confirmed Sticky Note updates
- `secretary-follow-up`: Secretary suggestions, clarifying questions, follow-up extraction, and user-confirmed reminders
- `skill-library`: Loading skill files from a selected folder and making them available to secretary profiles
- `secretary-conversation`: Secretary session lifecycle, message exchange with the LLM, and conversation saving/export

### Modified Capabilities

## Impact

- **Rust backend**: New SQLite tables for secretary settings, profiles, personas, skills, memories, reminders, and conversations; new Tauri commands for configuration, folder selection, skill loading, app context reading, confirmed note updates, memory management, reminder management, LLM completion, and conversation save/load
- **Frontend**: New Secretary tab with configuration panel, persona/profile selector, context panel for Todos/milestones/notes, memory panel, reminder panel, skill selector, transcript, composer, and conversation save controls
- **Dependencies**: HTTP client support in Rust for OpenAI-compatible chat completion requests; optional file dialog support through existing Tauri APIs or plugins
- **Security**: API keys are read from environment variables first and should be masked in the UI; saved keys must not be displayed in plain text
- **Privacy**: Memories and app context are local by default, user-visible, and controllable; memory, todo, milestone, and note content sent to the LLM is limited to selected or relevant entries for the active conversation; note mutations require explicit user confirmation
- **Existing code**: App tab navigation expands to include Secretary; existing Todo, Pomodoro, Toolbox, and Sticky Notes behavior remains unchanged
