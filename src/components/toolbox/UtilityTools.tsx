import { useState } from "react";
import { useTranslation } from "react-i18next";
import { evaluateCalculation, CalculationResult } from "../../utils/calculator";
import { buildCronExpression, CronBuildInput, CronExplainResult, explainCronExpression } from "../../utils/cron";
import { copyText } from "../../utils/crypto";

type UtilityMode = "calculator" | "cron";
type CronBuilderKind = CronBuildInput["kind"];

const CALCULATOR_EXAMPLES = ["1 + 2 * (3 + 4)", "sqrt(81) + max(1, 5)", "sin(pi / 2)", "10 / 3"];

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

export function UtilityTools() {
  const { t } = useTranslation();
  const [mode, setMode] = useState<UtilityMode>("calculator");

  const [calcExpression, setCalcExpression] = useState("1 + 2 * (3 + 4)");
  const [calcResult, setCalcResult] = useState<CalculationResult | null>(null);

  const [cronExpression, setCronExpression] = useState("*/15 9-17 * * 1-5");
  const [cronResult, setCronResult] = useState<CronExplainResult | null>(null);
  const [builderKind, setBuilderKind] = useState<CronBuilderKind>("everyMinutes");
  const [includeSeconds, setIncludeSeconds] = useState(false);
  const [everyMinutes, setEveryMinutes] = useState(5);
  const [minute, setMinute] = useState(0);
  const [hour, setHour] = useState(9);
  const [dayOfWeek, setDayOfWeek] = useState(1);
  const [dayOfMonth, setDayOfMonth] = useState(1);

  const calculate = () => {
    setCalcResult(evaluateCalculation(calcExpression));
  };

  const explainCron = () => {
    setCronResult(explainCronExpression(cronExpression));
  };

  const clearCalculator = () => {
    setCalcExpression("");
    setCalcResult(null);
  };

  const clearCron = () => {
    setCronExpression("");
    setCronResult(null);
  };

  const buildCron = () => {
    const base = { includeSeconds };
    let input: CronBuildInput;
    if (builderKind === "everyMinutes") input = { ...base, kind: builderKind, minutes: everyMinutes };
    else if (builderKind === "hourly") input = { ...base, kind: builderKind, minute };
    else if (builderKind === "daily") input = { ...base, kind: builderKind, hour, minute };
    else if (builderKind === "weekly") input = { ...base, kind: builderKind, dayOfWeek, hour, minute };
    else input = { ...base, kind: builderKind, dayOfMonth, hour, minute };

    try {
      const expression = buildCronExpression(input);
      setCronExpression(expression);
      setCronResult(explainCronExpression(expression));
    } catch (err) {
      setCronResult({
        ok: false,
        error: err instanceof Error ? err.message : t("invalidCronBuilderInput"),
      });
    }
  };

  return (
    <div className="tool-group">
      <div className="tool-group-header">
        <label htmlFor="utility-mode">{t("tool")}:</label>
        <select
          id="utility-mode"
          className="tool-select"
          value={mode}
          onChange={(event) => setMode(event.target.value as UtilityMode)}
        >
          <option value="calculator">{t("calculator")}</option>
          <option value="cron">{t("cronExpression")}</option>
        </select>
      </div>

      {mode === "calculator" && (
        <div className="tool-io-grid">
          <div className="tool-io-col">
            <label htmlFor="calculator-expression">{t("expression")}</label>
            <textarea
              id="calculator-expression"
              className="tool-textarea"
              value={calcExpression}
              onChange={(event) => setCalcExpression(event.target.value)}
              placeholder={t("calculatorInputPlaceholder")}
              spellCheck={false}
            />
            <div className="tool-actions">
              <button type="button" className="tool-btn primary" onClick={calculate}>
                {t("calculate")}
              </button>
              <button type="button" className="tool-btn danger" onClick={clearCalculator}>
                {t("clear")}
              </button>
            </div>
            <div className="tool-field-row">
              <span className="tool-hint">{t("examples")}:</span>
              {CALCULATOR_EXAMPLES.map((example) => (
                <button
                  type="button"
                  className="tool-btn"
                  key={example}
                  onClick={() => {
                    setCalcExpression(example);
                    setCalcResult(null);
                  }}
                >
                  {example}
                </button>
              ))}
            </div>
          </div>
          <div className="tool-io-col">
            <label>{t("result")}</label>
            <div className={`tool-output-box ${calcResult && !calcResult.ok ? "invalid" : ""}`}>
              {!calcResult ? "—" : calcResult.ok ? calcResult.display : calcResult.error}
            </div>
            <div className="tool-actions">
              <button
                type="button"
                className="tool-btn"
                onClick={() => calcResult?.ok && doCopy(calcResult.display, t("copiedBang"), t("copyFailedBare"))}
                disabled={!calcResult?.ok}
              >
                {t("copyResult")}
              </button>
            </div>
          </div>
        </div>
      )}

      {mode === "cron" && (
        <div className="tool-group">
          <div className="tool-io-grid">
            <div className="tool-io-col">
              <label htmlFor="cron-expression-input">{t("cronExpression")}</label>
              <textarea
                id="cron-expression-input"
                className="tool-textarea"
                value={cronExpression}
                onChange={(event) => {
                  setCronExpression(event.target.value);
                  setCronResult(null);
                }}
                placeholder={t("cronInputPlaceholder")}
                spellCheck={false}
              />
              <div className="tool-actions">
                <button type="button" className="tool-btn primary" onClick={explainCron}>
                  {t("explain")}
                </button>
                <button type="button" className="tool-btn" onClick={() => doCopy(cronExpression, t("copiedBang"), t("copyFailedBare"))}>
                  {t("copyExpression")}
                </button>
                <button type="button" className="tool-btn danger" onClick={clearCron}>
                  {t("clear")}
                </button>
              </div>
              <span className="tool-hint">{t("cronFormatHint")}</span>
            </div>

            <div className="tool-io-col">
              <label>{t("meaning")}</label>
              {!cronResult ? (
                <div className="tool-output-box">—</div>
              ) : cronResult.ok ? (
                <div className="tool-results-panel">
                  <span className="k">{t("description")}:</span>
                  <span className="v">{cronResult.description}</span>
                  <span className="k">{t("cronFormat")}:</span>
                  <span className="v">{cronResult.format === "unix5" ? t("unixFiveField") : t("sixFieldWithSeconds")}</span>
                  <span className="k">{t("normalizedExpression")}:</span>
                  <span className="v">{cronResult.normalized}</span>
                  <span className="k">{t("parserExpression")}:</span>
                  <span className="v">{cronResult.parserExpression}</span>
                </div>
              ) : (
                <div className="tool-output-box invalid">{cronResult.error}</div>
              )}
            </div>
          </div>

          {cronResult?.ok && (
            <div className="tool-group">
              <div className="tool-group-header">
                <label>{t("nextRuns")}</label>
                <span className="tool-hint">{t("localTime")}</span>
              </div>
              <table className="tool-table">
                <thead>
                  <tr>
                    <th>#</th>
                    <th>{t("datetime")}</th>
                  </tr>
                </thead>
                <tbody>
                  {cronResult.nextRuns.map((run, index) => (
                    <tr key={`${run}-${index}`}>
                      <td>{index + 1}</td>
                      <td className="mono">{run}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          <div className="tool-group">
            <div className="tool-group-header">
              <label htmlFor="cron-builder-kind">{t("cronBuilder")}</label>
              <select
                id="cron-builder-kind"
                className="tool-select"
                value={builderKind}
                onChange={(event) => setBuilderKind(event.target.value as CronBuilderKind)}
              >
                <option value="everyMinutes">{t("everyNMinutes")}</option>
                <option value="hourly">{t("hourlyAtMinute")}</option>
                <option value="daily">{t("dailyAtTime")}</option>
                <option value="weekly">{t("weeklyAtTime")}</option>
                <option value="monthly">{t("monthlyAtTime")}</option>
              </select>
              <label className="tool-checkbox">
                <input type="checkbox" checked={includeSeconds} onChange={(event) => setIncludeSeconds(event.target.checked)} />
                {t("includeSeconds")}
              </label>
            </div>

            <div className="tool-field-row">
              {builderKind === "everyMinutes" && (
                <label>
                  {t("builderMinutes")}
                  <input
                    type="number"
                    min={1}
                    max={59}
                    className="tool-input"
                    value={everyMinutes}
                    onChange={(event) => setEveryMinutes(parseInt(event.target.value || "1", 10))}
                  />
                </label>
              )}
              {builderKind !== "everyMinutes" && (
                <label>
                  {t("minute")}
                  <input
                    type="number"
                    min={0}
                    max={59}
                    className="tool-input"
                    value={minute}
                    onChange={(event) => setMinute(parseInt(event.target.value || "0", 10))}
                  />
                </label>
              )}
              {(builderKind === "daily" || builderKind === "weekly" || builderKind === "monthly") && (
                <label>
                  {t("hour")}
                  <input
                    type="number"
                    min={0}
                    max={23}
                    className="tool-input"
                    value={hour}
                    onChange={(event) => setHour(parseInt(event.target.value || "0", 10))}
                  />
                </label>
              )}
              {builderKind === "weekly" && (
                <label>
                  {t("dayOfWeek")}
                  <select className="tool-select" value={dayOfWeek} onChange={(event) => setDayOfWeek(parseInt(event.target.value, 10))}>
                    <option value={0}>{t("sunday")}</option>
                    <option value={1}>{t("monday")}</option>
                    <option value={2}>{t("tuesday")}</option>
                    <option value={3}>{t("wednesday")}</option>
                    <option value={4}>{t("thursday")}</option>
                    <option value={5}>{t("friday")}</option>
                    <option value={6}>{t("saturday")}</option>
                  </select>
                </label>
              )}
              {builderKind === "monthly" && (
                <label>
                  {t("dayOfMonth")}
                  <input
                    type="number"
                    min={1}
                    max={31}
                    className="tool-input"
                    value={dayOfMonth}
                    onChange={(event) => setDayOfMonth(parseInt(event.target.value || "1", 10))}
                  />
                </label>
              )}
              <button type="button" className="tool-btn primary" onClick={buildCron}>
                {t("applyBuilder")}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
