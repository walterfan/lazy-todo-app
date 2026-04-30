## Context

Lazy Todo App is a local-first Tauri v2 desktop app with React/TypeScript UI, Rust commands, SQLite persistence, and OpenAI-compatible LLM integration. The current Secretary module introduced one assistant with persona, memory, app context, streaming responses, settings, reminders, and confirmed note edits.

The next product step is broader than a single secretary. Users want multiple virtual-human AI Agents, such as Confucius, Steve Jobs, Laozi, or custom community-created Agents, that can be installed, enabled, disabled, selected, and discussed with individually or together. The design must preserve local privacy and Tauri IPC boundaries while making Agent behavior independent from app core code.

## Goals / Non-Goals

**Goals:**
- Rename and refactor the Secretary experience into an Agents module.
- Deliver an initial usable phase with a bundled `plugins/` folder containing Personal Secretary and Confucius as standard plugin folders.
- Let the user select one Agent in the Agents chat window and talk with it using streaming replies.
- Give the app soul and memory through a local user identity profile, durable memories, and previous-conversation recall.
- Support static, configuration-driven Agent plugin packages with no executable plugin code.
- Load, validate, enable, disable, uninstall, and hot-refresh Agent plugins from local folders.
- Support single-Agent chat and multi-Agent group discussion with streaming replies.
- Let Agents request app-owned built-in tools for notes, todos, milestones, and scoped files.
- Let users register approved external CLI tools, such as `helper` and `skill`, for Agents to call through controlled schemas.
- Persist sessions, messages, Agent participation, plugin state, and local Agent data in SQLite.
- Use sqlite-vec for per-Agent local RAG knowledge where `rag_knowledge.md` is present and enabled.
- Keep LLM API credentials, app context selection, identity, memory, reminders, and note-edit confirmation local and user-controlled.
- Migrate existing Secretary data into a default personal Agent when feasible.

**Non-Goals:**
- Remote plugin marketplace backend, upload review workflow, or account-based community hosting in this change.
- Loading arbitrary executable code from Agent plugins.
- Cloud sync of Agent plugins, conversations, memories, or vector indexes.
- Autonomous write/change tool execution by Agents without explicit app-defined confirmation flows.
- Silent or hidden long-term memory creation without user visibility and edit/delete controls.
- Arbitrary shell access, shell-script generation, or plugin-provided executable tools.
- Multi-provider LLM abstraction beyond the current OpenAI-compatible chat and embedding shape.

## Phasing

**Phase 1: Built-in single-Agent chat**
- Add a bundled `plugins/` directory with two subfolders: `secretary/` and `confucius/`.
- Make both bundled Agent folders follow the standard plugin structure: `manifest.json`, `system_prompt.md`, `config.json`, `avatar.png`, `README.md`, and optional `rag_knowledge.md`.
- Scan the bundled plugin directory on startup so built-in Agents use the same loading and validation path as future external plugins.
- Show an Agents chat page with an Agent selector, selected Agent identity, transcript, composer, and streaming reply state.
- Support initial built-in read tools and proposed write/change tool calls with confirmation.
- Add a basic user identity and memory panel, attach recent relevant conversation summaries to Agent prompts, and keep memory recall visible to the user.
- Persist sessions and messages with Agent IDs.
- Reuse the existing LLM configuration and streaming chat completion flow.
- Keep external CLI tool execution, plugin installation, plugin ZIP packaging, RAG indexing, and multi-Agent group chat behind later tasks.

**Phase 2: Local plugin runtime**
- Add plugin directory scanning, validation, enable/disable, and plugin management UI.

**Phase 3: Local RAG and multi-Agent collaboration**
- Add sqlite-vec RAG ingestion/retrieval and group discussion dispatch.

## Decisions

### 1. Agents replace Secretary as the product model

**Decision**: Rename user-facing Secretary UI and backend concepts to Agents. Existing Secretary data migrates into a built-in personal Agent so current work is preserved.

**Rationale**: “Secretary” is one useful Agent persona, but the new model must support many virtual humans and group discussion. A default personal Agent gives continuity without forcing the old name into the architecture.

**Alternatives considered:**
- Keep Secretary and add Agents beside it: less risky short term, but duplicates chat, settings, memory, and LLM code.
- Treat Agents as just Secretary profiles: simpler, but does not support plugin lifecycle, packaging, RAG isolation, or community contribution.

