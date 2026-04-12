# Lazy Todo App — Build, Release, and Publish

<!-- maintained-by: human+ai -->

## Scope

This page describes how to build the desktop app locally, how the GitHub Actions pipelines work, and how to publish both desktop release packages and the documentation site.

The relevant automation files are:

- `/.github/workflows/release.yml` for desktop release packages
- `/.github/workflows/docs.yml` for the bilingual documentation site
- `/.github/workflows/pkb-check.yml` for advisory PKB freshness checks
- `scripts/check_pkb_staleness.py` for local and CI PKB impact analysis
- `src-tauri/tauri.conf.json` for Tauri bundle settings
- `package.json` and `src-tauri/Cargo.toml` for version tracking

## Local Build Paths

### Frontend + Desktop App

Development mode:

```bash
npm install
npm run tauri dev
```

Production build:

```bash
npm run tauri build
```

Typical bundle output locations:

- macOS: `src-tauri/target/release/bundle/dmg/`
- Linux: `src-tauri/target/release/bundle/deb/` and `src-tauri/target/release/bundle/appimage/`
- Windows: `src-tauri/target/release/bundle/msi/` and `src-tauri/target/release/bundle/nsis/`

### Documentation Site

Build the bilingual docs locally:

```bash
cd doc
poetry install
poetry run make html
```

### Refresh Chinese Translation Catalogs

When English docs change, refresh the gettext catalogs before rebuilding the Chinese site:

```bash
cd doc
poetry run make gettext
poetry run make intl-update
```

Then review the affected translation files in `doc/locale/zh_CN/LC_MESSAGES/`, especially:

- `index.po` when the docs navigation changes
- `08-build.po` when this page changes
- Any other `.po` files matching the English pages you edited

Look for new untranslated entries or `#, fuzzy` markers, update the translations, and then rebuild:

```bash
cd doc
poetry run make html
```

You usually only need this refresh flow after changing the English Markdown source. If you only edit existing Chinese `.po` translations, you can skip `make gettext` and `make intl-update`.

Build the GitHub Pages staging site:

```bash
cd doc
poetry run make pages
```

Generated output:

- English HTML: `doc/_build/en/html/`
- Chinese HTML: `doc/_build/zh_CN/html/`
- Pages site root: `doc/_build/site/`

## CI/CD Pipelines

### Desktop Release Pipeline

`/.github/workflows/release.yml` publishes Tauri bundles to GitHub Releases.

Trigger conditions:

- Push a git tag matching `v*`
- Manual run via `workflow_dispatch`

Build matrix:

- macOS Apple Silicon: `aarch64-apple-darwin`
- macOS Intel: `x86_64-apple-darwin`
- Linux: `ubuntu-22.04`
- Windows: `windows-latest`

The workflow:

1. Checks out the repo
2. Installs platform-specific dependencies
3. Sets up Node.js 20 and Rust stable
4. Runs `npm ci`
5. Calls `tauri-apps/tauri-action@v0`
6. Uploads generated installers to GitHub Releases

### Documentation Publish Pipeline

`/.github/workflows/docs.yml` builds and deploys the bilingual docs to GitHub Pages.

Trigger conditions:

- Push to `main` or `master`
- The pushed commit changes `doc/**`, `README.md`, or `/.github/workflows/docs.yml`
- Manual run via `workflow_dispatch`

The workflow:

1. Checks out the repo
2. Sets up Python 3.12
3. Installs Poetry
4. Runs `poetry install` in `doc/`
5. Runs `poetry run make pages`
6. Uploads `doc/_build/site`
7. Deploys to GitHub Pages

Published docs URL:

