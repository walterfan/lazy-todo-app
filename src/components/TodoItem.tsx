import { useState } from "react";
import type { Recurrence, Todo, UpdateTodo } from "../types/todo";
import { useCountdown } from "../hooks/useCountdown";
import type { Translator } from "../i18n";

interface Props {
  todo: Todo;
  onToggle: (id: number) => Promise<void>;
  onUpdate: (input: UpdateTodo) => Promise<void>;
  onDelete: (id: number) => Promise<void>;
  t: Translator;
}

const PRIORITY_LABELS: Record<number, { emoji: string; text: string }> = {
  1: { emoji: "🔴", text: "High" },
  2: { emoji: "🟡", text: "Medium" },
  3: { emoji: "🟢", text: "Low" },
};

const RECURRENCE_LABEL_KEY: Record<Recurrence, string> = {
  none: "repeatNone",
  daily: "repeatDaily",
  weekly: "repeatWeekly",
  monthly: "repeatMonthly",
  yearly: "repeatYearly",
};

const REMINDER_OPTIONS = [0, 5, 10, 15, 30, 60, 1440] as const;
const WEEKDAY_OPTIONS = [
  { value: 1, key: "weekdayMon" },
  { value: 2, key: "weekdayTue" },
  { value: 3, key: "weekdayWed" },
  { value: 4, key: "weekdayThu" },
  { value: 5, key: "weekdayFri" },
  { value: 6, key: "weekdaySat" },
  { value: 7, key: "weekdaySun" },
] as const;

function reminderOptionLabel(minutes: number, t: Translator): string {
  switch (minutes) {
    case 5:
      return t("reminder5m");
    case 10:
      return t("reminder10m");
    case 15:
      return t("reminder15m");
    case 30:
      return t("reminder30m");
    case 60:
      return t("reminder1h");
    case 1440:
      return t("reminder1d");
    default:
      return t("reminderNone");
  }
}

function formatDeadline(deadline: string): string {
  const d = new Date(deadline);
  const month = d.getMonth() + 1;
  const day = d.getDate();
  const hour = d.getHours().toString().padStart(2, "0");
  const min = d.getMinutes().toString().padStart(2, "0");
  return `${month}/${day} ${hour}:${min}`;
}

function dateFromLocalInput(value: string): Date | null {
  if (!value) return null;
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? null : date;
}

