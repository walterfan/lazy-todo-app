# AGENTS.md - Lazy Todo App

<!-- This file follows the open agents.md standard: https://agents.md -->

Lazy Todo App is a local-first Tauri v2 desktop productivity app built with Rust, SQLite, React, and TypeScript for todos, sticky notes, Pomodoro, toolbox utilities, and AI Agents.

> Reading guide: this file is the fast entry point for coding agents. Deeper architecture, data, workflow, build, and testing details live in `doc/`.

## 1. What This Project Is

This repository contains a cross-platform desktop app. The Rust backend owns Tauri commands, SQLite persistence, window/tray behavior, LLM calls, and local filesystem access. The React frontend owns UI state, tabs, forms, timers, streaming transcript rendering, and calls Rust only through Tauri `invoke()`.

Key facts:

| Area | Detail |
| --- | --- |
| Desktop shell | Tauri v2 |
| Backend | Rust 2021, `rusqlite`, Tauri managed state |
| Frontend | React 18, TypeScript, Vite |
| Package manager | npm, validated by `package-lock.json` |
| Task runner | `make` wraps npm and cargo commands |
| Local storage | SQLite database created in `src-tauri/src/db.rs` |
| Feature spec flow | OpenSpec changes under `openspec/changes/` |

## 2. Knowledge Base Index

Use the Project Knowledge Base in `doc/` instead of duplicating long explanations here.

| Topic | File |
| --- | --- |
| AI reading order | `doc/ai-guide.md` |
| Product overview | `doc/00-overview.md` |
| Architecture | `doc/02-architecture.md` |
| Repository map | `doc/04-repo-map.md` |
| Data and Tauri API | `doc/05-data-and-api.md` |
| Workflows | `doc/06-workflows.md` |
| Conventions | `doc/07-conventions.md` |
| Build and release | `doc/08-build.md` |
| Testing | `doc/09-testing.md` |
| Runbook | `doc/10-runbook.md` |

Recommended reading order:

1. Read this file, then `doc/ai-guide.md`.
2. For code changes, read `doc/04-repo-map.md` and `doc/05-data-and-api.md`.
3. For behavioral or architecture changes, read `doc/02-architecture.md` and the active OpenSpec change.
4. For verification, read `doc/09-testing.md` and `doc/08-build.md`.

If time is short, read `doc/ai-guide.md` plus the specific source files you will edit.

## 3. Repository Layout

```text
.
├── AGENTS.md                  # canonical coding-agent instructions
├── README.md / README_zh.md   # user-facing overview
├── Makefile                   # preferred task runner
├── package.json               # frontend scripts and Tauri CLI entry
├── src/                       # React frontend
│   ├── App.tsx                # main shell and tab routing
│   ├── components/            # UI panels and reusable components
│   ├── hooks/                 # frontend invoke/state hooks
│   ├── types/                 # TypeScript mirrors of Rust models
│   └── utils/                 # frontend-only utilities
├── src-tauri/                 # Rust backend and Tauri app
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       ├── lib.rs             # app setup, tray, command registration
│       ├── db.rs              # SQLite schema and persistence
│       ├── commands/          # Tauri command handlers
│       └── models/            # Rust request/response/domain models
├── agents/                   # bundled AI Agent package folders
├── openspec/                  # OpenSpec config, specs, and changes
├── doc/                       # bilingual PKB / Sphinx docs
├── scripts/                   # release and PKB helper scripts
└── .github/workflows/         # release, docs, and PKB advisory CI
```

Boundaries that matter:

- Public app behavior crosses the Tauri command boundary. Keep frontend calls in `@tauri-apps/api/core.invoke()` and Rust commands in `src-tauri/src/commands/`.
- SQLite schema and migrations live in `src-tauri/src/db.rs`. Do not persist app data directly from React.
- Shared structs need Rust and TypeScript mirrors, usually in `src-tauri/src/models/` and `src/types/`.
- Bundled AI Agents live in `agents/<agent_id>/` and must follow the static Agent package structure: `manifest.json`, `system_prompt.md`, `config.json`, `avatar.png`, `README.md`, and optional `rag_knowledge.md`.
- Tauri resources are configured in `src-tauri/tauri.conf.json`; update it when bundling new static resources.

Danger zones:

- `src-tauri/src/db.rs`: schema changes must be additive and safe for existing local databases.
- `src-tauri/src/lib.rs`: command registration and tray/window lifecycle affect startup.
- `src-tauri/src/commands/agents.rs` and `src-tauri/src/models/agents.rs`: Agent package validation, LLM streaming, memory, RAG, and tool boundaries are security-sensitive.
- `agents/`: Agent IDs, file names, and manifest fields are part of the local Agent package contract.
- `scripts/release_version.sh` and `.github/workflows/release.yml`: release automation can tag or publish artifacts.
- Generated output under `dist/`, `src-tauri/target/`, and `doc/_build/` should not be hand-edited.

## 4. Commands

Prefer `make` targets for day-to-day work. Use `make help` to list available targets.

```bash
make install          # npm install
make dev              # run the Tauri app in dev mode
make check            # TypeScript typecheck plus cargo check
make lint             # currently aliases frontend typecheck
make test             # Rust backend tests
make build-frontend   # tsc plus Vite production build
make build            # Tauri production desktop bundles
make pkb-check        # advisory docs freshness check
```

Focused commands:

```bash
npx tsc --noEmit
cd src-tauri && cargo check
cd src-tauri && cargo test
cd src-tauri && cargo fmt
npm run build
npm run tauri dev
```

