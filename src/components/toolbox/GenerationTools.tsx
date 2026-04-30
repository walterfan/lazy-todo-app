import { useState } from "react";
import { useTranslation } from "react-i18next";
import { copyText } from "../../utils/crypto";

type Mode = "uuid" | "random" | "password";

const LOWER = "abcdefghijklmnopqrstuvwxyz";
const UPPER = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const DIGITS = "0123456789";
const SYMBOLS = "!@#$%^&*()-_=+[]{};:,.<>?/";

function randIndex(max: number): number {
  const arr = new Uint32Array(1);
  crypto.getRandomValues(arr);
  return arr[0] % max;
}

function randomFromCharset(charset: string, length: number): string {
  const buf = new Uint32Array(length);
  crypto.getRandomValues(buf);
  let out = "";
  for (let i = 0; i < length; i++) out += charset[buf[i] % charset.length];
  return out;
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

export function GenerationTools() {
  const { t } = useTranslation();
  const [mode, setMode] = useState<Mode>("uuid");

  // UUID
  const [uuidOut, setUuidOut] = useState("");

  // Random string
  const [rsLen, setRsLen] = useState(16);
  const [rsLetters, setRsLetters] = useState(true);
  const [rsDigits, setRsDigits] = useState(true);
  const [rsSymbols, setRsSymbols] = useState(false);
  const [rsOut, setRsOut] = useState("");

  // Password
  const [pwLen, setPwLen] = useState(16);
  const [pwUpper, setPwUpper] = useState(true);
  const [pwDigits, setPwDigits] = useState(true);
  const [pwSymbols, setPwSymbols] = useState(true);
  const [pwOut, setPwOut] = useState("");

  const generateUuid = () => setUuidOut(crypto.randomUUID());

  const generateRandom = () => {
    const charset = [rsLetters ? LOWER + UPPER : "", rsDigits ? DIGITS : "", rsSymbols ? SYMBOLS : ""].join("");
    if (!charset) {
      setRsOut("Error: pick at least one character class");
      return;
    }
    if (rsLen < 1 || rsLen > 4096) {
      setRsOut("Error: length must be between 1 and 4096");
      return;
    }
    setRsOut(randomFromCharset(charset, rsLen));
  };

  const generatePassword = () => {
    if (pwLen < 8 || pwLen > 64) {
      setPwOut("Error: length must be between 8 and 64");
      return;
    }
    const pools: string[] = [LOWER];
    if (pwUpper) pools.push(UPPER);
    if (pwDigits) pools.push(DIGITS);
    if (pwSymbols) pools.push(SYMBOLS);
    // Guarantee at least one from each enabled class.
    const required = pools.map((p) => p[randIndex(p.length)]);
    const allChars = pools.join("");
    const remaining = randomFromCharset(allChars, pwLen - required.length);
    const combined = (required.join("") + remaining).split("");
    // Fisher-Yates shuffle with crypto RNG.
    for (let i = combined.length - 1; i > 0; i--) {
      const j = randIndex(i + 1);
      [combined[i], combined[j]] = [combined[j], combined[i]];
    }
    setPwOut(combined.join(""));
  };

  return (
    <div className="tool-group">
      <div className="tool-group-header">
        <label htmlFor="gen-mode">{t("generator")}:</label>
        <select id="gen-mode" className="tool-select" value={mode} onChange={(e) => setMode(e.target.value as Mode)}>
          <option value="uuid">UUID v4</option>
          <option value="random">{t("randomString")}</option>
          <option value="password">{t("password")}</option>
        </select>
      </div>

      {mode === "uuid" && (
        <div className="tool-io-grid">
          <div className="tool-io-col">
            <div className="tool-actions">
              <button className="tool-btn primary" onClick={generateUuid}>{t("generateUuid")}</button>
              <button className="tool-btn danger" onClick={() => setUuidOut("")}>{t("clear")}</button>
            </div>
          </div>
          <div className="tool-io-col">
            <label>{t("result")}</label>
            <div className="tool-output-box">{uuidOut || "—"}</div>
            <div className="tool-actions">
              <button className="tool-btn" onClick={() => doCopy(uuidOut, t("copiedBang"), t("copyFailedBare"))} disabled={!uuidOut}>{t("copyToClipboard")}</button>
            </div>
          </div>
        </div>
      )}

      {mode === "random" && (
        <div className="tool-io-grid">
          <div className="tool-io-col">
            <label>{t("length")}</label>
            <input
              type="number"
              min={1}
              max={4096}
              className="tool-input"
              value={rsLen}
              onChange={(e) => setRsLen(parseInt(e.target.value || "0", 10))}
            />
            <div className="tool-field-row">
              <label className="tool-checkbox">
                <input type="checkbox" checked={rsLetters} onChange={(e) => setRsLetters(e.target.checked)} /> {t("letters")}
              </label>
              <label className="tool-checkbox">
                <input type="checkbox" checked={rsDigits} onChange={(e) => setRsDigits(e.target.checked)} /> {t("digits")}
              </label>
              <label className="tool-checkbox">
                <input type="checkbox" checked={rsSymbols} onChange={(e) => setRsSymbols(e.target.checked)} /> {t("symbols")}
              </label>
            </div>
            <div className="tool-actions">
              <button className="tool-btn primary" onClick={generateRandom}>{t("generate")}</button>
              <button className="tool-btn danger" onClick={() => setRsOut("")}>{t("clear")}</button>
            </div>
          </div>
          <div className="tool-io-col">
            <label>{t("result")}</label>
            <div className="tool-output-box">{rsOut || "—"}</div>
            <div className="tool-actions">
              <button className="tool-btn" onClick={() => doCopy(rsOut, t("copiedBang"), t("copyFailedBare"))} disabled={!rsOut}>{t("copyToClipboard")}</button>
            </div>
          </div>
        </div>
      )}

      {mode === "password" && (
        <div className="tool-io-grid">
          <div className="tool-io-col">
            <label>{t("length")}: {pwLen}</label>
            <input
              type="range"
              min={8}
              max={64}
              value={pwLen}
              onChange={(e) => setPwLen(parseInt(e.target.value, 10))}
            />
            <div className="tool-field-row">
              <label className="tool-checkbox">
                <input type="checkbox" checked={pwUpper} onChange={(e) => setPwUpper(e.target.checked)} /> {t("uppercase")}
              </label>
              <label className="tool-checkbox">
                <input type="checkbox" checked={pwDigits} onChange={(e) => setPwDigits(e.target.checked)} /> {t("digits")}
              </label>
              <label className="tool-checkbox">
                <input type="checkbox" checked={pwSymbols} onChange={(e) => setPwSymbols(e.target.checked)} /> {t("symbols")}
              </label>
            </div>
            <div className="tool-actions">
              <button className="tool-btn primary" onClick={generatePassword}>{t("generatePassword")}</button>
              <button className="tool-btn danger" onClick={() => setPwOut("")}>{t("clear")}</button>
            </div>
          </div>
          <div className="tool-io-col">
            <label>{t("result")}</label>
            <div className="tool-output-box">{pwOut || "—"}</div>
            <div className="tool-actions">
              <button className="tool-btn" onClick={() => doCopy(pwOut, t("copiedBang"), t("copyFailedBare"))} disabled={!pwOut}>{t("copyToClipboard")}</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
