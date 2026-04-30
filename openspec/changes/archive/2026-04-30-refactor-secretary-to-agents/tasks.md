## 1. Dependencies & Module Setup

- [x] 1.1 Decide sqlite-vec crate/loading strategy and add required Rust dependencies
- [x] 1.2 Add any embedding/HTTP request dependencies needed for RAG ingestion
- [x] 1.3 Create `src-tauri/src/models/agents.rs` for plugin, session, message, identity, memory, RAG, and migration structs
- [x] 1.4 Register the Agents model module in `src-tauri/src/models/mod.rs`
- [x] 1.5 Create `src-tauri/src/commands/agents.rs` and register it in `commands/mod.rs`
- [x] 1.6 Register Agents Tauri commands in `src-tauri/src/lib.rs`

## 2. Database Schema & Migration

- [x] 2.1 Add SQLite tables for Agent plugin metadata, validation state, lifecycle state, and local file paths
- [x] 2.2 Add SQLite tables for Agent sessions, session participants, messages, turn metadata, and stream/error status
- [x] 2.3 Add SQLite tables for user identity profile, durable app memory, memory proposals, conversation summaries, memory usage records, and Agent private data
- [x] 2.4 Add SQLite tables for external CLI tool registrations, permission scopes, and execution audit records
- [x] 2.5 Add SQLite tables for memory links and Secretary-to-Agents migration status
- [ ] 2.6 Add sqlite-vec tables or virtual tables for per-Agent RAG chunks and embeddings
- [x] 2.7 Add indexes for plugin ID, session ID, participant Agent ID, memory scope, external tool ID, timestamps, and RAG source hash
- [x] 2.8 Implement safe in-place migrations without dropping existing Secretary tables

## 3. Plugin Format Validation

- [x] 3.1 Create bundled `plugins/secretary/` and `plugins/confucius/` directories following the standard plugin structure
- [x] 3.2 Add `manifest.json`, `system_prompt.md`, `config.json`, `avatar.png`, and `README.md` for `secretary`
- [x] 3.3 Add `manifest.json`, `system_prompt.md`, `config.json`, `avatar.png`, `README.md`, and optional `rag_knowledge.md` for `confucius`
- [x] 3.4 Define bundled plugin metadata and prompt/config content for Personal Secretary and Confucius entirely through plugin files
- [x] 3.5 Implement plugin ID validation for lowercase letters, digits, underscores, uniqueness, and 32-character maximum
- [x] 3.6 Implement `manifest.json` parser and schema validation
- [x] 3.7 Implement `config.json` parser with range validation for temperature, top_p, rag_top_k, and embedding_dim
- [x] 3.8 Implement required file checks for manifest, system prompt, config, avatar, and README
- [x] 3.9 Implement optional `rag_knowledge.md` detection and RAG-enabled compatibility checks
- [x] 3.10 Implement plugin file size limits and safe filename/path validation
- [x] 3.11 Return structured validation diagnostics for invalid plugins

## 4. Plugin Runtime & Lifecycle

- [x] 4.1 Implement bundled plugin directory discovery for the app-shipped `plugins/` folder
- [x] 4.2 Scan `secretary/` and `confucius/` through the same validation/loading path as external plugins
- [x] 4.3 Mark bundled plugin folders as app-managed and not uninstallable by default
- [x] 4.4 Implement command to list bundled and installed Agents with IDs, names, descriptions, avatars, tags, and enabled status
- [x] 4.5 Implement configured Agent plugin directory read/save commands
- [x] 4.6 Implement startup and manual plugin directory scan for bundled and user plugin directories
- [x] 4.7 Persist discovered, valid, invalid, loaded, enabled, disabled, and uninstalled lifecycle states
- [x] 4.8 Implement enable and disable plugin commands
- [x] 4.9 Implement uninstall command for user-installed app-managed plugin directories with path traversal safeguards
- [x] 4.10 Implement ZIP install command that extracts exactly one plugin folder safely
- [x] 4.11 Implement hot refresh behavior that reloads changed manifests and prompts without app restart
- [x] 4.12 Add plugin detail command returning manifest, config, README, avatar path, validation diagnostics, and local data status

## 5. Secretary-to-Agents Migration

- [x] 5.1 Create migration routine that detects existing Secretary settings, persona, profile, memory, reminders, and conversations
- [x] 5.2 Map existing Secretary persona/profile data onto the bundled `secretary` plugin state when no migrated Agent state exists
- [x] 5.3 Migrate Secretary conversations into Agent sessions where sender mapping is safe
- [x] 5.4 Preserve Secretary tables and record migration status/errors
- [x] 5.5 Add command to read migration status for Settings or Agent management UI