### 1a. Phase 1 uses bundled plugin folders

**Decision**: The first implementation phase SHALL ship two built-in Agents as standard plugin folders under a bundled `plugins/` directory: `secretary/` and `confucius/`. They use the same manifest, prompt, config, avatar, README, validation, and loading path as future user-installed plugins.

**Rationale**: Bundled plugin folders make the Agents module usable immediately while also proving the real plugin contract. This avoids a special code-only Agent path that would later need to be removed.

**Alternatives considered:**
- Define built-ins only in Rust or database seed data: faster initially, but creates a second Agent definition format.
- Keep only Personal Secretary initially: simpler, but does not validate the “different virtual humans” product direction.

### 2. Plugins are static resource packages

**Decision**: An Agent plugin is a folder or zip package containing `manifest.json`, `system_prompt.md`, `config.json`, `avatar.png`, `README.md`, and optional `rag_knowledge.md`. Plugins cannot include executable code.

**Rationale**: Static packages are easy to author, audit, share, and load safely in a desktop app. They give the user ecosystem benefits without introducing arbitrary code execution.

**Alternatives considered:**
- WebAssembly or native code plugins: powerful, but too risky for v1 and harder to sandbox.
- Store all Agents only in SQLite: easier to query, but poor for community contribution and filesystem-based package sharing.

### 3. Rust owns plugin loading and validation

**Decision**: Rust commands implement plugin scanning, manifest parsing, validation, install/uninstall, enable/disable, RAG indexing, conversation dispatch, and LLM requests. React calls these capabilities only through Tauri `invoke()` and listens for stream events.

**Rationale**: This follows existing architecture rules, centralizes filesystem and database access, and keeps secrets out of frontend request code.

**Alternatives considered:**
- Let React read plugin files directly: simpler UI iteration, but violates the app boundary and weakens filesystem validation.
- Use a sidecar plugin daemon: flexible, but unnecessary for static packages and adds deployment complexity.

### 4. Plugin lifecycle is explicit

**Decision**: Track lifecycle state separately from files on disk: discovered, valid, invalid, loaded, enabled, disabled, active in session, and uninstalled. Invalid plugins remain visible with diagnostics instead of silently disappearing.

**Rationale**: Users and plugin authors need actionable feedback. A visible invalid state makes local ecosystem development much less mysterious.

**Alternatives considered:**
- Only show valid plugins: cleaner UI, but poor debugging.
- Reload all plugins only on app restart: simple, but fails the hot-load/hot-switch goal.

### 5. Multi-Agent chat uses session participants

**Decision**: A conversation session has one or more Agent participants. In single-Agent mode, one participant responds. In group mode, the dispatcher sends the user message to each selected enabled Agent, streams attributed replies, and persists each Agent response as a separate message.

**Rationale**: Separate participant/message records support speaker attribution, replay, export, and future moderator strategies. Phase 1 only needs one participant per session, but the schema should not block group discussion later.

**Alternatives considered:**
- Merge all Agent replies into one assistant message: compact, but loses attribution and makes group discussion hard to replay.
- Force turn-by-turn sequential debate in v1: richer, but slower and more complex than a first group discussion mode.

### 6. Shared context is read-only unless confirmed by app actions

**Decision**: Agents can receive shared conversation history, selected app context, prior Agent replies, user identity, relevant memory, previous conversation summaries, retrieved RAG snippets, and results from approved built-in tools. Agents cannot directly mutate todos, milestones, notes, memories, reminders, or files; app-defined confirmation flows remain required for write/change tools.

**Rationale**: Multi-Agent discussion should be informed but not surprising. The current confirmed note-edit and reminder patterns remain the safety boundary.

**Alternatives considered:**
- Let Agents autonomously write app data: powerful, but too easy to corrupt local user data.
- Give each Agent isolated context only: safe, but prevents real group discussion.

### 6a. App memory is separate from Agent persona

**Decision**: The app SHALL own the user's identity profile, durable memories, and previous-conversation summaries. Agent plugins define who the Agent is; app memory defines who the user is, what the app remembers, and which prior conversations may be recalled. Agents may propose durable memory, but the app stores it only after user confirmation.

**Rationale**: The user wants Agents with soul and memory, but that memory must not be hidden inside arbitrary persona prompts. Keeping app-owned memory separate makes it reusable across Agents, transparent, editable, and locally private.

