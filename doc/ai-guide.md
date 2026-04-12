# Lazy Todo App — AI Guide

<!-- maintained-by: human+ai -->

## Recommended Reading Order

1. Start with `00-overview.md` for product scope and deployment model.
2. Read `01-repo-map.md` to locate entry points and important directories.
3. Use `02-architecture.md` and `03-workflows.md` for cross-cutting behavior.
4. Check `04-data-and-api.md` before touching the database, Tauri commands, or shared types.
5. Use `06-runbook.md` and `07-testing.md` before validating changes.

## Project Rules Worth Preserving

- Frontend-to-backend communication goes through Tauri `invoke()` only.
- Rust command handlers live in `src-tauri/src/commands/`.
- SQLite persistence is centralized in `src-tauri/src/db.rs`.
- Shared settings are persisted in SQLite and mirrored in TypeScript and Rust models.
- Bilingual docs use English source Markdown plus gettext catalogs under `locale/`.

## When Updating Docs

- Keep file references real and current.
- Treat `doc/` as the PKB root; do not introduce `man/` for this repo.
- Update the `<!-- PKB-metadata -->` footer whenever a doc changes meaningfully.
- Regenerate translation catalogs after English source changes: `poetry run make intl-update`.

---
<!-- PKB-metadata
last_updated: 2026-04-12
commit: f9ba186
updated_by: human+ai
-->