## 6. RAG Knowledge Indexing

- [x] 6.1 Implement `rag_knowledge.md` chunking with deterministic chunk IDs and source hash
- [ ] 6.2 Implement embedding request flow using configured OpenAI-compatible embedding endpoint or selected local embedding provider
- [ ] 6.3 Store embeddings in sqlite-vec with plugin ID, version, source hash, model, dimension, chunk text, and vector data
- [ ] 6.4 Implement stale-index detection when plugin version, config, source hash, model, or dimension changes
- [x] 6.5 Implement rebuild command for one plugin and all plugins
- [x] 6.6 Implement RAG cleanup on plugin uninstall
- [x] 6.7 Implement retrieval command/helper that returns top-k chunks only for the active Agent

## 7. Agent Prompt Assembly

- [x] 7.1 Implement deterministic prompt assembly from Agent system prompt, runtime config, response style, ban topics, user identity, selected app context, app memory, previous conversation summaries, RAG snippets, tool results, and conversation history
- [x] 7.2 Add token and character limits for prompt sections and retrieved RAG snippets
- [x] 7.3 Label retrieved RAG snippets as local knowledge and keep system prompt precedence
- [x] 7.4 Include enabled built-in tool schemas in the LLM request for Agent sessions
- [x] 7.5 Include enabled external CLI tool schemas in the LLM request for Agent sessions
- [x] 7.6 Label user identity, app memory, and conversation summaries as app-owned context distinct from Agent persona
- [x] 7.7 Prioritize pinned memories, recent relevant summaries, selected app context, and current session history when limits are reached
- [x] 7.8 Preserve existing environment-first LLM configuration resolution
- [x] 7.9 Preserve streaming-compatible OpenAI chat completion payload construction

## 8. Built-In Tool Registry & Executor

- [x] 8.1 Define built-in tool metadata, JSON argument schemas, result schemas, permission category, and safety class
- [x] 8.2 Implement `read_note` through the existing Sticky Note read path
- [x] 8.3 Implement `write_note` as a proposed note write action requiring confirmation
- [x] 8.4 Implement `read_todo_list` through the existing Todo list path
- [x] 8.5 Implement `add_todo_item` as a proposed todo add action requiring confirmation
- [x] 8.6 Implement `change_todo_item` as a proposed todo change action requiring confirmation
- [x] 8.7 Implement `read_milestones` through the existing Pomodoro milestone path
- [x] 8.8 Implement `change_milestone` as a proposed milestone change action requiring confirmation
- [x] 8.9 Implement `read_file` with configured safe-root validation, size limits, and text/binary safeguards
- [x] 8.10 Implement `write_file` as a proposed file write action requiring confirmation and safe-root validation
- [x] 8.11 Persist tool call audit records with session ID, Agent ID, tool name, arguments, status, result, and timestamps
- [x] 8.12 Add commands to confirm or reject proposed write/change tool calls

## 9. External CLI Tool Registry & Executor

- [x] 9.1 Define external CLI tool registration model with name, executable path or command name, allowed subcommands, argument schema, working directory policy, environment allowlist, timeout, output limit, safety class, and enabled scopes
- [x] 9.2 Add commands to create, list, update, test, enable, disable, and delete external CLI tool registrations
- [x] 9.3 Validate executable availability and reject registrations that cannot be resolved or executed
- [x] 9.4 Validate Agent-supplied CLI arguments against the registered JSON schema
- [x] 9.5 Execute registered CLIs by spawning the executable directly without shell interpolation
- [x] 9.6 Enforce working directory policy, environment allowlist, timeout, stdout/stderr output limits, and cancellation behavior
- [x] 9.7 Create proposed CLI actions for sensitive, write, destructive, or networked tools and execute them only after user confirmation
- [x] 9.8 Capture exit code, bounded stdout, bounded stderr, duration, timeout status, and truncation metadata as tool results
- [x] 9.9 Persist external CLI audit records with masked sensitive arguments and environment values
- [x] 9.10 Add sample local registrations or presets for common `helper` and `skill` CLI read-only commands where available

## 10. Conversation Dispatcher

