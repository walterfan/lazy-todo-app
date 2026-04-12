import { useState } from "react";
import type { Todo, UpdateTodo } from "../types/todo";
import { useCountdown } from "../hooks/useCountdown";

interface Props {
  todo: Todo;
  onToggle: (id: number) => Promise<void>;
  onUpdate: (input: UpdateTodo) => Promise<void>;
  onDelete: (id: number) => Promise<void>;
}

const PRIORITY_LABELS: Record<number, { emoji: string; text: string }> = {
  1: { emoji: "🔴", text: "High" },
  2: { emoji: "🟡", text: "Medium" },
  3: { emoji: "🟢", text: "Low" },
};

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

export function TodoItem({ todo, onToggle, onUpdate, onDelete }: Props) {
  const countdown = useCountdown(todo.deadline);
  const pri = PRIORITY_LABELS[todo.priority] ?? PRIORITY_LABELS[2];
  const [editing, setEditing] = useState(false);
  const [editTitle, setEditTitle] = useState(todo.title);
  const [editDesc, setEditDesc] = useState(todo.description);
  const [editPriority, setEditPriority] = useState(todo.priority);
  const [editDeadline, setEditDeadline] = useState(todo.deadline ?? "");

  const handleSave = async () => {
    await onUpdate({
      id: todo.id,
      title: editTitle.trim() || undefined,
      description: editDesc.trim(),
      priority: editPriority,
      deadline: editDeadline || undefined,
    });
    setEditing(false);
  };

  const handleCancel = () => {
    setEditTitle(todo.title);
    setEditDesc(todo.description);
    setEditPriority(todo.priority);
    setEditDeadline(todo.deadline ?? "");
    setEditing(false);
  };

  if (editing) {
    return (
      <div className={`todo-item editing priority-${editPriority}`}>
        <div className="edit-form">
          <input
            type="text"
            value={editTitle}
            onChange={(e) => setEditTitle(e.target.value)}
            className="edit-input"
            placeholder="Task title"
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
            placeholder="Description (optional)"
          />
          <div className="edit-options">
            <label>
              Priority:
              <select value={editPriority} onChange={(e) => setEditPriority(Number(e.target.value))}>
                <option value={1}>🔴 High</option>
                <option value={2}>🟡 Medium</option>
                <option value={3}>🟢 Low</option>
              </select>
            </label>
            <label>
              Deadline:
              <input
                type="datetime-local"
                value={editDeadline}
                onChange={(e) => setEditDeadline(e.target.value)}
              />
            </label>
          </div>
          <div className="edit-actions">
            <button onClick={handleSave} className="btn-save">Save</button>
            <button onClick={handleCancel} className="btn-cancel">Cancel</button>
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
            <span className="priority-badge" title={`优先级: ${pri.text}`}>
              {pri.emoji}
            </span>
            <span>{todo.title}</span>
          </div>
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
        <button onClick={() => onToggle(todo.id)} className="btn-complete" title="Complete" aria-label="Complete task">
          <CompleteIcon />
        </button>
        <button onClick={() => setEditing(true)} className="btn-edit" title="Edit">
          ✎
        </button>
        <button onClick={() => onDelete(todo.id)} className="btn-delete" title="Delete">
          ✕
        </button>
      </div>
    </div>
  );
}
