## Context

`lazy-todo-app` is a Tauri v2 + React 18 + TypeScript desktop app with Rust/SQLite backend. The sidebar currently exposes three primary tabs — Todos, Notes, Pomodoro — plus Settings and Quit. Users working in focus sessions frequently interrupt their flow to open a browser for small utility tasks (Base64 decode, UUID generation, hash verification, JWT inspection). A reference Vue 3 implementation exists at `lazy-ai-coder/web/src/views/EncodingToolsView.vue` which we will port to React idioms.

Architectural constraint (from `CLAUDE.md`): all data persistence goes through Rust/SQLite; frontend calls backend only via `@tauri-apps/api/core.invoke()`. The Toolbox is fully stateless and **client-side only**, so this change does **not** require new Tauri commands, new SQLite tables, or any Rust code. All crypto uses the browser-embedded Web Crypto API (Tauri v2 WebView provides it).

## Goals / Non-Goals

**Goals:**
- Zero backend changes; frontend-only feature.
- Port the four Vue tool groups (Conversion, Checksum, Generation, Encryption) into React components following existing patterns (`PomodoroPanel.tsx`, `SettingsPanel.tsx`).
- Single new top-level nav entry in the sidebar below Pomodoro.
- All cryptographic primitives use Web Crypto (`crypto.subtle`); no new npm dependencies.
- JWT encoding produces a **correct HMAC-SHA256 signature** (reference Vue code has a non-cryptographic simplified hash; we will fix this during the port).
- Accessibility: keyboard-navigable inner tabs, labeled inputs, copy-to-clipboard feedback.

**Non-Goals:**
- Persisting tool inputs/outputs to SQLite or localStorage.
- Cross-window sync of toolbox state (the Tauri sticky-notes child window model does not apply here).
- Full RSA/ECDSA JWT signing (HS256 only for v1).
- File-based hashing (text-input only for v1; file upload is a v2 follow-up).
- Supporting legacy classical ciphers beyond ROT13/Caesar/Atbash (e.g., Vigenère) in v1.

## Decisions

### Decision 1: Folder layout — `src/components/toolbox/` (new subfolder)
**Chosen**: Group per-tool components under a new `src/components/toolbox/` subfolder, with `ToolboxPanel.tsx` as the container at the top of `src/components/` (mirroring `PomodoroPanel.tsx`).
**Alternatives considered**:
- Flat `src/components/` (rejected — adds 5+ files to an already-flat directory; hurts discoverability).
- `src/features/toolbox/` (rejected — project has no `features/` convention; inconsistent with existing code).

### Decision 2: Inner navigation — vertical tabs vs. dropdown
**Chosen**: Inner horizontal tab bar inside `ToolboxPanel` with 4 group tabs (Conversion / Checksum / Generation / Encryption) plus optional Help. Each group renders a dropdown `<select>` (or small pill bar) to switch between tools within the group — matching the Vue reference UX.
**Alternatives considered**:
- Flat list of ~15 tools in a single select (rejected — poor discovery).
- Accordion (rejected — occupies too much vertical space in our compact layout).

### Decision 3: Crypto primitives — Web Crypto only
**Chosen**: Use `crypto.subtle.digest` for SHA-1/256/384/512 and `crypto.subtle.encrypt/decrypt` with `AES-CBC` / `AES-GCM`. MD5 is intentionally absent from Web Crypto; we inline a ~50-line pure-JS MD5 implementation (or use `js-md5` if the user prefers a dep — defer to implementation).
**Alternatives considered**:
- `crypto-js` (rejected — ~40 KB, unnecessary given Web Crypto covers 80% of needs).
- Delegating crypto to Rust backend (rejected — violates client-side constraint and adds IPC latency for interactive tools).
**Risk mitigation**: JWT HS256 uses `crypto.subtle.sign({name:'HMAC'}, ...)` — cryptographically correct, unlike the Vue reference's simplified hash.

### Decision 4: AES mode support
**Chosen**: Support `AES-CBC` and `AES-GCM` only in v1 (both in Web Crypto). Drop `AES-CFB`, `AES-ECB`, `AES-OFB` from the Vue reference since Web Crypto does not support them and adding a JS-only implementation is out of scope.
**Alternatives considered**:
- Pull in `crypto-js` to match the Vue menu (rejected — bundle cost).
- Hide those options entirely vs. show them disabled with tooltip (chosen: **hide** them in v1 to avoid confusion).

### Decision 5: State management — local `useState` per component
**Chosen**: Each tool group manages its own input/output state via `useState`. No context, no Zustand/Redux, no hook layer. Matches existing patterns (`AddTodo.tsx`, `NoteEditor.tsx`).
**Alternatives considered**:
- A shared `useToolbox` hook (rejected — no cross-tool data sharing needed).
- Persisting last-used values (rejected — explicit non-goal; users may paste sensitive data).

### Decision 6: Sidebar placement
**Chosen**: Extend the `Tab` type to `"todos" | "notes" | "pomodoro" | "toolbox" | "settings"` and append `{ key: "toolbox", icon: "🧰", label: "Toolbox" }` to `NAV_ITEMS` after the Pomodoro entry.
**Alternatives considered**:
- Nest Toolbox as a sub-tab inside Pomodoro (user-rejected in the proposal dialog).
- Put it under Settings (rejected — tools aren't configuration).

### Decision 7: Search bar behavior
**Chosen**: Toolbox tab hides the header search bar (same as Pomodoro/Settings today). The condition `(activeTab === "todos" || activeTab === "notes")` already handles this correctly — no additional work needed.

## Risks / Trade-offs

- **Risk**: Web Crypto operations are async (`Promise`-based), unlike the synchronous Vue reference. → **Mitigation**: wrap tool actions in `async` handlers and display a brief "Computing…" state for hashes on large inputs.
- **Risk**: Clipboard write (`navigator.clipboard.writeText`) can fail in unusual Tauri webview states. → **Mitigation**: wrap in try/catch, fall back to selecting the output textarea so the user can Cmd+C manually; show a small toast.
- **Risk**: Pasting very large text (multi-MB) into hashing or encryption can freeze the UI. → **Mitigation**: hard-cap input at 1 MB with a friendly warning; hashing is fast on Web Crypto so 1 MB is well under 50 ms.
- **Risk**: MD5 is cryptographically broken — users may misuse it. → **Mitigation**: show a small "MD5 is insecure; use SHA-256 for security" hint next to the MD5 option.
- **Trade-off**: Dropping AES-ECB/CFB/OFB reduces feature parity with the Vue reference but removes the need for a bulky JS crypto dep. Acceptable because GCM covers the modern-security case and CBC covers the interop case.
- **Trade-off**: No persistence means users lose their input on tab switch. Accepted because tools are intended for quick throw-away operations; persistence can be added later if requested.

## Migration Plan

No migration required. This is a purely additive, frontend-only change.
- **Deploy**: standard `npm run tauri build`; users on the next release see the new tab automatically.
- **Rollback**: revert the PR; the Toolbox tab disappears. No data cleanup needed (nothing is written to SQLite).

## Open Questions

- Should the Help tab (from the Vue reference `ToolsHelp.vue`) be ported in v1, or deferred? **Proposed**: include a minimal Help panel listing what each group does; defer deep docs to v2.
- Inline MD5 vs. adding `js-md5` as an npm dep? **Proposed**: inline ~50 LOC MD5 to preserve "no new deps" promise; reassess if code gets messy.