- [x] 10.1 Implement command to create single-Agent sessions
- [x] 10.2 Implement command to create a single-Agent session for Personal Secretary or Confucius
- [x] 10.3 Implement command to load sessions and messages with speaker attribution
- [x] 10.4 Implement single-Agent send message flow with streaming events and final persistence
- [x] 10.5 Attach relevant identity, memory, and previous conversation context before the LLM request
- [x] 10.6 Detect tool calls in LLM responses and route them through the built-in tool executor or external CLI executor
- [x] 10.7 Return tool results to the Agent turn and continue response generation when needed
- [x] 10.8 Persist user messages, Agent draft/final messages, stream errors, tool calls, and completion status
- [x] 10.9 Update or schedule conversation summary refresh after message persistence
- [x] 10.10 Implement transcript export with Agent names, IDs, timestamps, tool calls, and session metadata
- [ ] 10.11 Implement command to create multi-Agent group sessions with participant validation
- [ ] 10.12 Implement group send message flow that dispatches to selected Agents in parallel with per-Agent stream IDs
- [ ] 10.13 Allow one Agent stream failure without discarding other Agent replies in the same group turn

## 11. Identity, Memory & Conversation Recall

- [x] 11.1 Add commands to read and update the local user identity profile
- [x] 11.2 Add commands to create, list, edit, delete, pin, archive, and rescope durable app memories
- [x] 11.3 Preserve proposed memory confirmation before storing memory
- [x] 11.4 Add memory proposal records with source session, source Agent, source message, proposed text, status, and timestamps
- [x] 11.5 Implement conversation summary creation/update for long-running sessions
- [x] 11.6 Implement relevant previous conversation summary selection by Agent, topic, recency, and user-selected memory settings
- [ ] 11.7 Add commands to inspect which identity fields, memories, and summaries were used for a response
- [ ] 11.8 Add per-session control to disable durable memory and previous conversation recall
- [x] 11.9 Migrate existing Secretary profile and memory into app-owned identity/memory where safe

## 12. App Context, Reminders & Confirmed Actions

- [x] 12.1 Reuse existing selected Todo, milestone, and Sticky Note context aggregation for Agents
- [ ] 12.2 Reuse existing reminder storage where applicable or map reminders into Agent-compatible tables
- [ ] 12.3 Preserve proposed reminder confirmation before storing reminder
- [x] 12.4 Preserve proposed Sticky Note edit preview and explicit user confirmation
- [x] 12.5 Ensure Agents cannot directly mutate Todos, milestones, notes, memory, reminders, files, or external CLI tools without app-defined confirmation and policy checks

## 13. Frontend Types & Hooks

- [ ] 13.1 Create `src/types/agents.ts` for plugin manifests, configs, lifecycle state, validation diagnostics, sessions, participants, messages, identity profile, memories, conversation summaries, RAG status, built-in tool calls, external CLI tool registrations, external CLI calls, and migration status
- [ ] 13.2 Create `useAgents` hook for loading plugins, settings, sessions, identity, app context, memory, conversation summaries, reminders, built-in tools, external CLI tools, and migration status
- [x] 13.3 Add hook actions for plugin refresh, install, uninstall, enable, disable, and RAG rebuild
- [ ] 13.4 Add hook actions for creating sessions, selecting participants, sending single-Agent messages, and sending group messages
- [x] 13.5 Add hook actions for confirming and rejecting proposed write/change tool calls, proposed external CLI calls, and proposed memories
- [x] 13.6 Add hook actions for updating identity profile, managing durable memories, and managing external CLI registrations
- [ ] 13.7 Add stream event listeners keyed by session, turn, and Agent ID
- [ ] 13.8 Replace Secretary-specific hook usage with Agents hook usage

## 14. Agents Chat UI

- [x] 14.1 Rename Secretary navigation item and page title to Agents
- [x] 14.2 Replace `SecretaryPanel` with an Agents chat panel
- [x] 14.3 Create Agent selector that shows Personal Secretary and Confucius in the initial phase
- [x] 14.4 Display selected Agent avatar, name, tags, description, author, and version
- [x] 14.5 Render speaker-attributed chat messages for the user and selected Agent
- [x] 14.6 Render streaming draft message for the selected Agent
- [x] 14.7 Render proposed tool-call cards with arguments, preview, confirm, and reject controls
- [x] 14.8 Render proposed external CLI action cards with command preview, arguments, safety class, confirm, and reject controls
- [x] 14.9 Render proposed memory cards with source context, edit, confirm, and reject controls
- [ ] 14.10 Render completed and rejected tool-call results in the transcript
- [x] 14.11 Show context details for identity, memories, and previous conversations used by a response
- [x] 14.12 Add session list, load session, new session, and export transcript controls
- [ ] 14.13 Extend Agent selection UI with multi-Agent group mode
- [ ] 14.14 Render streaming draft messages per Agent in group sessions
- [ ] 14.15 Show per-Agent stream errors without hiding successful Agent replies