Docs commands:

```bash
cd doc && poetry install
cd doc && poetry run make html
cd doc && poetry run make gettext
cd doc && poetry run make intl-update
```

Notes:

- Frontend has no dedicated unit test runner configured yet.
- Frontend linting currently means TypeScript checking; add ESLint/Biome before listing stricter lint commands.
- Use `LAZY_TODO_DB_DIR=/path/to/db-dir make dev` when isolating manual database tests.

## 5. Conventions

- Keep all Tauri commands in `src-tauri/src/commands/` and return `Result<T, String>`.
- Keep all persistence in Rust/SQLite through `Database`; React must not read or write local app data directly.
- Register new Rust command modules in `src-tauri/src/commands/mod.rs` and `src-tauri/src/lib.rs`.
- Mirror changed request/response shapes in `src-tauri/src/models/` and `src/types/`.
- Use `@tauri-apps/api/core.invoke()` for frontend/backend calls.
- Keep UI state in React hooks under `src/hooks/`; keep component rendering in `src/components/`.
- Use additive SQLite migrations with `CREATE TABLE IF NOT EXISTS`, `ALTER TABLE` guards, and data preservation.
- For AI Agents, treat Agent packages as static resources only. Do not add executable Agent package code.
- For LLM credentials, preserve environment-variable precedence: `LLM_BASE_URL`, `LLM_MODEL`, `LLM_API_KEY`.
- Do not add logs that expose API keys, local file contents, private notes, or user memory.

## 6. AI Agents Module Rules

The Agents module is replacing the older Secretary user experience. During the transition, some `secretary_*` tables, files, and CSS names still exist.

- Keep Agent persona and knowledge in `agents/<agent_id>/`.
- Keep app-owned identity, memory, sessions, and tool permissions in SQLite.
- Agents may read selected app context, but writes to notes, todos, milestones, files, memory, or external CLIs must go through app-owned confirmation flows.
- Streaming LLM replies should emit Tauri events and persist final messages after completion.
- RAG knowledge is per Agent. `rag_knowledge.md` must not leak between Agents.
- External CLI tool execution must use explicit registrations and direct process spawning, never shell interpolation of Agent-generated strings.

## 7. OpenSpec Workflow

Use OpenSpec for meaningful feature or architecture changes.

- Active changes live in `openspec/changes/<change-id>/`.
- Each change usually has `proposal.md`, `design.md`, `tasks.md`, and `specs/**/spec.md`.
- Implement tasks incrementally and mark each checkbox only after the code and verification are done.
- Archive completed changes with the OpenSpec archive flow instead of deleting them by hand.
- When a user invokes an OpenSpec skill from `.agents/`, `.codex/`, `.claude/`, or `.cursor/`, follow that skill file.

Useful commands:

```bash
openspec list
openspec validate <change-id> --strict
openspec instructions apply --change <change-id> --json
```

If the local `openspec` executable is not on `PATH`, inspect the existing workflow or ask before installing anything.

## 8. Working Protocol For Coding Agents

Before editing:

- Read this file and the nearest relevant source files.
- Check the active OpenSpec change when the task mentions a spec, proposal, or change ID.
- Inspect existing patterns before adding abstractions.
- Treat a dirty worktree as user-owned. Do not revert unrelated changes.

While editing:

- Keep diffs focused on the requested behavior.
- Use existing model, command, hook, and component patterns.
- Update tests when touching persistence, command contracts, prompt assembly, or user-visible workflows.
- Update docs or OpenSpec tasks when the implementation changes behavior.

Before finishing:

- Run the narrowest meaningful checks first, then broader checks when feasible.
- Report exactly what passed and what was not run.
- Call out remaining OpenSpec tasks or risk areas honestly.

## 9. Agent Client Wiring

This repo carries shared OpenSpec skill wiring for multiple coding-agent clients.

| Client | Commands | Skills / rules |
| --- | --- | --- |
| Claude Code | `.claude/commands/` | `.claude/skills/` |
| Cursor | `.cursor/commands/` | `.cursor/skills/` |
| Codex | none | `.codex/skills/` |
| Generic agents | none | `.agents/skills/` |

Keep `AGENTS.md` as the canonical root instruction file. If client-specific files diverge, migrate durable project rules back here.

## 10. When Things Go Wrong

- If `node` is missing or broken, validate with the project scripts once Node/npm are available; do not edit `package-lock.json` by hand.
- If Tauri dev startup fails, run `make doctor`, `make check`, then `cd src-tauri && cargo check`.
- If SQLite behavior changes unexpectedly, inspect `LAZY_TODO_DB_DIR` and the active app data directory before assuming data loss.
- If the frontend cannot find a command, confirm it is registered in both `src-tauri/src/commands/mod.rs` and the `generate_handler!` list in `src-tauri/src/lib.rs`.
- If Agent loading behaves strangely, validate the Agent package folder structure and `manifest.json` before changing loader code.
- If OpenSpec validation succeeds but prints telemetry/network errors, treat the validation result as authoritative and note the network warning separately.

## 11. Keeping This File Useful

Update this file when:

- A new top-level module, feature area, or app resource directory is added.
- Makefile, npm, Cargo, docs, or OpenSpec commands change.
- The Agents package contract changes.
- New persistent tables or Tauri command families are introduced.
- A new coding-agent client is wired into the repo.

Keep this file under 400 lines. Put detailed architecture and runbook material in `doc/`.

<!-- last_updated: 2026-04-29 -->
<!-- maintained-by: walter.fan -->
<!-- canonical_kb: doc/index.md -->
