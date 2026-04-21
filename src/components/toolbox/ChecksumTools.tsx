import { useState } from "react";
import { copyText, HashAlgorithm, InputTooLargeError, sha } from "../../utils/crypto";

const ALGORITHMS: HashAlgorithm[] = ["MD5", "SHA-1", "SHA-256", "SHA-384", "SHA-512"];
const LOADING_THRESHOLD_BYTES = 64 * 1024;

function showToast(msg: string) {
  const el = document.createElement("div");
  el.className = "tool-toast";
  el.textContent = msg;
  document.body.appendChild(el);
  setTimeout(() => el.remove(), 1600);
}

export function ChecksumTools() {
  const [algo, setAlgo] = useState<HashAlgorithm>("SHA-256");
  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");
  const [loading, setLoading] = useState(false);

  const compute = async () => {
    const byteLen = new TextEncoder().encode(input).length;
    const showLoader = byteLen > LOADING_THRESHOLD_BYTES;
    if (showLoader) setLoading(true);
    try {
      const result = await sha(algo, input);
      setOutput(result);
    } catch (e) {
      if (e instanceof InputTooLargeError) {
        setOutput(`Error: input exceeds 1 MB limit (${e.bytes.toLocaleString()} bytes).`);
      } else {
        setOutput(`Error: ${e instanceof Error ? e.message : "unknown error"}`);
      }
    } finally {
      setLoading(false);
    }
  };

  const clear = () => {
    setInput("");
    setOutput("");
  };

  const copy = async () => {
    const ok = await copyText(output);
    showToast(ok ? "Copied!" : "Copy failed");
  };

  return (
    <div className="tool-group">
      <div className="tool-group-header">
        <label htmlFor="algo">Algorithm:</label>
        <select
          id="algo"
          className="tool-select"
          value={algo}
          onChange={(e) => setAlgo(e.target.value as HashAlgorithm)}
        >
          {ALGORITHMS.map((a) => (
            <option key={a} value={a}>{a}</option>
          ))}
        </select>
        {algo === "MD5" && (
          <span className="tool-hint warn">
            MD5 is not cryptographically secure — use SHA-256 for security-critical work.
          </span>
        )}
      </div>

      <div className="tool-io-grid">
        <div className="tool-io-col">
          <label>Input</label>
          <textarea
            className="tool-textarea"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder="Text to hash…"
          />
          <div className="tool-actions">
            <button className="tool-btn primary" onClick={compute} disabled={loading}>
              {loading ? "Computing…" : "Compute"}
            </button>
            <button className="tool-btn danger" onClick={clear}>Clear</button>
          </div>
        </div>
        <div className="tool-io-col">
          <label>Hex digest</label>
          <div className="tool-output-box" style={{ minHeight: 60 }}>
            {loading ? "Computing…" : output || "—"}
          </div>
          <div className="tool-actions">
            <button className="tool-btn" onClick={copy} disabled={!output || loading}>Copy</button>
          </div>
        </div>
      </div>
    </div>
  );
}
