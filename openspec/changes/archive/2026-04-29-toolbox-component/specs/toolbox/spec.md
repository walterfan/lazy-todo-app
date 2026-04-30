## ADDED Requirements

### Requirement: Toolbox sidebar navigation entry
The application SHALL expose a top-level sidebar navigation entry labeled "Toolbox" with a tool icon (🧰), positioned immediately below the "Pomodoro" entry and above the "Settings" entry.

#### Scenario: Toolbox tab is visible on launch
- **WHEN** the user launches the application
- **THEN** the sidebar displays nav items in order: Todos, Notes, Pomodoro, Toolbox, (Settings/Quit in the bottom section)

#### Scenario: Clicking Toolbox activates the panel
- **WHEN** the user clicks the Toolbox nav item
- **THEN** the main content area renders the Toolbox panel
- **AND** the header title shows "🧰 Toolbox"
- **AND** the Toolbox nav item has the `active` class

#### Scenario: Search bar hidden on Toolbox tab
- **WHEN** the Toolbox tab is active
- **THEN** the header search bar is not rendered

### Requirement: Toolbox panel inner tab navigation
The Toolbox panel SHALL display four inner tabs — Conversion, Checksum, Generation, Encryption — with the Conversion tab selected by default.

#### Scenario: Default tab on first open
- **WHEN** the user opens the Toolbox tab for the first time in a session
- **THEN** the Conversion inner tab is active
- **AND** the Conversion tool group is rendered

#### Scenario: Switching inner tabs preserves component mount
- **WHEN** the user switches from Conversion to Checksum and back to Conversion
- **THEN** any text the user had entered in the Conversion tab is still visible

#### Scenario: Inner tabs are keyboard-navigable
- **WHEN** focus is on an inner tab button and the user presses the Right Arrow or Left Arrow key
- **THEN** focus moves to the next or previous inner tab
- **AND** pressing Enter or Space activates the focused tab

### Requirement: Conversion tools
The Conversion tab SHALL provide encode/decode utilities for Base64, Hex↔ASCII, URL, HTML escape, Number base (binary/octal/decimal/hex), Unix timestamp↔date (with second/millisecond unit selection and batch conversion), and JWT.

#### Scenario: Base64 round-trip
- **WHEN** the user enters "Hello, 世界" in the Base64 input and clicks Encode
- **THEN** the output textarea shows a valid Base64 string
- **AND** clicking Decode on that output produces "Hello, 世界" again

#### Scenario: URL encode special characters
- **WHEN** the user enters `a b&c=d` and clicks Encode
- **THEN** the output is `a%20b%26c%3Dd`

#### Scenario: Number base conversion displays all four bases
- **WHEN** the user enters `255` with From=Decimal and clicks Convert
- **THEN** the result panel shows Binary `11111111`, Octal `377`, Decimal `255`, Hexadecimal `FF`

#### Scenario: Timestamp unit selector defaults to seconds
- **WHEN** the user opens the Timestamp tool
- **THEN** a unit selector is visible with options "Seconds (s)" and "Milliseconds (ms)"
- **AND** the default selection is "Seconds (s)"

