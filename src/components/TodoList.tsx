import type { Todo, UpdateTodo } from "../types/todo";
import type { DisplayStyle } from "../types/settings";
import type { Translator } from "../i18n";
import { TodoItem } from "./TodoItem";

interface Props {
  todos: Todo[];
  onToggle: (id: number) => Promise<void>;
  onUpdate: (input: UpdateTodo) => Promise<void>;
  onDelete: (id: number) => Promise<void>;
  displayStyle?: DisplayStyle;
  t: Translator;
}

export function TodoList({ todos, onToggle, onUpdate, onDelete, displayStyle = "list", t }: Props) {
  const pending = todos.filter((t) => !t.completed);
  const isGrid = displayStyle === "grid";

  return (
    <div className="todo-list">
      {pending.length === 0 && (
        <div className="empty-state">{t("noActiveTasks")}</div>
      )}

      {pending.length > 0 && (
        <div className="todo-section">
          <h3 className="section-title">{t("todos")} ({pending.length})</h3>
          <div className={isGrid ? "todo-grid" : ""}>
            {pending.map((todo) => (
              <TodoItem key={todo.id} todo={todo} onToggle={onToggle} onUpdate={onUpdate} onDelete={onDelete} t={t} />
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