**Alternatives considered:**
- Put user memory in each Agent plugin: simple for one Agent, but fragments the user's history and makes deletion hard.
- Attach full conversation history to every request: easy to reason about, but expensive, noisy, and risky for privacy.

### 6b. Conversation recall uses summaries plus recent messages

**Decision**: The dispatcher SHALL include current session history, recent messages, and relevant local conversation summaries within configured limits. Long sessions are summarized locally so Agents can remember previous conversations without replaying full transcripts.

**Rationale**: Useful memory requires continuity, but raw transcripts quickly exceed context limits. Summaries provide continuity while preserving the ability to inspect the original source session.

**Alternatives considered:**
- Only use the current session: safer and simpler, but Agents forget previous discussions.
- Vectorize every message immediately: powerful, but larger implementation scope than the initial identity and summary memory layer.

### 6c. Built-in tools are app-owned and mediated

**Decision**: Agents SHALL call only app-registered built-in tools. The initial registry includes `read_note`, `write_note`, `read_todo_list`, `add_todo_item`, `change_todo_item`, `read_milestones`, `change_milestone`, `read_file`, and `write_file`. Rust validates tool arguments, enforces scope, executes approved calls, logs results, and returns tool output to the Agent conversation.

**Rationale**: Tool use makes Agents genuinely useful inside the app while preserving the Tauri boundary and local safety model. Plugins describe persona and knowledge, but the app owns all capabilities that touch user data or files.

**Alternatives considered:**
- Let plugins define arbitrary tools: more flexible, but unsafe for static community plugins.
- Let the LLM mutate app data directly from natural language: fast, but too opaque and hard to audit.

### 6d. Read tools and write tools have different safety gates

**Decision**: Read tools can run without per-call confirmation when the user has enabled the relevant source for the session. Write/change tools produce a proposed action by default and execute only after explicit user confirmation. File tools are limited to configured safe roots, and writes require confirmation.

**Rationale**: Reading selected local context is part of useful Agent assistance, but changing notes, todos, milestones, and files can destroy user work. Confirmation keeps the Agent capable without becoming surprising.

**Alternatives considered:**
- Require confirmation for every read: safest, but too noisy for normal chat.
- Allow writes automatically after a global toggle: efficient, but risky for early versions.

### 6e. External CLI tools are user-registered app tools

**Decision**: Users MAY register local external CLI tools such as `helper`, `skill`, or other command-line integrations. The app stores each CLI tool registration with executable path or command name, allowed subcommands, JSON argument schema, working directory policy, environment allowlist, timeout, output limits, safety class, and enablement scope. Agents receive only enabled CLI schemas and cannot invent arbitrary shell commands.

**Rationale**: The user already has useful local CLIs. Registering them lets Agents use existing workflows without coupling those tools into the app binary or unsafe plugin packages.

**Alternatives considered:**
- Give Agents raw terminal access: flexible, but too dangerous and impossible to audit safely.
- Hard-code support for only `helper` and `skill`: faster initially, but misses the user's broader CLI ecosystem.
- Let Agent plugins bundle CLI commands: convenient for plugin authors, but violates the static no-executable plugin safety model.

### 6f. CLI execution avoids shell interpolation

**Decision**: External CLI calls SHALL be executed by spawning the configured executable directly with validated structured arguments. The app SHALL not pass Agent-generated strings through a shell. Sensitive, write, networked, or destructive CLI tools require confirmation before execution.

**Rationale**: Direct process spawning with argument validation reduces injection risk, keeps executions auditable, and lets the UI preview exactly what will run.

**Alternatives considered:**
- Execute a templated command string through the user's shell: easy to configure, but vulnerable to command injection and quoting mistakes.
- Require confirmation for every CLI call: safest, but too noisy for read-only helper commands once a user trusts a registration.

### 7. RAG is per-Agent and rebuildable

**Decision**: `rag_knowledge.md` is chunked and indexed per plugin using sqlite-vec. Index rows include plugin ID, plugin version, source file hash, chunk text, embedding model, embedding dimension, and vector data. Indexes are rebuildable and deleted on uninstall.

**Rationale**: Per-Agent indexes keep knowledge isolated and manageable. Rebuildable indexes avoid coupling user data to plugin package internals.

