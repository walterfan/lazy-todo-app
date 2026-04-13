# Appendix A — Frequently Asked Questions (FAQ)

<!-- maintained-by: human+ai -->

## General

### Q: What is Lazy Todo App?

A desktop to-do / task management application built with **Tauri 2 + React + TypeScript**. It runs natively on macOS, Windows, and Linux with a small footprint and offline-first SQLite storage.

### Q: Why "Lazy"?

The app is designed to reduce friction — quick capture, minimal clicks, smart defaults. The goal is to let you manage tasks *lazily* (effortlessly), not to encourage laziness.

### Q: Is it free / open-source?

Yes. The project is hosted on GitHub under an open-source license. See the repository root for the exact license file.

---

## Installation & Setup

### Q: What are the minimum system requirements?

| Platform | Requirement |
|----------|-------------|
| macOS    | 10.15 (Catalina) or later, Apple Silicon or Intel |
| Windows  | Windows 10 (1803+) or later |
| Linux    | Ubuntu 20.04+ / Fedora 36+ or equivalent with WebKitGTK 4.1 |

### Q: How do I install the app?

Download the latest release from the GitHub Releases page:

- **macOS**: `.dmg` file — open and drag to Applications
- **Windows**: `.msi` or `.exe` (NSIS) installer
- **Linux**: `.deb` or `.AppImage`

### Q: How do I set up the development environment?

See [01-quick-start.md](01-quick-start.md) for a step-by-step guide.

### Q: `npm run tauri dev` fails with missing system dependencies on Linux

Install the required WebKitGTK and other system libraries:

```bash
sudo apt update
sudo apt install -y libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```

See the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for the full list.

### Q: Rust compilation is very slow on the first build

This is expected. The first `cargo build` downloads and compiles all crate dependencies. Subsequent builds use the cache and are much faster. You can also use `sccache` to speed up repeated clean builds.

---

## Development

### Q: Which package manager should I use?

**pnpm** is the project standard. The lock file (`pnpm-lock.yaml`) is committed to the repo. Do not mix with `npm` or `yarn` to avoid lock file conflicts.

### Q: How do I add a new frontend dependency?

```bash
pnpm add <package-name>
```

For dev-only dependencies:

```bash
pnpm add -D <package-name>
```

### Q: How do I add a Rust (backend) dependency?

Edit `src-tauri/Cargo.toml` or use:

```bash
cd src-tauri
cargo add <crate-name>
```

### Q: How do I create a new Tauri command?

1. Define the Rust function in `src-tauri/src/` with the `#[tauri::command]` attribute
2. Register it in the Tauri builder (usually in `main.rs` or `lib.rs`)
3. Call it from the frontend via `import { invoke } from '@tauri-apps/api/core'`

### Q: Hot-reload is not working

- Make sure you are running `pnpm tauri dev` (not `pnpm dev` alone — that only starts Vite without the Tauri shell)
- Frontend (React) changes hot-reload automatically via Vite HMR
- Rust changes require a recompile; Tauri CLI handles this but it takes a few seconds

---

## Data & Storage

### Q: Where is my data stored?

The app uses SQLite. The database file is stored in the platform-specific app data directory:

| Platform | Path |
|----------|------|
| macOS    | `~/Library/Application Support/com.lazy-todo-app/` |
| Windows  | `%APPDATA%\com.lazy-todo-app\` |
| Linux    | `~/.local/share/com.lazy-todo-app/` |

### Q: Can I back up my data?

Yes. Simply copy the SQLite database file from the path above. You can also export tasks via the app's export feature (if available).

### Q: Is my data synced to the cloud?

No. All data is stored locally. This is by design for privacy and offline-first usage.

---

## Build & Release

### Q: How do I create a release?

See the **Release Process** section in [08-build.md](08-build.md). In short:

```bash
./scripts/release_version.sh v0.x.x
```

### Q: The CI build failed — do I need a new tag?

Not necessarily. If the code is correct and it was a transient CI failure, you can re-run the failed jobs from the GitHub Actions UI. See [08-build.md](08-build.md) for details.

### Q: How do I build for a different platform?

Cross-compilation for Tauri desktop apps is not straightforward. The recommended approach is to build on the target platform (or use CI with a build matrix). The GitHub Actions release workflow already covers macOS (ARM + Intel), Linux, and Windows.

---

## Troubleshooting

### Q: The app window is blank after launch

- Check the WebView console (right-click → Inspect in dev mode)
- Ensure the frontend was built: `pnpm build`
- On Linux, verify WebKitGTK is installed

### Q: Database migration errors on app update

The app runs migrations automatically on startup. If a migration fails:

1. Check the app logs (see [11-observability.md](11-observability.md))
2. Back up your database file
3. Report the issue on GitHub with the error message

### Q: How do I reset the app to a clean state?

Delete the app data directory (see "Where is my data stored?" above). **Warning**: this removes all your tasks.
