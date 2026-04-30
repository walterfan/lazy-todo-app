## Context

Lazy Todo App is a Tauri v2 desktop app with React/TypeScript UI, Rust commands, managed SQLite state, and tab-based navigation in `App.tsx`. The app already persists productivity data in `todos.db` and exposes backend operations through `invoke()`.

The Secretary module adds an external LLM integration, local prompt/profile management, persona continuity, memory, app-context reading, follow-up handling, reminder management, skill loading from disk, and conversation persistence. This is cross-cutting because it touches frontend UI, Rust commands, SQLite schema, filesystem access, environment variables, local privacy controls, desktop notifications, and outbound HTTP calls.

## Goals / Non-Goals

**Goals:**
- Add a Secretary tab where users can configure and talk to an OpenAI-compatible LLM endpoint
- Resolve `LLM_BASE_URL`, `LLM_MODEL`, and `LLM_API_KEY` from environment variables first, then saved configuration
- Let users select a role, domain, and related skills before starting a discussion
- Let users give the secretary a persistent persona: name, voice, values, interaction style, and boundaries
- Add local, user-controlled memory for durable preferences, project facts, decisions, and recurring ideas
- Let the secretary read selected Todo, Pomodoro milestone, and Sticky Note context
- Let the secretary propose Sticky Note edits and apply them after explicit user confirmation
- Let the secretary ask clarifying questions, give suggestions, identify follow-ups, and propose reminders
- Stream LLM replies into the transcript while the secretary is answering
- Provide built-in roles for domain answering, domain questioning, idea criticism, and idea generation
- Load skills from a user-selected folder and make them selectable in secretary profiles
- Save conversation transcripts into a user-selected folder
- Persist non-secret settings and metadata in SQLite

**Non-Goals:**
- Building a multi-provider abstraction beyond OpenAI-compatible chat completions in v1
- Cloud synchronization of profiles, skills, or conversations
- Fine-grained agent tool execution or autonomous task execution
- Editing skill files from inside the app
- Fully automatic memory creation without user review
- Automatically mutating Todos, milestones, or notes from secretary suggestions without confirmation
- Mutating Todos or Pomodoro milestones from secretary suggestions in v1
- Semantic/vector search in v1; deterministic keyword/domain/profile filtering is sufficient to start

## Decisions

### 1. LLM calls run through Rust commands

**Decision**: The frontend SHALL call Rust secretary commands via `invoke()`. Rust builds the prompt, attaches credentials, sends the HTTP request, streams assistant chunks through Tauri events, persists the final result, and returns the secretary message.

**Rationale**: This follows the app rule that frontend talks to backend only through Tauri commands. It also keeps API keys out of browser-facing request code and centralizes environment-variable precedence.

**Alternatives considered:**
- Frontend `fetch()` directly to the LLM: simpler UI code, but exposes secrets and violates the project architecture.
- Shelling out to a CLI LLM client: avoids HTTP implementation, but adds an operational dependency and weaker error handling.

### 1a. Streaming replies use Tauri events

**Decision**: `send_secretary_message_stream` SHALL request OpenAI-compatible streaming chat completions and emit `secretary-stream-chunk`, `secretary-stream-error`, and `secretary-stream-finish` events keyed by a frontend-generated stream ID. The hook keeps transient pending user text and streaming assistant text until the final persisted conversation arrives.

**Rationale**: Streaming gives the secretary a more present, responsive feel while preserving the architecture rule that the frontend talks to backend commands. Stream IDs prevent stale chunks from an older request from updating the current transcript.

**Alternatives considered:**
- Return only the final response: simpler, but makes longer secretary replies feel inert.
- Use a separate websocket server: flexible, but unnecessary inside a Tauri app because command plus event flow covers this use case.

### 2. Environment variables override saved configuration

**Decision**: Resolve effective LLM settings in this order: `LLM_BASE_URL`, `LLM_MODEL`, `LLM_API_KEY` from process environment first; saved SQLite configuration second. The UI can save fallback values, but it must display when an environment value is active.

**Rationale**: Environment variables are the safest and most portable way to inject secrets into desktop app runtime. Saved configuration keeps local development convenient when the user chooses it.

