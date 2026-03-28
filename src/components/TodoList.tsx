import type { Todo, UpdateTodo } from "../types/todo";
import { TodoItem } from "./TodoItem";

interface Props {
  todos: Todo[];
  onToggle: (id: number) => Promise<void>;
  onUpdate: (input: UpdateTodo) => Promise<void>;
  onDelete: (id: number) => Promise<void>;
}

export function TodoList({ todos, onToggle, onUpdate, onDelete }: Props) {
  const pending = todos.filter((t) => !t.completed);
  const done = todos.filter((t) => t.completed);

  return (
    <div className="todo-list">
      {pending.length === 0 && done.length === 0 && (
        <div className="empty-state">暂无任务，添加一个吧 👆</div>
      )}

      {pending.length > 0 && (
        <div className="todo-section">
          <h3 className="section-title">待完成 ({pending.length})</h3>
          {pending.map((todo) => (
            <TodoItem key={todo.id} todo={todo} onToggle={onToggle} onUpdate={onUpdate} onDelete={onDelete} />
          ))}
        </div>
      )}

      {done.length > 0 && (
        <div className="todo-section">
          <h3 className="section-title">已完成 ({done.length})</h3>
          {done.map((todo) => (
            <TodoItem key={todo.id} todo={todo} onToggle={onToggle} onUpdate={onUpdate} onDelete={onDelete} />
          ))}
        </div>
      )}
    </div>
  );
}
