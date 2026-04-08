import { useState, useEffect } from "react";
import type { PomodoroSettings as Settings } from "../types/pomodoro";

interface PomodoroSettingsProps {
  settings: Settings;
  onSave: (s: Settings) => Promise<void>;
}

export function PomodoroSettings({ settings, onSave }: PomodoroSettingsProps) {
  const [open, setOpen] = useState(false);
  const [form, setForm] = useState(settings);

  useEffect(() => { setForm(settings); }, [settings]);

  const handleSave = async () => {
    await onSave(form);
    setOpen(false);
  };

  if (!open) {
    return (
      <button className="pomo-settings-toggle" onClick={() => setOpen(true)}>
        ⚙️ Settings
      </button>
    );
  }

  return (
    <div className="pomo-settings">
      <div className="pomo-settings-row">
        <label>Work<input type="number" min={1} max={90} value={form.work_minutes} onChange={(e) => setForm({ ...form, work_minutes: +e.target.value })} /> min</label>
        <label>Short break<input type="number" min={1} max={30} value={form.short_break_min} onChange={(e) => setForm({ ...form, short_break_min: +e.target.value })} /> min</label>
      </div>
      <div className="pomo-settings-row">
        <label>Long break<input type="number" min={1} max={60} value={form.long_break_min} onChange={(e) => setForm({ ...form, long_break_min: +e.target.value })} /> min</label>
        <label>Rounds<input type="number" min={1} max={10} value={form.rounds_per_cycle} onChange={(e) => setForm({ ...form, rounds_per_cycle: +e.target.value })} /></label>
      </div>
      <div className="pomo-settings-actions">
        <button className="btn-save" onClick={handleSave}>Save</button>
        <button className="btn-cancel" onClick={() => { setForm(settings); setOpen(false); }}>Cancel</button>
      </div>
    </div>
  );
}
