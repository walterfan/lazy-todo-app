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
    setEditReminderMinutes(todo.reminder_minutes_before ?? 0);
    setEditing(false);
  };

  const isRecurring = todo.recurrence !== "none";
  const reminderActive = todo.reminder_state !== "none";

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
                onChange={(e) => setEditDeadline(e.target.value)}
              />
            </label>
            <label>
              {t("repeat")}:
              <select
                value={editRecurrence}
                onChange={(e) => setEditRecurrence(e.target.value as Recurrence)}
                disabled={!editDeadline}
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
