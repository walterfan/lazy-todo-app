import { useState } from "react";
import type { CreateTodo, Recurrence } from "../types/todo";
import type { Translator } from "../i18n";

interface Props {
  onAdd: (input: CreateTodo) => Promise<void>;
  t: Translator;
}

const WEEKDAY_OPTIONS = [
  { value: 1, key: "weekdayMon" },
  { value: 2, key: "weekdayTue" },
  { value: 3, key: "weekdayWed" },
  { value: 4, key: "weekdayThu" },
  { value: 5, key: "weekdayFri" },
  { value: 6, key: "weekdaySat" },
  { value: 7, key: "weekdaySun" },
] as const;

function dateFromLocalInput(value: string): Date | null {
  if (!value) return null;
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? null : date;
}

function toLocalInputValue(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  const hours = String(date.getHours()).padStart(2, "0");
  const minutes = String(date.getMinutes()).padStart(2, "0");
  return `${year}-${month}-${day}T${hours}:${minutes}`;
}

function weekdayFromDate(date: Date): number {
  return date.getDay() === 0 ? 7 : date.getDay();
}

function lastDayOfMonth(year: number, monthIndex: number): number {
  return new Date(year, monthIndex + 1, 0).getDate();
}

function withWeekday(value: string, weekday: number): string {
  const date = dateFromLocalInput(value);
  if (!date) return value;
  const current = weekdayFromDate(date);
  let days = (weekday - current + 7) % 7;
  if (days === 0) days = 7;
  date.setDate(date.getDate() + days);
  return toLocalInputValue(date);
}

function withMonthDay(value: string, monthDay: number): string {
  const date = dateFromLocalInput(value);
  if (!date) return value;
  const day = Math.max(1, Math.min(31, monthDay));
  date.setDate(Math.min(day, lastDayOfMonth(date.getFullYear(), date.getMonth())));
  return toLocalInputValue(date);
}

export function AddTodo({ onAdd, t }: Props) {
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [priority, setPriority] = useState(2);
  const [deadline, setDeadline] = useState("");
  const [recurrence, setRecurrence] = useState<Recurrence>("none");
  const [recurrenceWeekday, setRecurrenceWeekday] = useState(1);
  const [recurrenceMonthDay, setRecurrenceMonthDay] = useState(1);
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
      recurrence_weekday: deadline && recurrence === "weekly" ? recurrenceWeekday : undefined,
      recurrence_month_day: deadline && recurrence === "monthly" ? recurrenceMonthDay : undefined,
      reminder_minutes_before: deadline ? reminderMinutes : 0,
    });

    setTitle("");
    setDescription("");
    setPriority(2);
    setDeadline("");
    setRecurrence("none");
    setRecurrenceWeekday(1);
    setRecurrenceMonthDay(1);
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
                onChange={(e) => {
                  const value = e.target.value;
                  setDeadline(value);
                  const date = dateFromLocalInput(value);
                  if (date) {
                    setRecurrenceWeekday(weekdayFromDate(date));
                    setRecurrenceMonthDay(date.getDate());
                  }
                }}
              />
            </label>
            <label>
              {t("repeat")}:
              <select
                value={recurrence}
                onChange={(e) => {
                  const value = e.target.value as Recurrence;
                  setRecurrence(value);
                  const date = dateFromLocalInput(deadline);
                  if (date) {
                    setRecurrenceWeekday(weekdayFromDate(date));
                    setRecurrenceMonthDay(date.getDate());
                  }
                }}
                disabled={!deadline}
              >
                <option value="none">{t("repeatNone")}</option>
                <option value="daily">{t("repeatDaily")}</option>
                <option value="weekly">{t("repeatWeekly")}</option>
                <option value="monthly">{t("repeatMonthly")}</option>
                <option value="yearly">{t("repeatYearly")}</option>
              </select>
            </label>
            {recurrence === "weekly" && (
              <label>
                {t("repeatWeekday")}:
                <select
                  value={recurrenceWeekday}
                  onChange={(e) => {
                    const weekday = Number(e.target.value);
                    setRecurrenceWeekday(weekday);
                    setDeadline(withWeekday(deadline, weekday));
                  }}
                  disabled={!deadline}
                >
                  {WEEKDAY_OPTIONS.map((day) => (
                    <option key={day.value} value={day.value}>
                      {t(day.key)}
                    </option>
                  ))}
                </select>
              </label>
            )}
            {recurrence === "monthly" && (
              <label>
                {t("repeatMonthDay")}:
                <input
                  type="number"
                  min={1}
                  max={31}
                  value={recurrenceMonthDay}
                  onChange={(e) => {
                    const day = Math.max(1, Math.min(31, Number(e.target.value) || 1));
                    setRecurrenceMonthDay(day);
                    setDeadline(withMonthDay(deadline, day));
                  }}
                  disabled={!deadline}
                />
              </label>
            )}
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
