import { useState, useEffect } from "react";
import type { Translator } from "../i18n";
import type { PomodoroSettings as Settings } from "../types/pomodoro";

const EMPTY_MILESTONES = Array.from({ length: 3 }, () => ({ name: "", deadline: "", status: "active" as const }));

function withMilestoneSlots(settings: Settings): Settings {
  const milestones = [
    ...settings.milestones.map((milestone) => ({ ...milestone })),
    ...EMPTY_MILESTONES,
  ].slice(0, 3);

  return {
    ...settings,
    milestones,
  };
}

interface PomodoroSettingsProps {
  settings: Settings;
  onSave: (s: Settings) => Promise<void>;
  t: Translator;
}

export function PomodoroSettings({ settings, onSave, t }: PomodoroSettingsProps) {
  const [open, setOpen] = useState(false);
  const [form, setForm] = useState(() => withMilestoneSlots(settings));

  useEffect(() => {
    setForm(withMilestoneSlots(settings));
  }, [settings]);

  const handleSave = async () => {
    await onSave(form);
    setOpen(false);
  };

  if (!open) {
    return (
      <button className="pomo-settings-toggle" onClick={() => setOpen(true)}>
        ⚙️ {t("settings")}
      </button>
    );
  }

  return (
    <div className="pomo-settings">
      <div className="pomo-settings-row">
        <label>{t("work")}<input type="number" min={1} max={90} value={form.work_minutes} onChange={(e) => setForm({ ...form, work_minutes: +e.target.value })} /> {t("minutes")}</label>
        <label>{t("shortBreak")}<input type="number" min={1} max={30} value={form.short_break_min} onChange={(e) => setForm({ ...form, short_break_min: +e.target.value })} /> {t("minutes")}</label>
      </div>
      <div className="pomo-settings-row">
        <label>{t("longBreak")}<input type="number" min={1} max={60} value={form.long_break_min} onChange={(e) => setForm({ ...form, long_break_min: +e.target.value })} /> {t("minutes")}</label>
        <label>{t("rounds")}<input type="number" min={1} max={10} value={form.rounds_per_cycle} onChange={(e) => setForm({ ...form, rounds_per_cycle: +e.target.value })} /></label>
      </div>
      <div className="pomo-settings-section-title">{t("milestones")}</div>
      {form.milestones.map((milestone, index) => (
        <div className="pomo-settings-row" key={`milestone-${index}`}>
          <label className="pomo-settings-text-label">
            {t("name")}
            <input
              type="text"
              maxLength={40}
              placeholder={`${t("milestones")} ${index + 1}`}
              value={milestone.name}
              onChange={(e) => {
                const milestones = [...form.milestones];
                milestones[index] = { ...milestone, name: e.target.value };
                setForm({ ...form, milestones });
              }}
            />
          </label>
          <label className="pomo-settings-date-label">
            {t("deadline")}
            <input
              type="date"
              value={milestone.deadline}
              onChange={(e) => {
                const milestones = [...form.milestones];
                milestones[index] = { ...milestone, deadline: e.target.value };
                setForm({ ...form, milestones });
              }}
            />
          </label>
        </div>
      ))}
      <div className="pomo-settings-actions">
        <button className="btn-save" onClick={handleSave}>{t("save")}</button>
        <button className="btn-cancel" onClick={() => { setForm(withMilestoneSlots(settings)); setOpen(false); }}>{t("cancel")}</button>
      </div>
    </div>
  );
}
