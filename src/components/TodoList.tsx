import type { Todo, UpdateTodo } from "../types/todo";
import type { DisplayStyle } from "../types/settings";
import { TodoItem } from "./TodoItem";

interface Props {
  todos: Todo[];
  onToggle: (id: number) => Promise<void>;
  onUpdate: (input: UpdateTodo) => Promise<void>;
  onDelete: (id: number) => Promise<void>;
  displayStyle?: DisplayStyle;
}

export function TodoList({ todos, onToggle, onUpdate, onDelete, displayStyle = "list" }: Props) {
  const pending = todos.filter((t) => !t.completed);
  const isGrid = displayStyle === "grid";

  return (
    <div className="todo-list">
      {pending.length === 0 && (
        <div className="empty-state">No active tasks. Add one above!</div>
      )}

      {pending.length > 0 && (
        <div className="todo-section">
          <h3 className="section-title">Pending ({pending.length})</h3>
          <div className={isGrid ? "todo-grid" : ""}>
            {pending.map((todo) => (
              <TodoItem key={todo.id} todo={todo} onToggle={onToggle} onUpdate={onUpdate} onDelete={onDelete} />
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
