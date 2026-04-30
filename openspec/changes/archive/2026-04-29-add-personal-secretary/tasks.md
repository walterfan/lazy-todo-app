## 1. Dependencies & Data Model

- [x] 1.1 Add Rust HTTP and JSON support needed for OpenAI-compatible chat completion requests
- [x] 1.2 Create `src-tauri/src/models/secretary.rs` with LLM settings, effective settings, personas, secretary roles, profiles, skills, memories, reminders, conversations, and messages
- [x] 1.3 Register the secretary model module in `src-tauri/src/models/mod.rs`
- [x] 1.4 Add SQLite tables for secretary settings, personas, secretary profiles, profile skill links, loaded skill metadata, memory entries, reminder entries, and saved conversation metadata
- [x] 1.5 Implement database functions for reading/saving fallback LLM settings, personas, profiles, skill metadata, memory entries, reminders, and conversation metadata
- [x] 1.6 Add secretary app-context structs for Todo context, milestone context, note context, selected context, used-context metadata, proposed note edits, and confirmed note edit results

## 2. Backend Secretary Configuration

- [x] 2.1 Create `src-tauri/src/commands/secretary.rs` and register it in `commands/mod.rs`
- [x] 2.2 Implement effective LLM configuration resolution with environment variables taking precedence over saved settings
- [x] 2.3 Implement commands to read masked configuration, save fallback configuration, and validate missing required settings
- [x] 2.4 Ensure API key values are not returned in plain text and are not logged
- [x] 2.5 Register secretary commands in `lib.rs` invoke handler

## 3. Skill Library Backend

- [x] 3.1 Implement command to save and read the configured skill folder path
- [x] 3.2 Implement skill folder scanning for supported text skill files with name, source path, summary, and content metadata
- [x] 3.3 Add file size and supported-extension safeguards for skill loading
- [x] 3.4 Return skipped-file diagnostics without failing the whole skill refresh
- [x] 3.5 Persist refreshed skill metadata so skills are available after restart

## 4. Secretary Profiles Backend

- [x] 4.1 Define built-in role templates for domain question answering, domain question asking, idea criticism, and idea generation
- [x] 4.2 Implement commands to create, update, list, and select secretary profiles
- [x] 4.3 Persist profile role, domain, selected persona, and selected skill references
- [x] 4.4 Implement deterministic system prompt assembly from persona, role, domain, selected skill contents, and selected memory entries

## 5. Secretary Persona Backend

- [x] 5.1 Implement default personal secretary persona seed data for first run
- [x] 5.2 Implement commands to create, update, list, select, and delete personas
- [x] 5.3 Persist persona name, voice, values, interaction style, and boundaries
- [x] 5.4 Include active persona summaries in secretary configuration/profile responses
- [x] 5.5 Ensure persona instructions remain separate from role instructions during prompt assembly

## 6. Memory Backend

- [x] 6.1 Implement commands to create, update, list, pin, archive, and delete memory entries
- [x] 6.2 Add memory scope support for global, domain, and profile-specific memories
- [x] 6.3 Implement secretary-proposed memory flow that requires user confirmation before storage
- [x] 6.4 Implement deterministic memory retrieval with status, scope, pinning, relevance, count, and character limits
- [x] 6.5 Return memory usage metadata with secretary responses so the UI can show which memories were used

## 7. Reminder & Follow-Up Backend

- [x] 7.1 Implement commands to create, list, edit, complete, snooze, and delete reminders
- [x] 7.2 Persist reminder title, notes, due time, status, source conversation, and created/updated timestamps
- [x] 7.3 Implement secretary-proposed reminder flow that requires user confirmation before storage
- [x] 7.4 Implement due-reminder query for the Secretary panel
- [x] 7.5 Use available notification support for due reminders when notifications are enabled

## 8. App Context Backend

- [x] 8.1 Implement command to read Todo context with title, description, priority, deadline, completion status, and created time
- [x] 8.2 Implement command to read Pomodoro milestone context with name, deadline, and status
- [x] 8.3 Implement command to read Sticky Note context with title, content, color, created time, and updated time
- [x] 8.4 Implement context selection support for source-level and item-level inclusion
- [x] 8.5 Implement context prompt formatting with count and character limits
- [x] 8.6 Return used-context metadata with secretary responses so the UI can show which Todos, milestones, and notes were included
- [x] 8.7 Implement proposed Sticky Note edit payloads with target note, changed fields, and before/after preview
- [x] 8.8 Implement confirmed Sticky Note edit command by reusing the existing note update persistence path
- [x] 8.9 Record accepted and rejected secretary note edit metadata in the conversation transcript

