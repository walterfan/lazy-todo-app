import type { AppSettings, DisplayStyle } from "../types/settings";

interface SettingsPanelProps {
  settings: AppSettings;
  dbPath: string;
  onUpdate: (patch: Partial<AppSettings>) => Promise<void>;
}

const DISPLAY_OPTIONS: { value: DisplayStyle; label: string }[] = [
  { value: "list", label: "📄 List" },
  { value: "grid", label: "📊 Grid" },
];

export function SettingsPanel({ settings, dbPath, onUpdate }: SettingsPanelProps) {
  return (
    <div className="settings-panel">
      <section className="settings-section">
        <h3 className="settings-section-title">Display</h3>

        <div className="settings-row">
          <label>Page size</label>
          <input
            type="number"
            min={10}
            max={200}
            step={10}
            value={settings.page_size}
            onChange={(e) => onUpdate({ page_size: Number(e.target.value) || 50 })}
          />
        </div>

        <div className="settings-row">
          <label>Todo display</label>
          <div className="settings-toggle-group">
            {DISPLAY_OPTIONS.map((opt) => (
              <button
                key={opt.value}
                className={`settings-toggle ${settings.todo_display === opt.value ? "active" : ""}`}
                onClick={() => onUpdate({ todo_display: opt.value })}
              >
                {opt.label}
              </button>
            ))}
          </div>
        </div>

        <div className="settings-row">
          <label>Note display</label>
          <div className="settings-toggle-group">
            {DISPLAY_OPTIONS.map((opt) => (
              <button
                key={opt.value}
                className={`settings-toggle ${settings.note_display === opt.value ? "active" : ""}`}
                onClick={() => onUpdate({ note_display: opt.value })}
              >
                {opt.label}
              </button>
            ))}
          </div>
        </div>
      </section>

      <section className="settings-section">
        <h3 className="settings-section-title">Notes</h3>

        <div className="settings-row">
          <label>Note folder</label>
          <input
            type="text"
            value={settings.note_folder}
            onChange={(e) => onUpdate({ note_folder: e.target.value })}
            placeholder="Default"
          />
        </div>

        <div className="settings-row settings-row-vertical">
          <label>Note template</label>
          <textarea
            value={settings.note_template}
            onChange={(e) => onUpdate({ note_template: e.target.value })}
            placeholder="Default Markdown template for new notes..."
            rows={4}
          />
        </div>
      </section>

      <section className="settings-section">
        <h3 className="settings-section-title">Storage</h3>
        <div className="settings-row">
          <label>Database path</label>
          <span className="settings-value" title={dbPath}>{dbPath || "—"}</span>
        </div>
      </section>
    </div>
  );
}
