import { useTodos } from "./hooks/useTodos";
import { AddTodo } from "./components/AddTodo";
import { TodoList } from "./components/TodoList";
import "./App.css";

function App() {
  const { todos, loading, addTodo, toggleTodo, updateTodo, deleteTodo } = useTodos();

  return (
    <div className="app">
      <header className="app-header">
        <h1>📋 Lazy Todo List</h1>
        <p className="subtitle">优先级 · 倒计时 · 桌面端</p>
      </header>

      <main className="app-main">
        <AddTodo onAdd={addTodo} />
        {loading ? (
          <div className="loading">加载中...</div>
        ) : (
          <TodoList todos={todos} onToggle={toggleTodo} onUpdate={updateTodo} onDelete={deleteTodo} />
        )}
      </main>

      <footer className="app-footer">
        Powered by Tauri + Rust + React · Harness Engineering Demo
      </footer>
    </div>
  );
}

export default App;
