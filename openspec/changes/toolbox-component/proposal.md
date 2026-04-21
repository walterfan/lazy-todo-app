## Why

While working inside a Pomodoro focus session, developers routinely need small utility tools — Base64/URL/JWT decode, hash checksums, UUID/password generation, and quick AES encryption — and switching to a browser or separate app breaks flow. Folding these into a lightweight "Toolbox" tab in the lazy-todo-app keeps the user in a single window alongside Todos, Notes, and Pomodoro, removing context-switches during focus time.

## What Changes

- Add a new top-level sidebar tab **Toolbox** (🧰) positioned directly below **Pomodoro**.
- Introduce a `ToolboxPanel` React component with an internal tabbed layout for four tool groups: **Conversion**, **Checksum**, **Generation**, **Encryption**.
- Each tool group is a standalone React component under `src/components/toolbox/` offering client-side operations only — no backend Tauri commands, no persistence, no network calls.
- Conversion group ports: Base64, Hex↔ASCII, URL, HTML escape, Number base, Unix timestamp (with **seconds/milliseconds unit selector** and **batch conversion** of multiple timestamps to datetime strings), JWT decode (HS256 encode optional/stub).
- Checksum group: MD5, SHA-1, SHA-256, SHA-384, SHA-512 via Web Crypto `SubtleCrypto.digest` (MD5 via a pure-JS fallback).
- Generation group: UUID v4, random string (configurable length & charset), password (configurable complexity).
- Encryption group: AES-CBC/GCM (Web Crypto), plus classical ciphers (ROT13, Caesar, Atbash).
- Extend the `Tab` union in `App.tsx` and add the nav entry + icon + render branch.
- Add "Toolbox" to the `NoSearchOn` list (search bar hidden for this tab).
- Non-goals: saving tool inputs/outputs to SQLite, syncing across windows, per-tool settings, offline-signed JWTs with cryptographically-secure HMAC (current reference port uses a simplified signature — port will replace with Web Crypto HMAC-SHA256 for correctness).

## Capabilities

### New Capabilities
- `toolbox`: New top-level feature providing grouped utility tools (conversion, checksum, generation, encryption) accessible from the sidebar.

### Modified Capabilities
<!-- None: existing Todos, Notes, Pomodoro specs do not change. Sidebar navigation is part of shell UI, not a spec'd capability in openspec/specs/. -->

## Impact

- **New source files** (frontend only):
  - `src/components/ToolboxPanel.tsx` — container with inner tab nav.
  - `src/components/toolbox/ConversionTools.tsx`
  - `src/components/toolbox/ChecksumTools.tsx`
  - `src/components/toolbox/GenerationTools.tsx`
  - `src/components/toolbox/EncryptionTools.tsx`
  - `src/components/toolbox/ToolsHelp.tsx` (optional help panel)
  - `src/utils/crypto.ts` — Web Crypto helpers (digest, AES encrypt/decrypt, HMAC for JWT).
- **Modified files**:
  - `src/App.tsx` — extend `Tab` type, add nav item, add render branch.
  - `src/App.css` — minor styling for tabbed inner layout (no breaking changes).
- **No changes** to Rust backend (`src-tauri/`), SQLite schema, or Tauri commands — toolbox is fully client-side.
- **No new runtime dependencies**: use Web Crypto + built-in `btoa`/`atob`. A small MD5 utility (~2 KB) will be added inline rather than a new npm package.
- **Bundle size**: expected ~15–25 KB gzipped for all toolbox code.
- **Accessibility**: inner tabs must be keyboard-navigable (arrow keys + Enter); inputs/outputs must have associated `<label>`s.
