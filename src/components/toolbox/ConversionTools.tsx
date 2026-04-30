import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { assertInputSize, bytesToBase64, copyText, hmacSha256Base64Url, InputTooLargeError } from "../../utils/crypto";

type Tool =
  | "base64"
  | "hex-ascii"
  | "url"
  | "html"
  | "number"
  | "timestamp"
  | "jwt";

type TimestampUnit = "s" | "ms";

interface TsRow {
  input: string;
  local: string;
  iso: string;
  valid: boolean;
}

function showToast(msg: string) {
  const el = document.createElement("div");
  el.className = "tool-toast";
  el.textContent = msg;
  document.body.appendChild(el);
  setTimeout(() => el.remove(), 1600);
}

async function doCopy(text: string, copied: string, failed: string) {
  const ok = await copyText(text);
  showToast(ok ? copied : failed);
}

function handleSizeError(err: unknown): string {
  if (err instanceof InputTooLargeError) {
    return `Error: input exceeds 1 MB limit (${err.bytes.toLocaleString()} bytes).`;
  }
  return err instanceof Error ? `Error: ${err.message}` : "Error";
}

// ---------- UTF-8-safe Base64 ----------
function utf8ToBase64(s: string): string {
  return btoa(unescape(encodeURIComponent(s)));
}
function base64ToUtf8(s: string): string {
  return decodeURIComponent(escape(atob(s)));
}

