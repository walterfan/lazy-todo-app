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
  1: { emoji: "🔴", text: "高" },
  2: { emoji: "🟡", text: "中" },
  3: { emoji: "🟢", text: "低" },
};

function formatDeadline(deadline: string): string {
  const d = new Date(deadline);
  const month = d.getMonth() + 1;
  const day = d.getDate();
  const hour = d.getHours().toString().padStart(2, "0");
  const min = d.getMinutes().toString().padStart(2, "0");
  return `${month}/${day} ${hour}:${min}`;
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
            placeholder="任务名称"
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
            placeholder="描述（可选）"
          />
          <div className="edit-options">
            <label>
              优先级：
              <select value={editPriority} onChange={(e) => setEditPriority(Number(e.target.value))}>
                <option value={1}>🔴 高</option>
                <option value={2}>🟡 中</option>
                <option value={3}>🟢 低</option>
              </select>
            </label>
            <label>
              截止时间：
              <input
                type="datetime-local"
                value={editDeadline}
                onChange={(e) => setEditDeadline(e.target.value)}
              />
            </label>
          </div>
          <div className="edit-actions">
            <button onClick={handleSave} className="btn-save">保存</button>
            <button onClick={handleCancel} className="btn-cancel">取消</button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={`todo-item ${todo.completed ? "completed" : ""} priority-${todo.priority}`}>
      <div className="todo-left">
        <input
          type="checkbox"
          checked={todo.completed}
          onChange={() => onToggle(todo.id)}
          className="todo-checkbox"
        />
        <div className="todo-content">
          <div className="todo-title">
            <span className="priority-badge" title={`优先级: ${pri.text}`}>
              {pri.emoji}
            </span>
            <span className={todo.completed ? "line-through" : ""}>{todo.title}</span>
          </div>
          {todo.description && (
            <div className="todo-desc">{todo.description}</div>
          )}
          {todo.deadline && (
            <div className="todo-deadline-row">
              <span className="todo-deadline-date">
                📅 {formatDeadline(todo.deadline)}
              </span>
              {countdown && !todo.completed && (
                <span className={`todo-countdown ${countdown.overdue ? "overdue" : ""} ${countdown.urgent ? "urgent" : ""}`}>
                  ⏱ {countdown.label}
                </span>
              )}
            </div>
          )}
        </div>
      </div>
      <div className="todo-actions">
        <button onClick={() => setEditing(true)} className="btn-edit" title="编辑">
          ✎
        </button>
        <button onClick={() => onDelete(todo.id)} className="btn-delete" title="删除">
          ✕
        </button>
      </div>
    </div>
  );
}
