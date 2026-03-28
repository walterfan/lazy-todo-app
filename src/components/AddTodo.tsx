import { useState } from "react";
import type { CreateTodo } from "../types/todo";

interface Props {
  onAdd: (input: CreateTodo) => Promise<void>;
}

export function AddTodo({ onAdd }: Props) {
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [priority, setPriority] = useState(2);
  const [deadline, setDeadline] = useState("");
  const [expanded, setExpanded] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!title.trim()) return;

    await onAdd({
      title: title.trim(),
      description: description.trim() || undefined,
      priority,
      deadline: deadline || undefined,
    });

    setTitle("");
    setDescription("");
    setPriority(2);
    setDeadline("");
    setExpanded(false);
  };

  return (
    <form onSubmit={handleSubmit} className="add-todo">
      <div className="add-todo-main">
        <input
          type="text"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          placeholder="添加新任务..."
          className="add-todo-input"
        />
        <button
          type="button"
          onClick={() => setExpanded(!expanded)}
          className="btn-expand"
          title="更多选项"
        >
          {expanded ? "▲" : "▼"}
        </button>
        <button type="submit" className="btn-add" disabled={!title.trim()}>
          添加
        </button>
      </div>

      {expanded && (
        <div className="add-todo-extra">
          <input
            type="text"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="描述（可选）"
            className="add-todo-desc"
          />
          <div className="add-todo-options">
            <label>
              优先级：
              <select value={priority} onChange={(e) => setPriority(Number(e.target.value))}>
                <option value={1}>🔴 高</option>
                <option value={2}>🟡 中</option>
                <option value={3}>🟢 低</option>
              </select>
            </label>
            <label>
              截止时间：
              <input
                type="datetime-local"
                value={deadline}
                onChange={(e) => setDeadline(e.target.value)}
              />
            </label>
          </div>
        </div>
      )}
    </form>
  );
}
