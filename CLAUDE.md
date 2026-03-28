# CLAUDE.md - Tauri Todo App with Priority & Countdown

## Project Overview
A cross-platform desktop Todo List app with priority levels and countdown timers.
Built with Tauri v2 + React + TypeScript. Backend in Rust, frontend in React.

## Tech Stack
- Tauri v2 (latest stable)
- Rust (backend commands, state management, SQLite persistence)
- React 18 + TypeScript (frontend UI)
- SQLite via rusqlite (local data storage)

## Architecture Rules
- All Tauri commands defined in `src-tauri/src/commands/`
- State management uses Tauri's managed state (`tauri::State`)
- Frontend calls backend ONLY through `@tauri-apps/api/core.invoke()`
- All data persistence goes through Rust/SQLite
- All Tauri commands return `Result<T, String>`

## Features
- Todo CRUD with title and optional description
- Priority levels: High (1), Medium (2), Low (3)
- Deadline with countdown timer display
- Sort by priority or deadline
- Persistent storage via SQLite
