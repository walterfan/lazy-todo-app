## 1. Scaffolding & Navigation

- [x] 1.1 Extend the `Tab` union in `src/App.tsx` to include `"toolbox"`.
- [x] 1.2 Add `{ key: "toolbox", icon: "🧰", label: "Toolbox" }` to `NAV_ITEMS` after the Pomodoro entry.
- [x] 1.3 Add a render branch in `<main className="content-main">` that mounts `<ToolboxPanel />` when `activeTab === "toolbox"` (use the same `display: none` trick as Pomodoro so internal tab state persists across sidebar switches).
- [x] 1.4 Confirm the header search bar is hidden on the Toolbox tab (existing condition already excludes non-`todos`/`notes` tabs — verify no extra work needed).
- [x] 1.5 Create the `src/components/toolbox/` folder and add an empty `ToolboxPanel.tsx` with a stub return so the app still builds.

## 2. Shared Utilities

- [x] 2.1 Create `src/utils/crypto.ts` exporting `sha(algorithm, text)` wrapping `crypto.subtle.digest` for SHA-1/256/384/512 with hex encoding.
- [x] 2.2 Add `md5(text)` in the same file using an inlined pure-JS MD5 implementation (~50 LOC; no new npm dep).
- [x] 2.3 Add `hmacSha256(key, data)` returning a Base64url string, built on `crypto.subtle.sign({name:'HMAC', hash:'SHA-256'}, ...)`.
- [x] 2.4 Add `aesEncrypt(mode, keyHex, ivHex, plaintext)` and `aesDecrypt(...)` helpers supporting AES-CBC and AES-GCM with 128/192/256-bit keys.
- [x] 2.5 Add a `copyText(text)` helper that wraps `navigator.clipboard.writeText` with a try/catch fallback and returns a success boolean.
- [x] 2.6 Add a `MAX_INPUT_BYTES = 1_048_576` constant and an `assertInputSize(text)` guard used by all tool action handlers.

## 3. Toolbox Container

- [x] 3.1 Implement `ToolboxPanel.tsx` with four tab buttons (Conversion, Checksum, Generation, Encryption) and optional Help tab.
- [x] 3.2 Use local `useState<'conversion' | 'checksum' | 'generation' | 'encryption' | 'help'>` for the inner active tab; default to `'conversion'`.
- [x] 3.3 Render the four tool-group components using a `display: none` toggle (not conditional unmount) so input state within each group persists across inner-tab switches.
- [x] 3.4 Implement keyboard navigation on the inner tab bar (ArrowLeft/ArrowRight moves focus + activates; Home/End jumps to first/last).
- [x] 3.5 Add CSS for the inner tab bar in `src/App.css` (or a co-located `ToolboxPanel.css`) matching the existing visual style.

## 4. Conversion Tools Component

- [x] 4.1 Create `src/components/toolbox/ConversionTools.tsx` with a tool selector (`<select>`) for: Base64, Hex-ASCII, URL, HTML, Number base, Timestamp, JWT.
- [x] 4.2 Implement Base64 encode/decode with UTF-8 safety (`btoa(unescape(encodeURIComponent(…)))` on encode, inverse on decode) and try/catch error messages.
- [x] 4.3 Implement Hex↔ASCII conversion with even-length validation.
- [x] 4.4 Implement URL encode/decode via `encodeURIComponent` / `decodeURIComponent`.
- [x] 4.5 Implement HTML escape/unescape for `& < > " '`.
- [x] 4.6 Implement Number base conversion with From/To selectors (2/8/10/16) and a result panel showing all four bases.
- [x] 4.7 Implement Timestamp↔date conversion:
  - [x] 4.7.1 Add a unit selector (`Seconds (s)` / `Milliseconds (ms)`); default to Seconds and persist the choice in component state.
  - [x] 4.7.2 Support **batch input**: parse the timestamp textarea by splitting on `\n`, `,`, whitespace, and tabs; trim empty tokens.
  - [x] 4.7.3 For each token, normalize to a `Date` using the selected unit (`new Date(n)` for ms, `new Date(n * 1000)` for s); invalid tokens keep their raw text and carry an "Invalid" flag.
  - [x] 4.7.4 Render results as a table with columns: Input, Local Datetime, ISO 8601; invalid rows render the original token in a muted/error style.
  - [x] 4.7.5 Implement "To Timestamp" from the datetime picker honoring the selected unit (`getTime()` for ms, `Math.floor(getTime()/1000)` for s); switching the unit re-renders the existing output without re-entering the datetime.
  - [x] 4.7.6 "Current" button fills the input with `Date.now()` or `Math.floor(Date.now()/1000)` based on the selected unit.
  - [x] 4.7.7 Add a "Copy all" button that copies the batch result as tab-separated values (Input \t Local \t ISO) so it pastes cleanly into a spreadsheet.
