# Lazy Todo App — AI Guide

<!-- maintained-by: human+ai -->

## Recommended Reading Order

1. Start with `00-overview.md` for product scope and deployment model.
2. Read `04-repo-map.md` to locate entry points and important directories.
3. Use `02-architecture.md` and `06-workflows.md` for cross-cutting behavior.
4. Check `05-data-and-api.md` before touching the database, Tauri commands, or shared types.
5. Use `10-runbook.md` and `09-testing.md` before validating changes.
6. Use `08-build.md` when the change affects releases, GitHub Actions, or docs publishing.
7. Use `12-document.md` when generating, updating, translating, or maintaining the PKB itself.

## Project Rules Worth Preserving

- Frontend-to-backend communication goes through Tauri `invoke()` only.
- Rust command handlers live in `src-tauri/src/commands/`.
- SQLite persistence is centralized in `src-tauri/src/db.rs`.
- Shared settings are persisted in SQLite and mirrored in TypeScript and Rust models.
- Bilingual docs use English source Markdown plus gettext catalogs under `locale/`.

## When Updating Docs

- Keep file references real and current.
- Treat `doc/` as the PKB root; do not introduce `man/` for this repo.
- Run `npm run pkb:check` after source, workflow, or build changes, or review the `pkb-check` GitHub Actions summary on PRs.
- Treat the PKB checker as advisory: refresh the suggested English PKB pages first, then update the Chinese gettext catalogs.
- Update the `<!-- PKB-metadata -->` footer whenever a doc changes meaningfully.
- Regenerate translation catalogs after English source changes: `poetry run make intl-update`.

---
<!-- PKB-metadata
last_updated: 2026-04-12
commit: 628f0c1
updated_by: human+ai
-->
