import { useState } from "react";
import type { CreateTodo, Recurrence } from "../types/todo";
import type { Translator } from "../i18n";

interface Props {
  onAdd: (input: CreateTodo) => Promise<void>;
  t: Translator;
}

export function AddTodo({ onAdd, t }: Props) {
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [priority, setPriority] = useState(2);
  const [deadline, setDeadline] = useState("");
  const [recurrence, setRecurrence] = useState<Recurrence>("none");
  const [reminderMinutes, setReminderMinutes] = useState(0);
  const [expanded, setExpanded] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!title.trim()) return;

    await onAdd({
      title: title.trim(),
      description: description.trim() || undefined,
      priority,
      deadline: deadline || undefined,
      recurrence: deadline ? recurrence : "none",
      reminder_minutes_before: deadline ? reminderMinutes : 0,
    });

    setTitle("");
    setDescription("");
    setPriority(2);
    setDeadline("");
    setRecurrence("none");
    setReminderMinutes(0);
    setExpanded(false);
  };

  return (
    <form onSubmit={handleSubmit} className="add-todo">
      <div className="add-todo-main">
        <input
          type="text"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          placeholder={t("addTaskPlaceholder")}
          className="add-todo-input"
        />
        <button
          type="button"
          onClick={() => setExpanded(!expanded)}
          className="btn-expand"
          title={t("moreOptions")}
        >
          {expanded ? "▲" : "▼"}
        </button>
        <button type="submit" className="btn-add" disabled={!title.trim()}>
          {t("addTask")}
        </button>
      </div>

      {expanded && (
        <div className="add-todo-extra">
          <input
            type="text"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder={t("taskDescriptionPlaceholder")}
            className="add-todo-desc"
          />
          <div className="add-todo-options">
            <label>
              {t("priority")}:
              <select value={priority} onChange={(e) => setPriority(Number(e.target.value))}>
                <option value={1}>🔴 {t("high")}</option>
                <option value={2}>🟡 {t("medium")}</option>
                <option value={3}>🟢 {t("low")}</option>
              </select>
            </label>
            <label>
              {t("deadline")}:
              <input
                type="datetime-local"
                value={deadline}
                onChange={(e) => setDeadline(e.target.value)}
              />
            </label>
            <label>
              {t("repeat")}:
              <select
                value={recurrence}
                onChange={(e) => setRecurrence(e.target.value as Recurrence)}
                disabled={!deadline}
              >
                <option value="none">{t("repeatNone")}</option>
                <option value="daily">{t("repeatDaily")}</option>
                <option value="weekly">{t("repeatWeekly")}</option>
                <option value="monthly">{t("repeatMonthly")}</option>
                <option value="yearly">{t("repeatYearly")}</option>
              </select>
            </label>
            <label>
              {t("reminder")}:
              <select
                value={reminderMinutes}
                onChange={(e) => setReminderMinutes(Number(e.target.value))}
                disabled={!deadline}
              >
                <option value={0}>{t("reminderNone")}</option>
                <option value={5}>{t("reminder5m")}</option>
                <option value={10}>{t("reminder10m")}</option>
                <option value={15}>{t("reminder15m")}</option>
                <option value={30}>{t("reminder30m")}</option>
                <option value={60}>{t("reminder1h")}</option>
                <option value={1440}>{t("reminder1d")}</option>
              </select>
            </label>
          </div>
        </div>
      )}
    </form>
  );
}