function toLocalInputValue(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  const hour = String(date.getHours()).padStart(2, "0");
  const minute = String(date.getMinutes()).padStart(2, "0");
  return `${year}-${month}-${day}T${hour}:${minute}`;
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

function scheduleLabel(todo: Todo, t: Translator): string {
  if (todo.recurrence === "weekly" && todo.recurrence_weekday) {
    const weekday = WEEKDAY_OPTIONS.find((day) => day.value === todo.recurrence_weekday);
    return weekday ? `${t("on")} ${t(weekday.key)}` : "";
  }
  if (todo.recurrence === "monthly" && todo.recurrence_month_day) {
    return `${t("onDay")} ${todo.recurrence_month_day}`;
  }
  return "";
}

function CompleteIcon() {
  return (
    <svg viewBox="0 0 16 16" aria-hidden="true">
      <path
        d="M3.5 8.5 6.5 11.5 12.5 4.5"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.8"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

export function TodoItem({ todo, onToggle, onUpdate, onDelete, t }: Props) {
  const countdown = useCountdown(todo.deadline);
  const pri = PRIORITY_LABELS[todo.priority] ?? PRIORITY_LABELS[2];
  const [editing, setEditing] = useState(false);
  const [editTitle, setEditTitle] = useState(todo.title);
  const [editDesc, setEditDesc] = useState(todo.description);
  const [editPriority, setEditPriority] = useState(todo.priority);
  const [editDeadline, setEditDeadline] = useState(todo.deadline ?? "");
  const [editRecurrence, setEditRecurrence] = useState<Recurrence>(todo.recurrence);
  const [editRecurrenceWeekday, setEditRecurrenceWeekday] = useState(
    todo.recurrence_weekday ?? (todo.deadline ? weekdayFromDate(new Date(todo.deadline)) : 1),
  );
  const [editRecurrenceMonthDay, setEditRecurrenceMonthDay] = useState(
    todo.recurrence_month_day ?? (todo.deadline ? new Date(todo.deadline).getDate() : 1),
  );
  const [editReminderMinutes, setEditReminderMinutes] = useState(todo.reminder_minutes_before ?? 0);

  const handleSave = async () => {
    await onUpdate({
      id: todo.id,
      title: editTitle.trim() || undefined,
      description: editDesc.trim(),
      priority: editPriority,
      deadline: editDeadline || undefined,
      clear_deadline: !editDeadline,
      recurrence: editDeadline ? editRecurrence : "none",
      recurrence_weekday: editDeadline && editRecurrence === "weekly" ? editRecurrenceWeekday : undefined,
      recurrence_month_day: editDeadline && editRecurrence === "monthly" ? editRecurrenceMonthDay : undefined,
      reminder_minutes_before: editDeadline ? editReminderMinutes : 0,
    });
    setEditing(false);
  };

  const handleCancel = () => {
    setEditTitle(todo.title);
    setEditDesc(todo.description);
    setEditPriority(todo.priority);
    setEditDeadline(todo.deadline ?? "");
    setEditRecurrence(todo.recurrence);
    setEditRecurrenceWeekday(
      todo.recurrence_weekday ?? (todo.deadline ? weekdayFromDate(new Date(todo.deadline)) : 1),
    );
    setEditRecurrenceMonthDay(
      todo.recurrence_month_day ?? (todo.deadline ? new Date(todo.deadline).getDate() : 1),
    );
    setEditReminderMinutes(todo.reminder_minutes_before ?? 0);
    setEditing(false);
  };

  const isRecurring = todo.recurrence !== "none";
  const reminderActive = todo.reminder_state !== "none";
  const recurringSchedule = scheduleLabel(todo, t);

  if (editing) {
    return (
      <div className={`todo-item editing priority-${editPriority}`}>
        <div className="edit-form">
          <input
            type="text"
            value={editTitle}
            onChange={(e) => setEditTitle(e.target.value)}
            className="edit-input"
            placeholder={t("taskTitlePlaceholder")}
            autoFocus
            onKeyDown={(e) => {
              if (e.key === "Enter") handleSave();
              if (e.key === "Escape") handleCancel();
            }}
          />
          <input
            type="text"
            value={editDesc}
            onChange={(e) => setEditDesc(e.target.value)}
            className="edit-input edit-input-desc"
            placeholder={t("taskDescriptionPlaceholder")}
          />
          <div className="edit-options">
            <label>
              {t("priority")}:
              <select value={editPriority} onChange={(e) => setEditPriority(Number(e.target.value))}>
                <option value={1}>🔴 {t("high")}</option>
                <option value={2}>🟡 {t("medium")}</option>
                <option value={3}>🟢 {t("low")}</option>
              </select>
            </label>
            <label>
              {t("deadline")}:
              <input
                type="datetime-local"
                value={editDeadline}
                onChange={(e) => {
                  const value = e.target.value;
                  setEditDeadline(value);
                  const date = dateFromLocalInput(value);
                  if (date) {
                    setEditRecurrenceWeekday(weekdayFromDate(date));
                    setEditRecurrenceMonthDay(date.getDate());
                  }
                }}
              />
            </label>
            <label>
              {t("repeat")}:
              <select
                value={editRecurrence}
                onChange={(e) => {
                  const value = e.target.value as Recurrence;
                  setEditRecurrence(value);
                  const date = dateFromLocalInput(editDeadline);
                  if (date) {
                    setEditRecurrenceWeekday(weekdayFromDate(date));
                    setEditRecurrenceMonthDay(date.getDate());
                  }
                }}
                disabled={!editDeadline}
              >
                <option value="none">{t("repeatNone")}</option>
                <option value="daily">{t("repeatDaily")}</option>
                <option value="weekly">{t("repeatWeekly")}</option>
                <option value="monthly">{t("repeatMonthly")}</option>
                <option value="yearly">{t("repeatYearly")}</option>
              </select>
            </label>
            {editRecurrence === "weekly" && (
              <label>
                {t("repeatWeekday")}:
                <select
                  value={editRecurrenceWeekday}
                  onChange={(e) => {
                    const weekday = Number(e.target.value);
                    setEditRecurrenceWeekday(weekday);
                    setEditDeadline(withWeekday(editDeadline, weekday));
                  }}
                  disabled={!editDeadline}
                >
                  {WEEKDAY_OPTIONS.map((day) => (
                    <option key={day.value} value={day.value}>
                      {t(day.key)}
                    </option>
                  ))}
                </select>
              </label>
            )}
            {editRecurrence === "monthly" && (
              <label>
                {t("repeatMonthDay")}:
                <input
                  type="number"
                  min={1}
                  max={31}
                  value={editRecurrenceMonthDay}
                  onChange={(e) => {
                    const day = Math.max(1, Math.min(31, Number(e.target.value) || 1));
                    setEditRecurrenceMonthDay(day);
                    setEditDeadline(withMonthDay(editDeadline, day));
                  }}
                  disabled={!editDeadline}
                />
              </label>
            )}
            <label>
              {t("reminder")}:
              <select
                value={editReminderMinutes}
                onChange={(e) => setEditReminderMinutes(Number(e.target.value))}
                disabled={!editDeadline}
              >
                {REMINDER_OPTIONS.map((minutes) => (
                  <option key={minutes} value={minutes}>
                    {reminderOptionLabel(minutes, t)}
                  </option>
                ))}
              </select>
            </label>
          </div>
          <div className="edit-actions">
            <button onClick={handleSave} className="btn-save">{t("save")}</button>
            <button onClick={handleCancel} className="btn-cancel">{t("cancel")}</button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={`todo-item ${todo.completed ? "completed" : ""} priority-${todo.priority}`}>
      <div className="todo-left">
        <div className="todo-content">
          <div className="todo-title">
            <span className="priority-badge" title={`${t("priority")}: ${pri.text}`}>
              {pri.emoji}
            </span>
            <span>{todo.title}</span>
          </div>
          {(isRecurring || reminderActive) && (
            <div className="todo-meta-row">
              {isRecurring && (
                <span className="todo-badge recurrence-badge">
                  {t("repeats")} {t(RECURRENCE_LABEL_KEY[todo.recurrence])}
                  {recurringSchedule ? ` ${recurringSchedule}` : ""}
                </span>
              )}
              {reminderActive && (
                <span className={`todo-badge reminder-badge reminder-${todo.reminder_state}`}>
                  {todo.reminder_state === "overdue"
                    ? t("overdue")
                    : todo.reminder_state === "due"
                      ? t("reminderDue")
                      : todo.reminder_state === "reminded"
                        ? t("reminded")
                        : t("reminderUpcoming")}
                </span>
              )}
            </div>
          )}
          {todo.description && (
            <div className="todo-desc">{todo.description}</div>
          )}
          {todo.deadline && (
            <div className="todo-deadline-row">
              <span className="todo-deadline-date">
                📅 {formatDeadline(todo.deadline)}
              </span>
              {countdown && (
                <span className={`todo-countdown ${countdown.overdue ? "overdue" : ""} ${countdown.urgent ? "urgent" : ""}`}>
                  ⏱ {countdown.label}
                </span>
              )}
            </div>
          )}
        </div>
      </div>
      <div className="todo-actions">
        <button
          onClick={() => onToggle(todo.id)}
          className="btn-complete"
          title={isRecurring ? t("completeOccurrence") : t("completeTask")}
          aria-label={isRecurring ? t("completeOccurrence") : t("completeTask")}
        >
          <CompleteIcon />
        </button>
        <button onClick={() => setEditing(true)} className="btn-edit" title={t("editTask")}>
          ✎
        </button>
        <button onClick={() => onDelete(todo.id)} className="btn-delete" title={t("deleteTask")}>
          ✕
        </button>
      </div>
    </div>
  );
}