**Alternatives considered:**
- Save every setting, including API key, in SQLite only: convenient but weak for secrets.
- Environment only: secure but unfriendly for users who expect desktop settings.

### 3. Store secretary metadata, memory, and reminders in SQLite; transcripts on disk

**Decision**: Use SQLite for settings, personas, profiles, skill metadata, memory entries, reminder entries, and conversation indexes. Save full conversation transcripts as Markdown plus a machine-readable JSON sidecar in the configured conversation folder.

**Rationale**: SQLite is already used by the app and is good for indexes, memory lifecycle state, and UI lists. Files are better for conversation artifacts because users can inspect, version, move, or share them outside the app.

**Alternatives considered:**
- Store all messages and memories only in SQLite: easier querying, but less portable for saved discussions.
- Store everything only as files: portable, but harder to list, search, filter memories, and resume reliably.

### 4. Skill loading is read-only and file-based

**Decision**: A skill folder scan reads supported text files, extracts name/description/content, records metadata, and makes selected skills available to profiles. The app does not modify skill files.

**Rationale**: The user asked to load skills from a specified folder. Read-only loading keeps the feature predictable and avoids ownership problems across arbitrary folders.

**Alternatives considered:**
- Import skills into the database: better portability, but duplicates source files and makes updates confusing.
- Support arbitrary binary/doc formats: broader, but risky and unnecessary for v1.

### 5. Profile prompt assembly is deterministic and layered

**Decision**: The backend constructs a system prompt from persona, built-in role instructions, selected domain, selected skill contents, relevant memory entries, and selected app context in a stable order. The conversation history is appended after that prompt.

**Rationale**: Deterministic prompt assembly makes behavior testable and makes saved conversations easier to reproduce. The persona layer provides "soul" and continuity, while memory and app context provide useful working knowledge without hiding what was sent to the model.

**Alternatives considered:**
- Let the frontend assemble prompts: easier to iterate visually, but duplicates backend validation and can leak secrets/skill content handling.
- Free-form custom prompts only: flexible, but misses the persona/role/domain/skill workflow that is the key product feature.

### 6. Memory is local, inspectable, and user-controlled

**Decision**: Memory entries are explicit records with content, scope, status, source conversation, timestamps, and optional tags. Memories can be global, domain-scoped, or profile-scoped. The secretary may propose memories, but the user approves them before storage. Users can edit, pin, archive, or delete memories.

**Rationale**: Memory makes the secretary feel continuous, but uncontrolled memory can feel opaque or invasive. Local, visible memory gives the app continuity while preserving user trust.

**Alternatives considered:**
- Automatically store inferred memories without confirmation: more magical, but too opaque for a local productivity app.
- Store only whole conversation history as memory: easy, but noisy and expensive to include in prompts.
- Vector embeddings for memory retrieval in v1: powerful, but adds dependency and migration complexity before the core experience is proven.

### 7. Persona is separate from role

**Decision**: Persona describes how the secretary shows up: name, voice, values, interaction style, and boundaries. Role describes the job it is doing in a domain: answering, asking, criticizing, or raising ideas.

**Rationale**: Separating persona from role lets the same secretary identity shift tasks without losing continuity. A thoughtful personal secretary can be an interviewer in one conversation and a critic in another while still feeling like the same presence.

**Alternatives considered:**
- Combine persona and role into one profile field: fewer controls, but less reusable and muddier behavior.
- Hard-code one secretary personality: coherent, but does not honor users who want a different style.

### 8. Reminders require user confirmation

**Decision**: The secretary can identify follow-ups and propose reminders, but reminders are only saved after the user confirms title, notes, and due time. Reminders are stored locally with status and source conversation metadata.

**Rationale**: A personal secretary should help the user remember, but should not silently create obligations. Confirmation keeps the user in control and makes reminder data trustworthy.

**Alternatives considered:**
- Automatically create reminders from every inferred commitment: efficient, but noisy and risky.
- Only allow manual reminders: safe, but misses the secretary value of noticing follow-ups in conversation.

### 9. App context is explicit, with confirmed note edits

**Decision**: The secretary can read Todo items, Pomodoro milestones, and Sticky Notes through backend context aggregation commands. Context inclusion is controlled by the user, can be source-level or item-level, and the response records which context items were used. The secretary may propose Sticky Note edits with a before/after preview, but the app only applies the edit after the user confirms. Todos and milestones remain read-only in v1.

