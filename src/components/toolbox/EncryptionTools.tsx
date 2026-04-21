import { useState } from "react";
import { aesDecrypt, aesEncrypt, AesMode, copyText, InputTooLargeError } from "../../utils/crypto";

type Mode = AesMode | "ROT13" | "Caesar" | "Atbash";

function showToast(msg: string) {
  const el = document.createElement("div");
  el.className = "tool-toast";
  el.textContent = msg;
  document.body.appendChild(el);
  setTimeout(() => el.remove(), 1600);
}

async function doCopy(text: string) {
  const ok = await copyText(text);
  showToast(ok ? "Copied!" : "Copy failed");
}

function rot13(s: string): string {
  return s.replace(/[a-zA-Z]/g, (c) => {
    const base = c <= "Z" ? 65 : 97;
    return String.fromCharCode(((c.charCodeAt(0) - base + 13) % 26) + base);
  });
}

function caesar(s: string, shift: number): string {
  const n = ((shift % 26) + 26) % 26;
  return s.replace(/[a-zA-Z]/g, (c) => {
    const base = c <= "Z" ? 65 : 97;
    return String.fromCharCode(((c.charCodeAt(0) - base + n) % 26) + base);
  });
}

function atbash(s: string): string {
  return s.replace(/[a-zA-Z]/g, (c) => {
    const base = c <= "Z" ? 65 : 97;
    return String.fromCharCode(base + 25 - (c.charCodeAt(0) - base));
  });
}

function handleErr(e: unknown): string {
  if (e instanceof InputTooLargeError) return `Error: input exceeds 1 MB limit (${e.bytes.toLocaleString()} bytes).`;
  return `Error: ${e instanceof Error ? e.message : "unknown error"}`;
}

export function EncryptionTools() {
  const [mode, setMode] = useState<Mode>("AES-GCM");

  // AES state
  const [aesKey, setAesKey] = useState("");
  const [aesIv, setAesIv] = useState("");
  const [aesPlain, setAesPlain] = useState("");
  const [aesCipher, setAesCipher] = useState("");

  // Classical state
  const [classicIn, setClassicIn] = useState("");
  const [classicOut, setClassicOut] = useState("");
  const [caesarShift, setCaesarShift] = useState(3);

  const isAes = mode === "AES-CBC" || mode === "AES-GCM";

  const encrypt = async () => {
    try {
      const result = await aesEncrypt(mode as AesMode, aesKey, aesIv, aesPlain);
      setAesCipher(result);
    } catch (e) {
      setAesCipher(handleErr(e));
    }
  };

  const decrypt = async () => {
    try {
      const result = await aesDecrypt(mode as AesMode, aesKey, aesIv, aesCipher);
      setAesPlain(result);
    } catch (e) {
      setAesPlain(handleErr(e));
    }
  };

  const applyClassical = () => {
    try {
      if (mode === "ROT13") setClassicOut(rot13(classicIn));
      else if (mode === "Caesar") setClassicOut(caesar(classicIn, caesarShift));
      else if (mode === "Atbash") setClassicOut(atbash(classicIn));
    } catch (e) {
      setClassicOut(handleErr(e));
    }
  };

  const clearAll = () => {
    setAesKey("");
    setAesIv("");
    setAesPlain("");
    setAesCipher("");
    setClassicIn("");
    setClassicOut("");
  };

  return (
    <div className="tool-group">
      <div className="tool-group-header">
        <label htmlFor="enc-mode">Mode:</label>
        <select
          id="enc-mode"
          className="tool-select"
          value={mode}
          onChange={(e) => setMode(e.target.value as Mode)}
        >
          <option value="AES-GCM">AES-GCM</option>
          <option value="AES-CBC">AES-CBC</option>
          <option value="ROT13">ROT13</option>
          <option value="Caesar">Caesar</option>
          <option value="Atbash">Atbash</option>
        </select>
        <button className="tool-btn danger" onClick={clearAll}>Clear all</button>
      </div>

      {isAes && (
        <>
          <div className="tool-field-row">
            <label style={{ minWidth: 80 }}>Key (hex)</label>
            <input
              className="tool-input"
              value={aesKey}
              onChange={(e) => setAesKey(e.target.value)}
              placeholder="16 / 24 / 32 bytes of hex (e.g. 32-char, 48-char, 64-char)"
              style={{ flex: 1 }}
            />
          </div>
          <div className="tool-field-row">
            <label style={{ minWidth: 80 }}>IV (hex)</label>
            <input
              className="tool-input"
              value={aesIv}
              onChange={(e) => setAesIv(e.target.value)}
              placeholder={mode === "AES-CBC" ? "16 bytes (32 hex chars)" : "12 bytes (24 hex chars)"}
              style={{ flex: 1 }}
            />
          </div>

          <div className="tool-io-grid">
            <div className="tool-io-col">
              <label>Plaintext</label>
              <textarea
                className="tool-textarea"
                value={aesPlain}
                onChange={(e) => setAesPlain(e.target.value)}
                placeholder="Text to encrypt…"
              />
              <div className="tool-actions">
                <button className="tool-btn primary" onClick={encrypt}>Encrypt →</button>
                <button className="tool-btn" onClick={() => doCopy(aesPlain)}>Copy</button>
              </div>
            </div>
            <div className="tool-io-col">
              <label>Ciphertext (Base64)</label>
              <textarea
                className="tool-textarea"
                value={aesCipher}
                onChange={(e) => setAesCipher(e.target.value)}
                placeholder="Base64 ciphertext…"
              />
              <div className="tool-actions">
                <button className="tool-btn primary" onClick={decrypt}>← Decrypt</button>
                <button className="tool-btn" onClick={() => doCopy(aesCipher)}>Copy</button>
              </div>
            </div>
          </div>
          <div className="tool-hint">
            Key/IV are hex-encoded. {mode === "AES-GCM" ? "AES-GCM IV: 12 bytes." : "AES-CBC IV: 16 bytes."} Non-letters pass through unchanged.
          </div>
        </>
      )}

      {!isAes && (
        <>
          {mode === "Caesar" && (
            <div className="tool-field-row">
              <label style={{ minWidth: 80 }}>Shift</label>
              <input
                type="number"
                min={-25}
                max={25}
                className="tool-input"
                value={caesarShift}
                onChange={(e) => setCaesarShift(parseInt(e.target.value || "0", 10))}
                style={{ maxWidth: 120 }}
              />
            </div>
          )}
          <div className="tool-io-grid">
            <div className="tool-io-col">
              <label>Input</label>
              <textarea
                className="tool-textarea"
                value={classicIn}
                onChange={(e) => setClassicIn(e.target.value)}
                placeholder="Text…"
              />
              <div className="tool-actions">
                <button className="tool-btn primary" onClick={applyClassical}>Apply {mode}</button>
                <button className="tool-btn danger" onClick={() => { setClassicIn(""); setClassicOut(""); }}>Clear</button>
              </div>
            </div>
            <div className="tool-io-col">
              <label>Output</label>
              <textarea className="tool-textarea" value={classicOut} readOnly />
              <div className="tool-actions">
                <button className="tool-btn" onClick={() => doCopy(classicOut)}>Copy</button>
              </div>
            </div>
          </div>
        </>
      )}
    </div>
  );
}