**Alternatives considered:**
- One global vector index: easier retrieval, but weaker isolation and harder uninstall cleanup.
- No RAG in v1: simpler, but misses a core requirement for virtual-human Agents with local knowledge.

### 8. LLM and embedding configuration stays global at first

**Decision**: Keep global LLM settings for base URL, model, and API key. Add embedding settings only if needed by sqlite-vec ingestion, still with environment-variable precedence over saved values.

**Rationale**: Most users need one local or remote LLM endpoint. Per-Agent model settings can be added later after the plugin runtime is stable.

**Alternatives considered:**
- Per-Agent LLM credentials: flexible, but more complex and riskier for secrets.
- Hard-code one model: simpler, but blocks local private deployments.

### 9. Plugin contribution is file-first

**Decision**: V1 supports local plugin package validation and zip install/export. The spec defines the contribution package format, but remote marketplace submission/review remains outside implementation.

**Rationale**: A file-first workflow lets users and community contributors build and test plugins now, while leaving room for a future hosted marketplace.

**Alternatives considered:**
- Build marketplace immediately: attractive, but requires backend hosting, moderation, identity, and legal workflows outside this desktop app scope.

## Risks / Trade-offs

- **[Risk] Invalid or malicious plugin content** -> Mitigation: static packages only, strict schema validation, size limits, path traversal prevention, no executable plugin code, and visible diagnostics.
- **[Risk] Prompt injection inside plugin knowledge** -> Mitigation: label retrieved snippets as untrusted knowledge, keep system prompt precedence, and never grant direct mutation powers.
- **[Risk] Tool calls can modify or expose sensitive user data** -> Mitigation: app-owned allowlist, argument validation, configured file roots, confirmation for writes/changes, and persistent tool-call audit records.
- **[Risk] External CLI tools can execute unsafe local actions** -> Mitigation: user registration only, no shell interpolation, schema-validated arguments, working-directory and environment allowlists, timeouts, output limits, confirmation for sensitive tools, and audit logs.
- **[Risk] Agents remember incorrect or sensitive facts** -> Mitigation: require confirmation for durable memory, show memory source metadata, allow edit/delete/scope changes, and support per-session memory recall disablement.
- **[Risk] Memory and previous conversation context bloats prompts** -> Mitigation: token limits, pinned memory priority, conversation summaries, recency/relevance selection, and visible context details per response.
- **[Risk] Multi-Agent conversations can be slow or expensive** -> Mitigation: configurable participant count, parallel dispatch limits, cancellation, streaming attribution, and token limits.
- **[Risk] sqlite-vec setup or embedding dimension mismatch** -> Mitigation: validate embedding dimension from config, store model/dimension metadata, and rebuild indexes when settings change.
- **[Risk] Large plugin folders slow startup** -> Mitigation: scan metadata first, cap file sizes, hash files for change detection, and move heavy RAG indexing behind explicit refresh/rebuild.
- **[Risk] Secretary migration can lose user trust if data changes unexpectedly** -> Mitigation: keep old tables until migration is verified, create a default personal Agent, and expose migration status.
- **[Risk] UI becomes too complex** -> Mitigation: separate Agents chat from Settings/plugin management, provide clear single-Agent and group modes, and keep advanced diagnostics in management panels.

## Migration Plan

1. Add new Agents database tables with `CREATE TABLE IF NOT EXISTS` without dropping Secretary tables.
2. Add Agents models and commands beside existing Secretary code.
3. Add bundled plugin folders for `secretary` and `confucius`, then scan them through the plugin engine.
4. Migrate Secretary conversations, profile, and memory into Agent sessions and app-owned identity/memory where mapping is safe; record migration status.
5. Add Agents UI and Settings/plugin management UI while leaving Secretary hidden or aliased during transition.
6. Redirect navigation from Secretary to Agents once feature parity is verified.
7. Roll back by hiding Agents UI and continuing to use the previous Secretary tables/commands.

## Open Questions

- Which embedding endpoint and environment variables should be standardized for sqlite-vec ingestion?
- Should group discussion dispatch be fully parallel in v1, or use a moderator/sequential strategy for better cross-Agent response quality?
- Which migrated Secretary memories should become global app memory, and which should stay scoped to the `secretary` Agent?
- Should plugin zip install be allowed from arbitrary paths immediately, or limited to copying folders into the configured plugin directory first?