#### Scenario: Single timestamp conversion in seconds
- **WHEN** the user selects unit "Seconds", enters `1700000000`, and clicks Convert
- **THEN** the output row shows the input `1700000000`
- **AND** the local datetime string for `2023-11-14T22:13:20Z` (rendered in the user's local timezone)
- **AND** the ISO 8601 string `2023-11-14T22:13:20.000Z`

#### Scenario: Single timestamp conversion in milliseconds
- **WHEN** the user selects unit "Milliseconds", enters `1700000000000`, and clicks Convert
- **THEN** the output row shows the same calendar instant as the seconds-mode scenario above (`2023-11-14T22:13:20.000Z`)

#### Scenario: Batch timestamp conversion
- **WHEN** the user pastes multiple numbers separated by newlines, commas, spaces, or tabs (e.g. `1700000000\n1700003600, 1700007200`) and clicks Convert
- **THEN** every numeric token is parsed as a timestamp using the currently selected unit
- **AND** the result is rendered as a table (or list) with one row per input showing: original input, local datetime, ISO 8601 string
- **AND** the row order matches the input order

#### Scenario: Batch conversion tolerates invalid entries
- **WHEN** the batch input contains a mix of valid numbers and invalid tokens (e.g. `1700000000, foo, 1700003600`)
- **THEN** valid rows show their converted datetimes
- **AND** invalid rows show the original token with an "Invalid" marker
- **AND** the tool does not abort on the first invalid entry

#### Scenario: Datetime-to-timestamp respects selected unit
- **WHEN** the user picks a datetime value and clicks "To Timestamp" with unit "Milliseconds" selected
- **THEN** the output is `date.getTime()` (milliseconds since epoch)
- **AND** switching unit to "Seconds" re-renders the output as `Math.floor(date.getTime() / 1000)` without requiring a re-entry of the datetime

#### Scenario: "Current" button honors the selected unit
- **WHEN** the user clicks the "Current" button with unit "Milliseconds" selected
- **THEN** the input field is populated with `Date.now()`
- **AND** when unit "Seconds" is selected, the input field is populated with `Math.floor(Date.now() / 1000)`

#### Scenario: Copy batch results
- **WHEN** the user clicks the "Copy all" button on a batch result
- **THEN** the clipboard receives a tab-separated-values block (one row per input, columns: input, local datetime, ISO string) suitable for pasting into a spreadsheet

#### Scenario: JWT decode splits header and payload
- **WHEN** the user pastes a valid JWT (three Base64url parts separated by dots) and clicks Decode JWT
- **THEN** the header textarea shows the decoded JSON header
- **AND** the payload textarea shows the decoded JSON payload
- **AND** if the payload contains `exp`, the token info section shows Expires At in local time and whether it is expired

#### Scenario: JWT encode produces verifiable HMAC-SHA256
- **WHEN** the user provides valid JSON header and payload, a non-empty secret, and clicks Encode JWT
- **THEN** the output is a three-part dot-separated string
- **AND** the signature is produced by HMAC-SHA256 over `base64url(header).base64url(payload)` using the supplied secret
- **AND** the generated token verifies with a standard HS256 validator

#### Scenario: Invalid Base64 input shows friendly error
- **WHEN** the user clicks Decode on non-Base64 input
- **THEN** the output textarea shows an "Error: Invalid Base64 string" message
- **AND** the application does not crash or throw an uncaught exception

### Requirement: Checksum tools
The Checksum tab SHALL compute MD5, SHA-1, SHA-256, SHA-384, and SHA-512 digests of a text input and display the hex-encoded result.

#### Scenario: SHA-256 of known input
- **WHEN** the user enters `abc` and selects SHA-256
- **THEN** the output is `ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad`

#### Scenario: Empty input produces empty-input digest
- **WHEN** the user clears the input and selects SHA-256
- **THEN** the output is `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`

#### Scenario: MD5 insecurity warning
- **WHEN** the user selects the MD5 algorithm
- **THEN** a visible hint states that MD5 is not cryptographically secure and recommends SHA-256 for security-critical uses

### Requirement: Generation tools
The Generation tab SHALL produce random UUIDs (v4), random strings with configurable length and character set, and random passwords with configurable complexity (length, include-uppercase, include-digits, include-symbols).

#### Scenario: UUID v4 format
- **WHEN** the user clicks Generate UUID
- **THEN** the output matches the regex `^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$`

#### Scenario: Password length respected
- **WHEN** the user sets length = 20 and clicks Generate Password
- **THEN** the generated password is exactly 20 characters long

#### Scenario: Password excludes disabled character classes
- **WHEN** the user unchecks "Include Symbols" and generates a password
- **THEN** the output contains no characters from the symbol class (e.g. `!@#$%^&*()-_=+[]{};:,.<>?/`)

#### Scenario: Copy to clipboard
- **WHEN** the user clicks the Copy button next to any generated value
- **THEN** the value is written to the system clipboard
- **AND** a brief confirmation (e.g. "Copied!") appears for at least 1 second

### Requirement: Encryption tools
The Encryption tab SHALL provide AES-CBC and AES-GCM encrypt/decrypt with configurable key size (128/192/256 bits), plus classical ciphers (ROT13, Caesar with configurable shift, Atbash).

#### Scenario: AES-GCM round-trip
- **WHEN** the user enters plaintext, a key and IV, selects AES-GCM 256-bit, and clicks Encrypt
- **THEN** the output is a Base64-encoded ciphertext
- **AND** clicking Decrypt on that ciphertext with the same key and IV produces the original plaintext

#### Scenario: Wrong key fails decryption gracefully
- **WHEN** the user attempts to decrypt AES-GCM ciphertext with an incorrect key
- **THEN** an error message "Decryption failed — wrong key, IV, or corrupted input" is shown
- **AND** the application does not crash

#### Scenario: ROT13 is self-inverse
- **WHEN** the user enters `Hello, World!` and applies ROT13
- **THEN** the output is `Uryyb, Jbeyq!`
- **AND** applying ROT13 again returns `Hello, World!`

#### Scenario: Caesar cipher with shift 3
- **WHEN** the user enters `abc` and applies Caesar with shift 3
- **THEN** the output is `def`
- **AND** non-letter characters (digits, spaces, punctuation) are preserved unchanged

### Requirement: No persistence and no network
The Toolbox SHALL perform all computation client-side inside the Tauri webview and SHALL NOT persist inputs or outputs to SQLite, localStorage, sessionStorage, cookies, or any remote endpoint.

#### Scenario: Inputs cleared on app restart
- **WHEN** the user types text into any tool, closes the application, and reopens it
- **THEN** the previously entered text is not present in any tool input or output

#### Scenario: No Tauri command invocation
- **WHEN** the user performs any Toolbox action (encode, hash, encrypt, generate)
- **THEN** no call is made to `@tauri-apps/api/core.invoke()` as part of that action

### Requirement: Input size limit
The Toolbox SHALL accept text inputs up to 1,048,576 bytes (1 MB) and SHALL display a clear warning if the user attempts to process a larger input.

#### Scenario: Large input rejected with warning
- **WHEN** the user pastes text larger than 1 MB into any tool input and triggers an action
- **THEN** the operation does not execute
- **AND** a warning message indicates the 1 MB limit
