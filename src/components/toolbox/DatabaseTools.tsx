import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { copyText } from "../../utils/crypto";

interface DatabaseQueryResult {
  columns: string[];
  rows: string[][];
  row_count: number;
  truncated: boolean;
  elapsed_ms: number;
}

const PRESET_QUERIES = [
  {
    labelKey: "dbPresetTables",
    sql: "SELECT name, type FROM sqlite_master WHERE type IN ('table', 'view') ORDER BY name;",
  },
  {
    labelKey: "dbPresetTodos",
    sql: "SELECT id, title, priority, completed, deadline, created_at FROM todos ORDER BY created_at DESC LIMIT 20;",
  },
  {
    labelKey: "dbPresetNotes",
    sql: "SELECT id, title, color, updated_at FROM sticky_notes ORDER BY updated_at DESC LIMIT 20;",
  },
  {
    labelKey: "dbPresetAgentSessions",
    sql: "SELECT session_id, session_title, session_type, updated_at FROM agent_sessions ORDER BY updated_at DESC LIMIT 20;",
  },
];
const SQL_HISTORY_KEY = "lazy-todo-toolbox-sql-history";
const MAX_SQL_HISTORY = 20;

function loadSqlHistory(): string[] {
  try {
    const raw = localStorage.getItem(SQL_HISTORY_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.filter((item): item is string => typeof item === "string" && item.trim().length > 0);
  } catch {
    return [];
  }
}

function saveSqlHistory(history: string[]) {
  try {
    localStorage.setItem(SQL_HISTORY_KEY, JSON.stringify(history));
  } catch {
    // History is a convenience feature; query execution should not depend on storage.
  }
}

function showToast(msg: string) {
  const el = document.createElement("div");
  el.className = "tool-toast";
  el.textContent = msg;
  document.body.appendChild(el);
  setTimeout(() => el.remove(), 1600);
}

function resultToTsv(result: DatabaseQueryResult): string {
  const escapeCell = (value: string) => value.replace(/\t/g, " ").replace(/\r?\n/g, " ");
  return [
    result.columns.map(escapeCell).join("\t"),
    ...result.rows.map((row) => row.map(escapeCell).join("\t")),
  ].join("\n");
}

export function DatabaseTools() {
  const { t } = useTranslation();
  const [dbPath, setDbPath] = useState("");
  const [defaultDbPath, setDefaultDbPath] = useState("");
  const [sql, setSql] = useState(PRESET_QUERIES[0].sql);
  const [sqlHistory, setSqlHistory] = useState<string[]>(() => loadSqlHistory());
  const [maxRows, setMaxRows] = useState(100);
  const [result, setResult] = useState<DatabaseQueryResult | null>(null);
  const [error, setError] = useState("");
  const [running, setRunning] = useState(false);

  useEffect(() => {
    invoke<string>("get_db_path")
      .then((path) => {
        setDbPath(path);
        setDefaultDbPath(path);
      })
      .catch(() => {
        setDbPath("");
        setDefaultDbPath("");
      });
  }, []);

  const runQuery = async () => {
    setRunning(true);
    setError("");
    try {
      const data = await invoke<DatabaseQueryResult>("query_database", {
        input: { sql, db_path: dbPath, max_rows: maxRows },
      });
      setResult(data);
      rememberSql(sql);
    } catch (err) {
      setResult(null);
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setRunning(false);
    }
  };

  const rememberSql = (nextSql: string) => {
    const trimmed = nextSql.trim();
    if (!trimmed) return;
    setSqlHistory((current) => {
      const next = [trimmed, ...current.filter((item) => item !== trimmed)].slice(0, MAX_SQL_HISTORY);
      saveSqlHistory(next);
      return next;
    });
  };

  const clearSqlHistory = () => {
    setSqlHistory([]);
    saveSqlHistory([]);
  };

  const copyRows = async () => {
    if (!result) return;
    const ok = await copyText(resultToTsv(result));
    showToast(ok ? t("copiedBang") : t("copyFailedBare"));
  };

  return (
    <div className="tool-group">
      <div className="tool-group-header">
        <label>{t("database")}</label>
        <span className="tool-hint">{t("dbReadOnlyHint")}</span>
      </div>

      <div className="tool-io-col">
        <label htmlFor="database-path">{t("sqliteDbPath")}</label>
        <div className="database-path-row">
          <input
            id="database-path"
            className="tool-input"
            type="text"
            value={dbPath}
            onChange={(event) => setDbPath(event.target.value)}
            placeholder={t("sqliteDbPathPlaceholder")}
          />
          <button
            type="button"
            className="tool-btn"
            onClick={() => setDbPath(defaultDbPath)}
            disabled={!defaultDbPath || dbPath === defaultDbPath}
          >
            {t("useAppDb")}
          </button>
        </div>
      </div>

      <div className="tool-field-row">
        {PRESET_QUERIES.map((preset) => (
          <button
            type="button"
            className="tool-btn"
            key={preset.labelKey}
            onClick={() => setSql(preset.sql)}
          >
            {t(preset.labelKey)}
          </button>
        ))}
      </div>

      {sqlHistory.length > 0 && (
        <div className="database-history-row">
          <label htmlFor="database-sql-history">{t("sqlHistory")}</label>
          <select
            id="database-sql-history"
            className="tool-select"
            value=""
            onChange={(event) => {
              if (event.target.value) setSql(event.target.value);
            }}
          >
            <option value="">{t("loadFromHistory")}</option>
            {sqlHistory.map((item) => (
              <option key={item} value={item}>
                {item}
              </option>
            ))}
          </select>
          <button type="button" className="tool-btn danger" onClick={clearSqlHistory}>
            {t("clearHistory")}
          </button>
        </div>
      )}

      <div className="tool-io-col">
        <label htmlFor="database-sql">{t("sqlQuery")}</label>
        <textarea
          id="database-sql"
          className="tool-textarea database-sql-input"
          value={sql}
          onChange={(event) => setSql(event.target.value)}
          spellCheck={false}
        />
      </div>

      <div className="tool-field-row">
        <label className="database-max-rows">
          {t("maxRows")}
          <input
            className="tool-input"
            type="number"
            min={1}
            max={500}
            value={maxRows}
            onChange={(event) => setMaxRows(Number(event.target.value || 1))}
          />
        </label>
        <button type="button" className="tool-btn primary" onClick={runQuery} disabled={running}>
          {running ? t("running") : t("runQuery")}
        </button>
        <button type="button" className="tool-btn danger" onClick={() => { setResult(null); setError(""); }}>
          {t("clear")}
        </button>
        <button type="button" className="tool-btn" onClick={copyRows} disabled={!result || result.rows.length === 0}>
          {t("copyAllTsv")}
        </button>
      </div>

      {error && <div className="tool-hint error">{error}</div>}

      {result && (
        <div className="database-result">
          <div className="tool-hint">
            {t("queryReturned", { count: result.row_count, elapsed: result.elapsed_ms })}
            {result.truncated ? ` ${t("queryTruncated")}` : ""}
          </div>
          {result.columns.length === 0 ? (
            <div className="tool-output-box">{t("queryNoColumns")}</div>
          ) : result.rows.length === 0 ? (
            <div className="tool-output-box">{t("queryNoRows")}</div>
          ) : (
            <div className="database-table-wrap">
              <table className="tool-table database-result-table">
                <thead>
                  <tr>
                    {result.columns.map((column) => (
                      <th key={column}>{column}</th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {result.rows.map((row, rowIndex) => (
                    <tr key={rowIndex}>
                      {result.columns.map((column, columnIndex) => (
                        <td key={`${rowIndex}-${column}`} className="mono">
                          {row[columnIndex] ?? ""}
                        </td>
                      ))}
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