## 9. Conversation Backend

- [x] 9.1 Implement command to configure and read the conversation save folder
- [x] 9.2 Implement command to start a conversation with the active profile context
- [x] 9.3 Implement command to send a user message to the configured LLM and return the secretary response
- [x] 9.4 Implement error handling for missing configuration, endpoint failures, and unsupported response shapes
- [x] 9.5 Implement transcript saving as Markdown plus JSON sidecar in the configured folder
- [x] 9.6 Implement commands to list saved conversations and load a saved transcript
- [x] 9.7 Implement streaming LLM command with chunk, error, and finish events keyed by stream ID

## 10. Frontend Types & Hooks

- [x] 10.1 Create `src/types/secretary.ts` for settings, personas, roles, profiles, skills, memories, reminders, app context, conversations, and messages
- [x] 10.2 Create a hook for loading and saving masked LLM configuration through `invoke()`
- [x] 10.3 Create a hook for skill folder selection, refresh, loaded skill state, and skipped-file diagnostics
- [x] 10.4 Create a hook for persona CRUD and active persona selection
- [x] 10.5 Create a hook for profile CRUD and active profile selection
- [x] 10.6 Create a hook for memory list, create, edit, pin, archive, delete, and proposed-memory confirmation
- [x] 10.7 Create a hook for reminder list, create, edit, complete, snooze, delete, and proposed-reminder confirmation
- [x] 10.8 Create a hook for Todo, milestone, and note context loading and selection
- [x] 10.9 Create a hook for proposed note edit preview, confirm, reject, and result state
- [x] 10.10 Create a hook for conversation state, sending messages with selected context, saving transcripts, and loading saved conversations
- [x] 10.11 Add stream event listeners and transient pending/streaming message state to the secretary hook

## 11. Secretary UI

- [x] 11.1 Create `SecretaryPanel` and add a Secretary tab to `App.tsx`
- [x] 11.2 Create LLM configuration controls with masked API key status and environment override indicators
- [x] 11.3 Create persona controls for secretary name, voice, values, interaction style, and boundaries
- [x] 11.4 Create role/domain/profile controls with built-in role selection and persona selection
- [x] 11.5 Create context panel for Todo, milestone, and note source toggles plus item selection
- [x] 11.6 Create note edit confirmation UI with target note, before/after preview, confirm, edit-before-apply, and reject actions
- [x] 11.7 Create memory panel for review, edit, pin, archive, delete, and proposed-memory confirmation
- [x] 11.8 Create reminder panel for due reminders, upcoming reminders, complete, snooze, edit, delete, and proposed-reminder confirmation
- [x] 11.9 Create skill library controls for folder path, refresh action, selectable skills, and skipped-file messages
- [x] 11.10 Create secretary transcript and composer UI with loading, error, retry, suggestion, question, memory-used, app-context-used, and note-edit indicators
- [x] 11.11 Create conversation save/load controls for the configured conversation folder
- [x] 11.12 Render pending user messages and streaming assistant chunks before final conversation persistence

## 12. Styling & UX

- [x] 12.1 Add Secretary styles to match the existing dark sidebar/content layout
- [x] 12.2 Keep controls compact and scannable within the app's desktop layout
- [x] 12.3 Ensure long model names, folder paths, skill names, memory entries, reminder text, Todo titles, note text, milestone names, and messages wrap without overlapping
- [x] 12.4 Provide disabled states for missing configuration, missing profile context, and in-flight requests
- [x] 12.5 Make persona, memory, reminders, app context, note edit confirmations, suggestions, and questions visible enough to feel present without crowding the secretary experience

## 13. Verification

- [x] 13.1 Run Rust checks for the Tauri backend
- [x] 13.2 Run TypeScript type checking
- [x] 13.3 Run frontend build
- [x] 13.10 Re-run Rust checks, TypeScript type checking, and frontend build after adding streaming replies
- [ ] 13.4 Manually verify environment-variable precedence over saved configuration
- [ ] 13.5 Manually verify skill loading, persona setup, role/domain profile setup, secretary request, save transcript, and load transcript flows
- [ ] 13.6 Manually verify memory create, proposed memory confirmation, edit, pin, archive, delete, and memory-used indicators
- [ ] 13.7 Manually verify Todo context, milestone context, note context, source toggles, item selection, and app-context-used indicators
- [ ] 13.8 Manually verify secretary-proposed Sticky Note edit preview, confirm, reject, edit-before-apply, persistence, and transcript metadata
- [ ] 13.9 Manually verify suggestion, clarifying question, proposed reminder confirmation, due reminder, complete, snooze, edit, and delete flows