export function ConversionTools() {
  const { t } = useTranslation();
  const [tool, setTool] = useState<Tool>("base64");

  // Base64
  const [b64In, setB64In] = useState("");
  const [b64Out, setB64Out] = useState("");

  // Hex <-> ASCII
  const [hexIn, setHexIn] = useState("");
  const [hexOut, setHexOut] = useState("");

  // URL
  const [urlIn, setUrlIn] = useState("");
  const [urlOut, setUrlOut] = useState("");

  // HTML
  const [htmlIn, setHtmlIn] = useState("");
  const [htmlOut, setHtmlOut] = useState("");

  // Number base
  const [numIn, setNumIn] = useState("");
  const [fromBase, setFromBase] = useState("10");
  const [numResult, setNumResult] = useState({ bin: "", oct: "", dec: "", hex: "" });

  // Timestamp
  const [tsUnit, setTsUnit] = useState<TimestampUnit>("s");
  const [tsIn, setTsIn] = useState("");
  const [tsRows, setTsRows] = useState<TsRow[]>([]);
  const [tsDateInput, setTsDateInput] = useState("");
  const [tsDateOut, setTsDateOut] = useState("");

  // JWT
  const [jwtIn, setJwtIn] = useState("");
  const [jwtHeader, setJwtHeader] = useState("");
  const [jwtPayload, setJwtPayload] = useState("");
  const [jwtInfo, setJwtInfo] = useState<{ iat?: number; exp?: number } | null>(null);

  const [jwtEncHeader, setJwtEncHeader] = useState('{"alg":"HS256","typ":"JWT"}');
  const [jwtEncPayload, setJwtEncPayload] = useState('{"sub":"1234567890","name":"walter fan","iat":1516239022}');
  const [jwtSecret, setJwtSecret] = useState("");
  const [jwtEncOut, setJwtEncOut] = useState("");

  // ----- handlers -----

  const encodeBase64 = () => {
    try {
      assertInputSize(b64In);
      setB64Out(utf8ToBase64(b64In));
    } catch (e) {
      setB64Out(handleSizeError(e));
    }
  };
  const decodeBase64 = () => {
    try {
      assertInputSize(b64In);
      setB64Out(base64ToUtf8(b64In));
    } catch {
      setB64Out("Error: Invalid Base64 string");
    }
  };

  const textToHex = () => {
    try {
      assertInputSize(hexIn);
      setHexOut(
        Array.from(new TextEncoder().encode(hexIn))
          .map((b) => b.toString(16).padStart(2, "0"))
          .join(" ")
      );
    } catch (e) {
      setHexOut(handleSizeError(e));
    }
  };
  const hexToText = () => {
    try {
      const clean = hexIn.replace(/\s+/g, "");
      if (clean.length % 2 !== 0) {
        setHexOut("Error: Hex string must have even length");
        return;
      }
      if (!/^[0-9a-fA-F]*$/.test(clean)) {
        setHexOut("Error: Invalid hex characters");
        return;
      }
      const bytes = new Uint8Array(clean.length / 2);
      for (let i = 0; i < bytes.length; i++) bytes[i] = parseInt(clean.substr(i * 2, 2), 16);
      setHexOut(new TextDecoder().decode(bytes));
    } catch {
      setHexOut("Error: Invalid hex string");
    }
  };

  const encodeUrl = () => {
    try {
      assertInputSize(urlIn);
      setUrlOut(encodeURIComponent(urlIn));
    } catch (e) {
      setUrlOut(handleSizeError(e));
    }
  };
  const decodeUrl = () => {
    try {
      assertInputSize(urlIn);
      setUrlOut(decodeURIComponent(urlIn));
    } catch {
      setUrlOut("Error: Invalid URL encoded string");
    }
  };

  const escapeHtml = () => {
    try {
      assertInputSize(htmlIn);
      const map: Record<string, string> = {
        "&": "&amp;",
        "<": "&lt;",
        ">": "&gt;",
        '"': "&quot;",
        "'": "&#39;",
      };
      setHtmlOut(htmlIn.replace(/[&<>"']/g, (m) => map[m]));
    } catch (e) {
      setHtmlOut(handleSizeError(e));
    }
  };
  const unescapeHtml = () => {
    try {
      assertInputSize(htmlIn);
      const map: Record<string, string> = {
        "&amp;": "&",
        "&lt;": "<",
        "&gt;": ">",
        "&quot;": '"',
        "&#39;": "'",
        "&apos;": "'",
      };
      setHtmlOut(htmlIn.replace(/&(?:amp|lt|gt|quot|#39|apos);/g, (m) => map[m]));
    } catch (e) {
      setHtmlOut(handleSizeError(e));
    }
  };

  const convertNumber = () => {
    const trimmed = numIn.trim();
    if (!trimmed) {
      setNumResult({ bin: "", oct: "", dec: "", hex: "" });
      return;
    }
    const dec = parseInt(trimmed, parseInt(fromBase, 10));
    if (isNaN(dec)) {
      const err = "Error: invalid number";
      setNumResult({ bin: err, oct: err, dec: err, hex: err });
      return;
    }
    setNumResult({
      bin: dec.toString(2),
      oct: dec.toString(8),
      dec: dec.toString(10),
      hex: dec.toString(16).toUpperCase(),
    });
  };

  // ----- Timestamp (unit-aware, batch-capable) -----

  const tokensOf = (text: string): string[] =>
    text.split(/[\s,;\t\r\n]+/).map((s) => s.trim()).filter(Boolean);

  const numToDate = (n: number): Date => (tsUnit === "s" ? new Date(n * 1000) : new Date(n));

  const convertTimestamps = () => {
    const tokens = tokensOf(tsIn);
    if (tokens.length === 0) {
      setTsRows([]);
      return;
    }
    const rows: TsRow[] = tokens.map((tok) => {
      const n = Number(tok);
      if (!Number.isFinite(n)) {
        return { input: tok, local: "Invalid", iso: "Invalid", valid: false };
      }
      const d = numToDate(n);
      if (isNaN(d.getTime())) {
        return { input: tok, local: "Invalid", iso: "Invalid", valid: false };
      }
      return { input: tok, local: d.toLocaleString(), iso: d.toISOString(), valid: true };
    });
    setTsRows(rows);
  };

  // Re-render existing rows when user switches unit (without requiring re-entry).
  const recomputeRowsFor = (unit: TimestampUnit): TsRow[] =>
    tsRows.map((r) => {
      if (!r.valid) return r;
      const n = Number(r.input);
      const d = unit === "s" ? new Date(n * 1000) : new Date(n);
      if (isNaN(d.getTime())) return { ...r, local: "Invalid", iso: "Invalid", valid: false };
      return { ...r, local: d.toLocaleString(), iso: d.toISOString() };
    });

  const changeUnit = (unit: TimestampUnit) => {
    setTsUnit(unit);
    if (tsRows.length > 0) setTsRows(recomputeRowsFor(unit));
    if (tsDateInput) {
      const d = new Date(tsDateInput);
      if (!isNaN(d.getTime())) {
        setTsDateOut(unit === "s" ? Math.floor(d.getTime() / 1000).toString() : d.getTime().toString());
      }
    }
  };

  const currentTimestamp = () => {
    const now = Date.now();
    const v = tsUnit === "s" ? Math.floor(now / 1000).toString() : now.toString();
    setTsIn(v);
  };

  const dateToTimestamp = () => {
    if (!tsDateInput) {
      setTsDateOut("Error: pick a date first");
      return;
    }
    const d = new Date(tsDateInput);
    if (isNaN(d.getTime())) {
      setTsDateOut("Error: invalid date");
      return;
    }
    setTsDateOut(tsUnit === "s" ? Math.floor(d.getTime() / 1000).toString() : d.getTime().toString());
  };

  const copyBatch = async () => {
    if (tsRows.length === 0) return;
    const tsv = tsRows.map((r) => `${r.input}\t${r.local}\t${r.iso}`).join("\n");
    await doCopy(tsv, t("copiedBang"), t("copyFailedBare"));
  };

  const clearTimestamps = () => {
    setTsIn("");
    setTsRows([]);
    setTsDateInput("");
    setTsDateOut("");
  };

  // ----- JWT -----

  const b64UrlDecode = (s: string): string => {
    const pad = s.length % 4 === 0 ? "" : "=".repeat(4 - (s.length % 4));
    return atob(s.replace(/-/g, "+").replace(/_/g, "/") + pad);
  };

  const decodeJwt = () => {
    try {
      const t = jwtIn.trim();
      if (!t) {
        setJwtHeader("Error: no JWT provided");
        setJwtPayload("");
        setJwtInfo(null);
        return;
      }
      const parts = t.split(".");
      if (parts.length !== 3) {
        setJwtHeader("Error: invalid JWT (expected 3 parts)");
        setJwtPayload("");
        setJwtInfo(null);
        return;
      }
      const header = JSON.parse(b64UrlDecode(parts[0]));
      const payload = JSON.parse(b64UrlDecode(parts[1]));
      setJwtHeader(JSON.stringify(header, null, 2));
      setJwtPayload(JSON.stringify(payload, null, 2));
      setJwtInfo({ iat: payload.iat, exp: payload.exp });
    } catch (e) {
      const msg = e instanceof Error ? e.message : "unknown error";
      setJwtHeader(`Error: failed to decode — ${msg}`);
      setJwtPayload("");
      setJwtInfo(null);
    }
  };

  const encodeJwt = async () => {
    try {
      if (!jwtEncHeader.trim()) return setJwtEncOut("Error: header is required");
      if (!jwtEncPayload.trim()) return setJwtEncOut("Error: payload is required");
      if (!jwtSecret) return setJwtEncOut("Error: secret is required");
      const header = JSON.parse(jwtEncHeader);
      const payload = JSON.parse(jwtEncPayload);
      const enc = (s: string) =>
        bytesToBase64(new TextEncoder().encode(s)).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/g, "");
      const headB = enc(JSON.stringify(header));
      const payB = enc(JSON.stringify(payload));
      const sig = await hmacSha256Base64Url(jwtSecret, `${headB}.${payB}`);
      setJwtEncOut(`${headB}.${payB}.${sig}`);
    } catch (e) {
      setJwtEncOut(`Error: ${e instanceof Error ? e.message : "invalid JSON"}`);
    }
  };

  const jwtStatus = useMemo(() => {
    if (!jwtInfo?.exp) return null;
    const now = Math.floor(Date.now() / 1000);
    return {
      iat: jwtInfo.iat ? new Date(jwtInfo.iat * 1000).toLocaleString() : "—",
      exp: jwtInfo.exp ? new Date(jwtInfo.exp * 1000).toLocaleString() : "—",
      expired: now > jwtInfo.exp,
    };
  }, [jwtInfo]);

  // ----- render -----

  return (
    <div className="tool-group">
      <div className="tool-group-header">
        <label htmlFor="conv-tool">{t("tool")}:</label>
        <select
          id="conv-tool"
          className="tool-select"
          value={tool}
          onChange={(e) => setTool(e.target.value as Tool)}
        >
          <option value="base64">Base64 {t("encode")}/{t("decode")}</option>
          <option value="hex-ascii">Hex ↔ ASCII</option>
          <option value="url">URL {t("encode")}/{t("decode")}</option>
          <option value="html">HTML {t("escape")}/{t("unescape")}</option>
          <option value="number">{t("fromBase")} {t("conversion")}</option>
          <option value="timestamp">Timestamp ↔ Datetime</option>
          <option value="jwt">JWT {t("encode")}/{t("decode")}</option>
        </select>
      </div>

      {tool === "base64" && (
        <div className="tool-io-grid">
          <div className="tool-io-col">
            <label>{t("input")}</label>
            <textarea className="tool-textarea" value={b64In} onChange={(e) => setB64In(e.target.value)} placeholder="Enter text…" />
            <div className="tool-actions">
              <button className="tool-btn primary" onClick={encodeBase64}>{t("encode")}</button>
              <button className="tool-btn" onClick={decodeBase64}>{t("decode")}</button>
              <button className="tool-btn danger" onClick={() => { setB64In(""); setB64Out(""); }}>{t("clear")}</button>
            </div>
          </div>
          <div className="tool-io-col">
            <label>{t("output")}</label>
            <textarea className="tool-textarea" value={b64Out} readOnly placeholder="Result…" />
            <div className="tool-actions">
              <button className="tool-btn" onClick={() => doCopy(b64Out, t("copiedBang"), t("copyFailedBare"))}>{t("copyToClipboard")}</button>
            </div>
          </div>
        </div>
      )}

      {tool === "hex-ascii" && (
        <div className="tool-io-grid">
          <div className="tool-io-col">
            <label>{t("input")}</label>
            <textarea className="tool-textarea" value={hexIn} onChange={(e) => setHexIn(e.target.value)} placeholder="Text or hex…" />
            <div className="tool-actions">
              <button className="tool-btn primary" onClick={textToHex}>Text → Hex</button>
              <button className="tool-btn" onClick={hexToText}>Hex → Text</button>
              <button className="tool-btn danger" onClick={() => { setHexIn(""); setHexOut(""); }}>{t("clear")}</button>
            </div>
          </div>
          <div className="tool-io-col">
            <label>{t("output")}</label>
            <textarea className="tool-textarea" value={hexOut} readOnly />
            <div className="tool-actions">
              <button className="tool-btn" onClick={() => doCopy(hexOut, t("copiedBang"), t("copyFailedBare"))}>{t("copyToClipboard")}</button>
            </div>
          </div>
        </div>
      )}

      {tool === "url" && (
        <div className="tool-io-grid">
          <div className="tool-io-col">
            <label>{t("input")}</label>
            <textarea className="tool-textarea" value={urlIn} onChange={(e) => setUrlIn(e.target.value)} placeholder="URL or component…" />
            <div className="tool-actions">
              <button className="tool-btn primary" onClick={encodeUrl}>{t("encode")}</button>
              <button className="tool-btn" onClick={decodeUrl}>{t("decode")}</button>
              <button className="tool-btn danger" onClick={() => { setUrlIn(""); setUrlOut(""); }}>{t("clear")}</button>
            </div>
          </div>
          <div className="tool-io-col">
            <label>{t("output")}</label>
            <textarea className="tool-textarea" value={urlOut} readOnly />
            <div className="tool-actions">
              <button className="tool-btn" onClick={() => doCopy(urlOut, t("copiedBang"), t("copyFailedBare"))}>{t("copyToClipboard")}</button>
            </div>
          </div>
        </div>
      )}

      {tool === "html" && (
        <div className="tool-io-grid">
          <div className="tool-io-col">
            <label>{t("input")}</label>
            <textarea className="tool-textarea" value={htmlIn} onChange={(e) => setHtmlIn(e.target.value)} placeholder="HTML fragment…" />
            <div className="tool-actions">
              <button className="tool-btn primary" onClick={escapeHtml}>{t("escape")}</button>
              <button className="tool-btn" onClick={unescapeHtml}>{t("unescape")}</button>
              <button className="tool-btn danger" onClick={() => { setHtmlIn(""); setHtmlOut(""); }}>{t("clear")}</button>
            </div>
          </div>
          <div className="tool-io-col">
            <label>{t("output")}</label>
            <textarea className="tool-textarea" value={htmlOut} readOnly />
            <div className="tool-actions">
              <button className="tool-btn" onClick={() => doCopy(htmlOut, t("copiedBang"), t("copyFailedBare"))}>{t("copyToClipboard")}</button>
            </div>
          </div>
        </div>
      )}

      {tool === "number" && (
        <div className="tool-io-grid">
          <div className="tool-io-col">
            <label>{t("input")}</label>
            <input className="tool-input" value={numIn} onChange={(e) => setNumIn(e.target.value)} placeholder="e.g. 255" />
            <label>{t("fromBase")}</label>
            <select className="tool-select" value={fromBase} onChange={(e) => setFromBase(e.target.value)}>
              <option value="2">{t("binary")} (2)</option>
              <option value="8">{t("octal")} (8)</option>
              <option value="10">{t("decimal")} (10)</option>
              <option value="16">{t("hex")} (16)</option>
            </select>
            <div className="tool-actions">
              <button className="tool-btn primary" onClick={convertNumber}>{t("convert")}</button>
              <button className="tool-btn danger" onClick={() => { setNumIn(""); setNumResult({ bin: "", oct: "", dec: "", hex: "" }); }}>{t("clear")}</button>
            </div>
          </div>
          <div className="tool-io-col">
            <label>{t("results")}</label>
            <div className="tool-results-panel">
              <span className="k">{t("binary")} (2):</span><span className="v">{numResult.bin}</span>
              <span className="k">{t("octal")} (8):</span><span className="v">{numResult.oct}</span>
              <span className="k">{t("decimal")} (10):</span><span className="v">{numResult.dec}</span>
              <span className="k">{t("hex")} (16):</span><span className="v">{numResult.hex}</span>
            </div>
          </div>
        </div>
      )}

      {tool === "timestamp" && (
        <div className="tool-group">
          <div className="tool-group-header">
            <label>{t("unit")}:</label>
            <select className="tool-select" value={tsUnit} onChange={(e) => changeUnit(e.target.value as TimestampUnit)}>
              <option value="s">Seconds (s)</option>
              <option value="ms">Milliseconds (ms)</option>
            </select>
            <span className="tool-hint">Batch input: newlines, commas, or spaces separate multiple timestamps.</span>
          </div>
          <div className="tool-io-grid">
            <div className="tool-io-col">
              <label>{t("timestampInput")}</label>
              <textarea
                className="tool-textarea"
                value={tsIn}
                onChange={(e) => setTsIn(e.target.value)}
                placeholder={"e.g.\n1700000000\n1700003600, 1700007200"}
              />
              <div className="tool-actions">
                <button className="tool-btn primary" onClick={convertTimestamps}>{t("convert")}</button>
                <button className="tool-btn" onClick={currentTimestamp}>{t("current")}</button>
                <button className="tool-btn" onClick={copyBatch} disabled={tsRows.length === 0}>{t("copyAllTsv")}</button>
                <button className="tool-btn danger" onClick={clearTimestamps}>{t("clear")}</button>
              </div>
            </div>
            <div className="tool-io-col">
              <label>{t("datetimePickerToTimestamp")}</label>
              <input
                type="datetime-local"
                className="tool-input"
                value={tsDateInput}
                onChange={(e) => setTsDateInput(e.target.value)}
              />
              <div className="tool-actions">
                <button className="tool-btn primary" onClick={dateToTimestamp}>{t("toTimestamp")}</button>
              </div>
              <div className="tool-output-box">{tsDateOut || "—"}</div>
            </div>
          </div>

          {tsRows.length > 0 && (
            <table className="tool-table">
              <thead>
                <tr>
                  <th>{t("input")}</th>
                  <th>Local Datetime</th>
                  <th>ISO 8601</th>
                </tr>
              </thead>
              <tbody>
                {tsRows.map((r, i) => (
                  <tr key={i}>
                    <td className="mono">{r.input}</td>
                    <td className={r.valid ? "" : "invalid"}>{r.local}</td>
                    <td className={r.valid ? "mono" : "invalid"}>{r.iso}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      )}

      {tool === "jwt" && (
        <div className="tool-group">
          <div className="tool-io-grid">
            <div className="tool-io-col">
              <label>{t("jwtTokenToDecode")}</label>
              <textarea className="tool-textarea" value={jwtIn} onChange={(e) => setJwtIn(e.target.value)} placeholder="eyJhbGciOi…" />
              <div className="tool-actions">
                <button className="tool-btn primary" onClick={decodeJwt}>{t("decode")}</button>
                <button className="tool-btn danger" onClick={() => { setJwtIn(""); setJwtHeader(""); setJwtPayload(""); setJwtInfo(null); }}>{t("clear")}</button>
              </div>
            </div>
            <div className="tool-io-col">
              <label>{t("header")}</label>
              <textarea className="tool-textarea" value={jwtHeader} readOnly />
              <label>{t("payload")}</label>
              <textarea className="tool-textarea" value={jwtPayload} readOnly />
              <div className="tool-actions">
                <button className="tool-btn" onClick={() => doCopy(jwtHeader, t("copiedBang"), t("copyFailedBare"))}>{t("copyHeader")}</button>
                <button className="tool-btn" onClick={() => doCopy(jwtPayload, t("copiedBang"), t("copyFailedBare"))}>{t("copyPayload")}</button>
              </div>
            </div>
          </div>

          {jwtStatus && (
            <div className="tool-results-panel">
              <span className="k">{t("issuedAt")}:</span><span className="v">{jwtStatus.iat}</span>
              <span className="k">{t("expiresAt")}:</span><span className="v">{jwtStatus.exp}</span>
              <span className="k">{t("status")}:</span>
              <span className="v" style={{ color: jwtStatus.expired ? "var(--danger)" : "var(--success)" }}>
                {jwtStatus.expired ? t("expired") : t("valid")}
              </span>
            </div>
          )}

          <hr style={{ border: "none", borderTop: "1px solid var(--border)", margin: "8px 0" }} />

          <div className="tool-io-grid">
            <div className="tool-io-col">
              <label>{t("headerJson")}</label>
              <textarea className="tool-textarea" value={jwtEncHeader} onChange={(e) => setJwtEncHeader(e.target.value)} />
              <label>{t("payloadJson")}</label>
              <textarea className="tool-textarea" value={jwtEncPayload} onChange={(e) => setJwtEncPayload(e.target.value)} />
              <label>{t("secretHs256")}</label>
              <input className="tool-input" type="password" value={jwtSecret} onChange={(e) => setJwtSecret(e.target.value)} />
              <div className="tool-actions">
                <button className="tool-btn primary" onClick={encodeJwt}>{t("encode")}</button>
                <button className="tool-btn danger" onClick={() => { setJwtEncHeader('{"alg":"HS256","typ":"JWT"}'); setJwtEncPayload('{"sub":"1234567890","name":"walter fan","iat":1516239022}'); setJwtSecret(""); setJwtEncOut(""); }}>{t("reset")}</button>
              </div>
            </div>
            <div className="tool-io-col">
              <label>{t("encodedJwt")}</label>
              <textarea className="tool-textarea" value={jwtEncOut} readOnly />
              <div className="tool-actions">
                <button className="tool-btn" onClick={() => doCopy(jwtEncOut, t("copiedBang"), t("copyFailedBare"))}>{t("copyToClipboard")}</button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
