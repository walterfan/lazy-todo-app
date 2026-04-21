export function ToolsHelp() {
  return (
    <div className="tool-group tool-help">
      <h4>🔄 Conversion</h4>
      <p>Encode/decode text between common formats without leaving the app.</p>
      <ul>
        <li><b>Base64</b> — UTF-8-safe encode/decode of arbitrary text.</li>
        <li><b>Hex ↔ ASCII</b> — View or recover the byte-level representation of text.</li>
        <li><b>URL / HTML</b> — Standard percent-encoding and HTML entity escape.</li>
        <li><b>Number base</b> — Convert between binary, octal, decimal, and hex.</li>
        <li><b>Timestamp</b> — Seconds or milliseconds; supports <b>batch input</b> (newline/comma/space separated).</li>
        <li><b>JWT</b> — Inspect a token (header / payload / iat / exp / status) or mint an HS256 token with a secret.</li>
      </ul>

      <h4>🔏 Checksum</h4>
      <p>Compute hex digests via Web Crypto (SHA family) or a built-in MD5.</p>
      <ul>
        <li>Use <b>SHA-256</b> or better for security-critical work.</li>
        <li>MD5 and SHA-1 are retained for compatibility only.</li>
      </ul>

      <h4>✨ Generation</h4>
      <p>Produce IDs and credentials from the OS secure RNG.</p>
      <ul>
        <li><b>UUID v4</b> via <code>crypto.randomUUID()</code>.</li>
        <li><b>Random string</b> — configurable length and character classes.</li>
        <li><b>Password</b> — 8–64 characters, guarantees at least one from each enabled class.</li>
      </ul>

      <h4>🔐 Encryption</h4>
      <p>Symmetric encryption and classical ciphers. Everything runs locally.</p>
      <ul>
        <li><b>AES-GCM / AES-CBC</b> — 128/192/256-bit keys, hex-encoded key and IV, Base64 output.</li>
        <li><b>ROT13 / Atbash / Caesar</b> — letter-only transforms for puzzles and demos.</li>
      </ul>

      <h4 style={{ color: "var(--warning)" }}>Privacy</h4>
      <p>
        Inputs and outputs are <b>never persisted</b> and <b>never sent over the network</b>. Values live
        only in memory for this window and are discarded when you close the app or switch to a different
        Toolbox tab is still preserved in memory but not written to disk.
      </p>
    </div>
  );
}