- [https://walterfan.github.io/lazy-todo-app](https://walterfan.github.io/lazy-todo-app)

### PKB Advisory Check Pipeline

`/.github/workflows/pkb-check.yml` runs an advisory-only PKB freshness check.

Trigger conditions:

- Pull requests that touch source, workflow, or docs-related paths
- Pushes to `main` or `master`
- Manual run via `workflow_dispatch`

The workflow:

1. Checks out the repo with git history
2. Runs `python3 scripts/check_pkb_staleness.py`
3. Maps changed files to likely PKB pages
4. Reports whether those PKB pages were updated in the same diff
5. Writes a GitHub Actions job summary with suggested next steps

This workflow is intentionally non-blocking. It warns about likely stale PKB docs but does not fail the build just because docs were not updated yet.

### Local PKB Advisory Check

Run the same advisory analysis locally:

```bash
npm run pkb:check
```

Use this when:

- a feature changed but you are not sure which PKB pages to refresh
- a PR changes commands, workflows, tray behavior, settings, or build files
- you want to review likely doc impact before pushing

## Release Process

### Step 1: Update Version Fields

Before creating a new release, keep the version aligned in these files:

- `package.json`
- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.toml`

Example: change all three from `0.1.0` to `0.1.1`.

Release helper script:

```bash
./scripts/release_version.sh v0.1.1
```

Or through npm:

```bash
npm run release:tag -- v0.1.1
```

This helper:

1. Uses `sed` to update the version in `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml`
2. Creates a commit like `release: v0.1.1`
3. Pushes the current branch
4. Creates the git tag `v0.1.1`
5. Pushes the tag to `origin`

For safety, the script requires a clean working tree before it runs.

### Step 2: Validate Locally

Recommended checks before tagging:

```bash
npm install
npx tsc --noEmit
cd src-tauri && cargo check
cd ..
npm run tauri build
```

If you changed docs, also run:

```bash
cd doc
poetry run make html
```

### Step 3: Commit and Push

Example:

```bash
git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml
git commit -m "release: v0.1.1"
git push origin main
```

If you use `./scripts/release_version.sh v0.1.1`, this step is handled automatically.

### Step 4: Create and Push the Release Tag

The release workflow does not trigger from a normal branch push. It triggers from a tag push such as `v0.1.1`.

```bash
git tag v0.1.1
git push origin v0.1.1
```

If you use `./scripts/release_version.sh v0.1.1`, this step is also handled automatically.

### Step 5: Verify the GitHub Release

After the tag push:

1. Open the GitHub repository
2. Go to `Actions`
3. Open the `publish` workflow run
4. Wait for all matrix jobs to finish
5. Open `Releases` and confirm that the new assets are attached

## Publishing Documentation

### Automatic Publish

The docs site is published automatically when documentation-related files are pushed to `main` or `master`.

Typical flow:

```bash
git add doc README.md
git commit -m "docs: update build guide"
git push origin main
```

Then verify:

1. Open GitHub `Actions`
2. Inspect the `docs` workflow run
3. Open the GitHub Pages URL after deployment finishes

### Manual Publish

Both workflows support manual execution from the GitHub UI:

1. Open the repository on GitHub
2. Go to `Actions`
3. Select `publish` or `docs`
4. Click `Run workflow`

Manual execution is useful when:

- A previous run failed due to transient CI issues
- You want to rebuild without creating a new commit
- You want to confirm a workflow fix before the next full release

## Re-running Failed Builds

### Re-run a Failed Release Job

If the source code and tag are already correct, you do not need a new commit just to retry CI.

Use GitHub:

1. Open the failed workflow run in `Actions`
2. Click `Re-run failed jobs` or `Re-run all jobs`

### When You Do Need a New Release Tag

Create a new version and a new tag when:

- The code or build configuration changed
- A packaging asset was missing
- Version fields were wrong
- The previous release artifacts should not be reused

In that case:

1. Fix the code or config
2. Bump the version in the tracked version files
3. Commit and push
4. Push a new tag such as `v0.1.2`

## Common Build and Publish Failures

| Symptom | Likely Cause | Fix |
|---------|--------------|-----|
| Windows bundle fails with `Couldn't find a .ico icon` | `src-tauri/tauri.conf.json` does not list `icons/icon.ico` | Add `"icons/icon.ico"` to `bundle.icon` and re-run release |
| Release workflow did not start | Only a branch was pushed, no `v*` tag was pushed | Create and push a release tag such as `v0.1.1` |
| Release has the wrong version name | Version fields are inconsistent across config files | Keep `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml` aligned |
| Docs workflow did not start | The commit did not change tracked docs paths | Push changes to `doc/**`, `README.md`, or run the workflow manually |
| PKB advisory workflow flags likely stale docs | Source changed but corresponding PKB pages were not updated | Run `npm run pkb:check`, refresh the suggested English PKB pages, then rebuild docs |
| GitHub Pages deploy succeeds but site is unavailable | GitHub Pages source is not set to Actions | In repo settings, set `Pages -> Source` to `GitHub Actions` |
| Docs build fails after adding Mermaid diagrams | Mermaid source contains parser-sensitive labels | Prefer quoted labels and avoid raw query strings or `{}` in flowchart node text |

## Suggested Release Checklist

Use this checklist before publishing:

1. Version fields updated in `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml`
2. Desktop app builds locally with `npm run tauri build`
3. Docs build locally with `poetry run make html` when docs changed
4. Release notes or README updates are committed if needed
5. Main branch is pushed
6. Release tag is pushed
7. GitHub `publish` workflow finishes successfully
8. GitHub Release assets can be downloaded
9. GitHub Pages site loads correctly after docs updates

---
<!-- PKB-metadata
last_updated: 2026-04-12
commit: a34edf3
updated_by: human+ai
-->