## 15. Settings & Plugin Management UI

- [x] 15.1 Move Agent plugin directory settings into Settings
- [x] 15.2 Add plugin management list with valid, invalid, enabled, disabled, and uninstalled states
- [x] 15.3 Add plugin install from ZIP or folder command UI
- [x] 15.4 Add plugin refresh, enable, disable, uninstall, and RAG rebuild controls
- [x] 15.5 Add plugin detail view with README, diagnostics, local data status, and RAG index status
- [x] 15.6 Keep LLM configuration in Settings and relabel it for Agents
- [x] 15.7 Keep app context selection in Settings or a clearly discoverable Agent settings panel
- [ ] 15.8 Add settings for user identity, durable memory, memory recall, enabled built-in tool categories, external CLI registrations, and safe file roots
- [x] 15.9 Add external CLI registration UI for command path, subcommands, schemas, working directory, environment allowlist, timeout, output limits, safety class, test run, and enablement scopes
- [x] 15.10 Show Secretary migration status and default personal Agent creation result

## 16. Styling & UX

- [ ] 16.1 Update layout and CSS from Secretary naming to Agents naming
- [x] 16.2 Ensure Agent cards, plugin diagnostics, long names, paths, README text, streamed messages, tool-call cards, external CLI action cards, memory cards, and context details wrap without overlap
- [ ] 16.3 Keep group chat readable with compact speaker headers and avatars
- [x] 16.4 Make invalid plugin states visible but not alarming
- [x] 16.5 Make local/private data, memory usage, built-in tool permissions, and external CLI permission boundaries visible in Settings and Agent details

## 17. Tests & Verification

- [x] 17.1 Add Rust unit tests for bundled `secretary` and `confucius` plugin folder validation/listing
- [x] 17.2 Add Rust unit tests for plugin ID, manifest, config, and required-file validation
- [x] 17.3 Add Rust tests for safe ZIP extraction and path traversal rejection
- [x] 17.4 Add Rust tests for plugin scan, enable, disable, uninstall, and hot refresh behavior
- [x] 17.5 Add Rust tests for identity profile persistence and prompt inclusion
- [x] 17.6 Add Rust tests for memory proposal confirmation, edit/delete, scope, and prompt exclusion after deletion
- [x] 17.7 Add Rust tests for conversation summary creation and previous conversation recall
- [ ] 17.8 Add Rust tests for sqlite-vec RAG index lifecycle where test dependencies allow
- [x] 17.9 Add Rust tests for read tool execution and write/change proposal creation
- [x] 17.10 Add Rust tests for file safe-root enforcement
- [x] 17.11 Add Rust tests for external CLI registration validation, unavailable executable handling, and argument schema rejection
- [x] 17.12 Add Rust tests for external CLI execution timeout, output truncation, environment allowlist, and audit masking
- [ ] 17.13 Add Rust tests for single-Agent and multi-Agent conversation persistence
- [x] 17.14 Add TypeScript type checking for Agents UI and hooks
- [x] 17.15 Run frontend build
- [x] 17.16 Run Tauri backend checks
- [ ] 17.17 Manually verify selecting Personal Secretary and chatting with it
- [ ] 17.18 Manually verify Agent knows the user's profile and recalls a previous relevant conversation
- [ ] 17.19 Manually verify user can inspect, edit, delete, and disable memory recall
- [ ] 17.20 Manually verify Agent can read notes, todo list, milestones, and allowed files
- [ ] 17.21 Manually verify Agent-proposed note, todo, milestone, and file writes require confirmation
- [ ] 17.22 Manually verify registering and testing a read-only external CLI such as `helper` or `skill`
- [ ] 17.23 Manually verify sensitive external CLI calls require confirmation and are audited
- [ ] 17.24 Manually verify selecting Confucius and chatting with it
- [ ] 17.25 Manually verify installing Confucius-style sample plugin and chatting with it
- [ ] 17.26 Manually verify group discussion with at least two sample Agents
- [ ] 17.27 Manually verify migration from existing Secretary data into the default personal Agent
