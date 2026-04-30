## Why

The current Secretary module is useful as a single personal assistant, but the product direction is broader: the app should let users talk with many virtual-human AI Agents such as Confucius, Steve Jobs, Laozi, or custom community Agents. Refactoring from Secretary to Agents creates a plugin-based foundation for independent Agent packages, multi-Agent conversations, local private knowledge, and future ecosystem contribution.

## What Changes

- **BREAKING**: Rename the user-facing Secretary module to Agents and migrate Secretary concepts into the new Agent model.
- Start the first phase with a bundled `plugins/` folder containing two standard Agent plugin subfolders: `secretary/` and `confucius/`.
- Add an Agents chat window where the user selects one built-in Agent and talks with it using streaming LLM replies.
- Add a local Agent plugin system where each Agent is a static resource package with metadata, system prompt, runtime config, avatar, README, and optional RAG knowledge.
- Add plugin discovery, validation, enable/disable, install/uninstall, reload, and hot-switch behavior from a configured local plugin directory.
- Add standardized Agent plugin packaging rules so users and community contributors can create Agents without writing Rust or TypeScript code.
- Add single-Agent conversation mode first, with multi-Agent group discussion retained as a later phase of the same architecture.
- Add app-owned identity and memory so Agents know who they are talking to and can recall relevant previous conversations with user-visible controls.
- Add an app-owned built-in tool layer so Agents can request safe tools such as reading/writing notes, reading/changing todos, reading/changing milestones, and reading/writing files.
- Add a user-registered external CLI tool layer so Agents can call approved local CLIs such as `helper`, `skill`, or other command-line integrations through safe schemas and audit logs.
- Add local storage for Agent plugins, sessions, messages, Agent participation, plugin state, and private plugin data.
- Add local RAG indexing and retrieval for Agent-specific knowledge using SQLite plus sqlite-vec, with deterministic limits and clear data isolation.
- Preserve existing LLM configuration, streaming replies, conversation persistence, app context selection, memory, reminders, and confirmed note-edit safety where they still apply to Agents.
- Add Settings and management UI for Agent plugin directory, installed plugins, validation errors, activation, and local data controls.

## Capabilities

### New Capabilities

- `agent-plugin-format`: Defines the required Agent plugin directory structure, manifest schema, prompt/config/avatar/README files, naming rules, validation rules, and packaging expectations.
- `agent-plugin-runtime`: Covers local plugin discovery, lifecycle states, install/uninstall, enable/disable, reload, hot-switch, validation diagnostics, and isolation between Agent packages.
- `agent-conversations`: Covers single-Agent chat, multi-Agent group chat, streaming responses, shared context, speaker attribution, session/message persistence, and transcript export.
- `agent-identity-memory`: Covers the user's local identity profile, durable app memory, previous-conversation summaries, memory proposal/confirmation, prompt attachment, transparency, and deletion controls.
- `agent-built-in-tools`: Covers the app-owned tool registry, tool-call lifecycle, allowed tools, confirmation rules, file access scope, execution logging, and tool result delivery back to Agents.
- `agent-external-cli-tools`: Covers registration, validation, permissioning, execution, confirmation, output capture, audit logging, and UI management for user-approved external CLI tools.
- `agent-rag-knowledge`: Covers per-Agent local RAG knowledge ingestion, chunking, sqlite-vec indexing, retrieval, rebuild, deletion, and prompt attachment.
- `agent-management-ui`: Covers the Agents tab and Settings/plugin management UI for selecting Agents, managing plugins, configuring local directories, and viewing local data status.

### Modified Capabilities

- None. Existing Secretary behavior is treated as source functionality to migrate into the new Agents capabilities because the Secretary specs have not yet been archived as baseline project specs.

## Impact

- Frontend: replace `SecretaryPanel` and related hooks/types with Agents-oriented components, Settings sections, plugin manager, Agent picker, identity/memory management, single-Agent chat, and multi-Agent group chat UI.
- Backend: replace `commands::secretary` and `models::secretary` with Agents commands/models while preserving LLM configuration, streaming, local persistence, and confirmed note-edit flows where applicable.
- Tools: add an Agents tool registry and executor for built-in tools such as `read_note`, `write_note`, `read_todo_list`, `add_todo_item`, `change_todo_item`, `read_milestones`, `change_milestone`, `read_file`, and `write_file`, plus user-registered external CLI tools such as `helper` and `skill`.
- Database: add or migrate tables for Agent plugins, plugin validation state, sessions, messages, session participants, user identity, app memory, conversation summaries, memory usage records, external CLI tool registrations, external CLI execution audit records, Agent private data, RAG chunks, and vector indexes.
- Filesystem: introduce configured local plugin directories, plugin package validation, avatar loading, README display, and safe install/uninstall operations.
- Dependencies: add or enable sqlite-vec integration and embedding/RAG support alongside the existing OpenAI-compatible LLM API flow.
- Migration: migrate existing Secretary persona/profile/conversation/settings data into a default built-in personal Agent where feasible; keep rollback possible by retaining old tables until migration is verified.
- First implementation slice: no external plugin install or group chat is required before the app can scan the bundled `plugins/` folder, load Personal Secretary and Confucius, and let the user chat with one selected Agent.