- [x] 4.8 Implement JWT decode: split on `.`, Base64url-decode header and payload, JSON.parse, display header, payload, iat/exp, validity duration, and expired/valid status.
- [x] 4.9 Implement JWT encode using `hmacSha256` from `src/utils/crypto.ts` (correct HS256, unlike the Vue reference); validate JSON inputs and non-empty secret.
- [x] 4.10 Add Copy and Clear buttons for every tool within the group.

## 5. Checksum Tools Component

- [x] 5.1 Create `src/components/toolbox/ChecksumTools.tsx` with an algorithm selector (MD5, SHA-1, SHA-256, SHA-384, SHA-512).
- [x] 5.2 Compute the digest in an async handler using `crypto.ts` helpers and display the hex result.
- [x] 5.3 Show a persistent hint next to the MD5 option: "MD5 is not cryptographically secure — use SHA-256 for security-critical work."
- [x] 5.4 Add Copy and Clear buttons.
- [x] 5.5 Add a loading indicator ("Computing…") for inputs larger than 64 KB.

## 6. Generation Tools Component

- [x] 6.1 Create `src/components/toolbox/GenerationTools.tsx` with sub-selectors for: UUID, Random String, Password.
- [x] 6.2 Implement UUID v4 using `crypto.randomUUID()` (available in Tauri v2 webview).
- [x] 6.3 Implement Random String with numeric input for length and checkboxes for character classes (letters, digits, symbols); use `crypto.getRandomValues` for randomness.
- [x] 6.4 Implement Password generator with length slider (8–64) and checkboxes for uppercase/digits/symbols; guarantee at least one character from each enabled class.
- [x] 6.5 Wire Copy buttons with the `copyText` helper and a brief "Copied!" toast.

## 7. Encryption Tools Component

- [x] 7.1 Create `src/components/toolbox/EncryptionTools.tsx` with a mode selector: AES-CBC, AES-GCM, ROT13, Caesar, Atbash.
- [x] 7.2 Implement AES-CBC encrypt/decrypt using `crypto.ts` helpers; accept hex-encoded key and IV; output Base64 ciphertext.
- [x] 7.3 Implement AES-GCM encrypt/decrypt with the same I/O contract; surface `OperationError` as "Decryption failed — wrong key, IV, or corrupted input."
- [x] 7.4 Implement ROT13 as a pure string transform.
- [x] 7.5 Implement Caesar cipher with a numeric shift input (−25…25); preserve case; pass non-letter characters through unchanged.
- [x] 7.6 Implement Atbash cipher (`a↔z, b↔y, …`).
- [x] 7.7 Add Copy and Clear buttons for each sub-tool.

## 8. Help (optional, lightweight)

- [x] 8.1 Create `src/components/toolbox/ToolsHelp.tsx` with a short description of each group and brief usage tips (1–2 lines per tool).
- [x] 8.2 Link or embed the Help content as a fifth inner tab in `ToolboxPanel`.

## 9. Testing & Verification

- [ ] 9.1 Manually verify every scenario in `specs/toolbox/spec.md` against the running dev build (`npm run tauri dev`). _(manual)_
- [x] 9.2 Verify SHA-256(`abc`) equals `ba7816bf...20015ad` (spec scenario). _(Node reference match, same algorithm)_
- [ ] 9.3 Verify JWT encode output verifies in a third-party validator (e.g. jwt.io) using the supplied secret. _(manual; HS256 produced via Web Crypto HMAC-SHA256)_
- [ ] 9.4 Verify AES-GCM round-trip (encrypt → decrypt → original plaintext) for 128/192/256-bit keys. _(manual in dev build)_
- [ ] 9.5 Verify the 1 MB input cap blocks oversized inputs with a clear warning. _(manual)_
- [x] 9.6 Run `npm run build` and confirm no TypeScript errors and bundle size delta is under ~30 KB gzipped. _(JS +~3 KB, CSS +~0.7 KB gzipped; build ok)_
- [ ] 9.7 Run the app, switch between all sidebar tabs, and confirm Toolbox inner-tab state persists across sidebar switches. _(manual)_
- [ ] 9.8 Test keyboard navigation on inner tabs (Arrow keys, Home, End, Enter). _(manual)_
- [x] 9.9 Confirm no `invoke()` call is made from any Toolbox interaction (grep + DevTools verification). _(grep: 0 matches in toolbox/ and utils/crypto.ts)_

## 10. Docs & Cleanup

- [x] 10.1 Update `README.md` (if it lists features) to mention the Toolbox tab.
- [x] 10.2 Update `doc/03-tech-stack.md` if it enumerates frontend components.
- [x] 10.3 Run `openspec validate toolbox-component` and fix any schema warnings.
- [ ] 10.4 Archive this change via `/opsx-archive` after merge.