**Rationale**: The secretary becomes useful when it can see the user's actual work and help maintain notes, but it must remain trustworthy. Explicit context and confirmed note edits let it suggest priorities, ask informed questions, improve notes, and propose reminders without surprising the user.

**Alternatives considered:**
- Always include all Todos, milestones, and notes: convenient, but creates privacy and prompt-size problems.
- Let the frontend assemble app context: simpler at first, but duplicates filtering and makes prompt assembly less consistent.
- Allow unconfirmed secretary edits to notes: fast, but too easy to lose user-authored content.
- Allow direct secretary edits to Todos and milestones in v1: powerful, but requires a broader confirmation and audit design.

### 10. Note edits reuse the existing note update path

**Decision**: Confirmed secretary note edits call the existing note update command/database path with the target note ID and changed title/content/color fields. The conversation records confirmation metadata and the updated note metadata.

**Rationale**: Reusing the existing note update path keeps persistence behavior consistent with the Notes module. The Secretary layer owns proposal, preview, and confirmation; the Notes layer owns the actual mutation.

**Alternatives considered:**
- Add separate secretary-only note mutation tables: useful for audit, but duplicates note persistence.
- Have the LLM write directly to note storage: unsafe and violates the Tauri command architecture.

## Risks / Trade-offs

- **[Risk] API key exposure through UI or logs** -> Mitigation: mask secrets, never log request headers, and prefer environment variables over saved values.
- **[Risk] Large skill folders or huge skill files slow the app** -> Mitigation: cap file size, skip unsupported files, and report skipped files in scan results.
- **[Risk] LLM endpoint compatibility varies** -> Mitigation: target OpenAI-compatible `/chat/completions` request/response shape and return clear errors for unsupported responses.
- **[Risk] Streaming endpoint compatibility varies** -> Mitigation: parse standard OpenAI-compatible SSE `data:` chunks, emit clear stream errors, and keep non-stream command behavior available internally.
- **[Risk] Conversation save folder may become unavailable** -> Mitigation: validate folder access before saving and keep unsaved transcript state in the UI until the user retries.
- **[Risk] Persona instructions could make responses theatrical or distracting** -> Mitigation: keep persona controls concise and include practical boundaries in the default persona.
- **[Risk] Memory may include sensitive content** -> Mitigation: require user approval for proposed memories, expose all memories in a management panel, and support delete/archive.
- **[Risk] Memory context can bloat prompts** -> Mitigation: include pinned and highly relevant memories only, with a limit on count and total characters.
- **[Risk] Todo/note context can expose too much data to the LLM** -> Mitigation: keep context source toggles, item selection, and used-context indicators visible in the Secretary UI.
- **[Risk] Large notes can bloat prompts** -> Mitigation: cap included note content and prefer selected or relevant notes.
- **[Risk] Secretary note edits could overwrite useful content** -> Mitigation: require explicit confirmation with before/after preview and record edit metadata in the conversation.
- **[Risk] Reminder suggestions can become noisy** -> Mitigation: require confirmation and provide dismiss/snooze/complete controls.
- **[Trade-off] Streaming adds transient UI state** -> The hook tracks pending user text, streamed assistant text, and stream IDs separately from persisted conversations so failed or stale streams do not corrupt saved transcripts.

## Migration Plan

1. Add new SQLite tables with `CREATE TABLE IF NOT EXISTS` so existing databases migrate in place.
2. Register new secretary models and commands without changing existing todo, note, pomodoro, toolbox, or settings commands.
3. Seed a default persona if no persona exists.
4. Add the Secretary tab behind normal app navigation.
5. Rollback by removing the Secretary tab and commands; unused secretary tables can remain harmlessly in SQLite.

## Open Questions

- Whether saved API keys should be allowed at all or replaced with OS keychain storage in a later iteration.
- Whether conversation resume should read full JSON transcripts from disk or persist message rows in SQLite as well.
- Whether v2 memory retrieval should add embeddings/vector search after the local memory workflow proves useful.
- Whether v2 should allow confirmed secretary edits to Todos and milestones with an audit trail.
