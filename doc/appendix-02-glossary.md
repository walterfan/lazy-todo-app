# Appendix B — Glossary

<!-- maintained-by: human+ai -->

| Term | Definition |
|------|-----------|
| **AppImage** | A portable Linux application format that runs without installation. The app ships as a single executable file. |
| **Cargo** | The Rust package manager and build system. Manages dependencies declared in `Cargo.toml` and compiles Rust code. |
| **CI / CD** | **Continuous Integration / Continuous Delivery**. Automated pipelines that build, test, and release software on every code change. This project uses GitHub Actions. |
| **Crate** | A Rust package or library. External crates are pulled from [crates.io](https://crates.io). |
| **DMG** | Apple Disk Image — the standard macOS distribution format for desktop applications. |
| **ESLint** | A JavaScript/TypeScript linter used to enforce code style and catch errors statically. |
| **GitHub Actions** | GitHub's built-in CI/CD platform. Workflows are defined in `.github/workflows/*.yml`. |
| **HMR** | **Hot Module Replacement**. A Vite feature that updates the browser in real-time when source files change, without a full page reload. |
| **IPC** | **Inter-Process Communication**. In Tauri, the mechanism by which the frontend (WebView) calls Rust backend functions via `invoke()`. |
| **MSI** | Microsoft Installer — a Windows installer package format. |
| **NSIS** | **Nullsoft Scriptable Install System**. An alternative Windows installer format supported by Tauri, producing `.exe` installers. |
| **Offline-first** | A design philosophy where the app works fully without network connectivity. Data is stored locally and optionally synced later. |
| **pnpm** | A fast, disk-efficient Node.js package manager. The project standard; uses `pnpm-lock.yaml` as the lock file. |
| **React** | A JavaScript library for building user interfaces using a component-based architecture. The frontend framework of this project. |
| **Rust** | A systems programming language focused on safety, speed, and concurrency. Powers the Tauri backend of this app. |
| **sccache** | A compiler cache for Rust (and C/C++). Speeds up repeated builds by caching compilation artifacts. |
| **SQLite** | A lightweight, file-based relational database engine. Used as the local data store for tasks and settings. |
| **Tauri** | An open-source framework for building lightweight desktop applications with a web frontend and a Rust backend. This project uses **Tauri v2**. |
| **Tauri Command** | A Rust function annotated with `#[tauri::command]` that can be called from the frontend via `invoke()`. |
| **TypeScript** | A typed superset of JavaScript that compiles to plain JS. Used for all frontend source code in this project. |
| **Vite** | A fast frontend build tool that provides instant dev server startup and HMR. Used as the bundler for the React frontend. |
| **WebKitGTK** | The Linux WebView engine used by Tauri to render the frontend UI. Must be installed as a system dependency on Linux. |
| **WebView** | An embedded browser component that renders the frontend HTML/CSS/JS inside the native Tauri window. |

---

## Project-Specific Terms

| Term | Definition |
|------|-----------|
| **Lazy Todo** | The application name. "Lazy" refers to the low-friction, minimal-effort UX philosophy. |
| **Task** | The core data entity — a to-do item with properties like title, description, priority, status, due date, etc. |
| **Quick Capture** | A UX pattern allowing users to create a task with minimal input (e.g., just a title), with smart defaults filling in the rest. |
| **Migration** | A versioned SQL script that evolves the SQLite database schema. Migrations run automatically on app startup. |
| **Release Script** | `scripts/release_version.sh` — automates version bumping, tagging, and triggering the CI release pipeline. |

---

> 💡 For architecture-level concepts, see [02-architecture.md](02-architecture.md).  
> For API and data model details, see [05-data-and-api.md](05-data-and-api.md).
