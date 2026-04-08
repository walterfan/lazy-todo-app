import { useState, useEffect, useMemo } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useTodos } from "./hooks/useTodos";
import { useNotes } from "./hooks/useNotes";
import { AddTodo } from "./components/AddTodo";
import { TodoList } from "./components/TodoList";
import { NoteEditor } from "./components/NoteEditor";
import { NoteList } from "./components/NoteList";
import { PomodoroPanel } from "./components/PomodoroPanel";
import "./App.css";

type Tab = "todos" | "notes" | "pomodoro";

function App() {
  const [activeTab, setActiveTab] = useState<Tab>("todos");
  const [noteAutoFocus, setNoteAutoFocus] = useState(false);
  const [dbPath, setDbPath] = useState("");
  const [searchQuery, setSearchQuery] = useState("");
  const { todos, loading: todosLoading, addTodo, toggleTodo, updateTodo, deleteTodo } = useTodos();
  const { notes, loading: notesLoading, addNote, updateNote, deleteNote } = useNotes();

  const filteredTodos = useMemo(() => {
    const q = searchQuery.toLowerCase().trim();
    if (!q) return todos;
    return todos.filter(
      (t) =>
        t.title.toLowerCase().includes(q) ||
        t.description.toLowerCase().includes(q)
    );
  }, [todos, searchQuery]);

  const filteredNotes = useMemo(() => {
    const q = searchQuery.toLowerCase().trim();
    if (!q) return notes;
    return notes.filter(
      (n) =>
        n.title.toLowerCase().includes(q) ||
        n.content.toLowerCase().includes(q)
    );
  }, [notes, searchQuery]);

  useEffect(() => {
    invoke<string>("get_db_path").then(setDbPath).catch(() => {});
  }, []);

  useEffect(() => {
    const unlisten = listen("tray-new-note", () => {
      setActiveTab("notes");
      setNoteAutoFocus(true);
      setTimeout(() => setNoteAutoFocus(false), 500);
    });
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  useEffect(() => {
    setSearchQuery("");
  }, [activeTab]);

  return (
    <div className="app">
      <header className="app-header">
        <h1>📋 Lazy Todo App</h1>
        <p className="subtitle">优先级 · 倒计时 · 备忘录 · 番茄钟</p>
      </header>

      <nav className="tab-bar">
        <button
          className={`tab-btn ${activeTab === "todos" ? "active" : ""}`}
          onClick={() => setActiveTab("todos")}
        >
          ✅ Todos
        </button>
        <button
          className={`tab-btn ${activeTab === "notes" ? "active" : ""}`}
          onClick={() => setActiveTab("notes")}
        >
          📝 Notes
        </button>
        <button
          className={`tab-btn ${activeTab === "pomodoro" ? "active" : ""}`}
          onClick={() => setActiveTab("pomodoro")}
        >
          🍅 Pomodoro
        </button>
      </nav>

      <main className="app-main">
        {(activeTab === "todos" || activeTab === "notes") && (
          <div className="search-bar">
            <span className="search-icon">🔍</span>
            <input
              type="text"
              className="search-input"
              placeholder={activeTab === "todos" ? "Search tasks..." : "Search notes..."}
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
            {searchQuery && (
              <button className="search-clear" onClick={() => setSearchQuery("")}>
                ✕
              </button>
            )}
          </div>
        )}

        {activeTab === "todos" && (
          <>
            <AddTodo onAdd={addTodo} />
            {todosLoading ? (
              <div className="loading">加载中...</div>
            ) : (
              <>
                {searchQuery && (
                  <div className="search-result-count">
                    Found {filteredTodos.length} of {todos.length} tasks
                  </div>
                )}
                <TodoList todos={filteredTodos} onToggle={toggleTodo} onUpdate={updateTodo} onDelete={deleteTodo} />
              </>
            )}
          </>
        )}

        {activeTab === "notes" && (
          <>
            <NoteEditor onAdd={addNote} autoFocus={noteAutoFocus} />
            {notesLoading ? (
              <div className="loading">加载中...</div>
            ) : (
              <>
                {searchQuery && (
                  <div className="search-result-count">
                    Found {filteredNotes.length} of {notes.length} notes
                  </div>
                )}
                <NoteList notes={filteredNotes} onUpdate={updateNote} onDelete={deleteNote} />
              </>
            )}
          </>
        )}

        <div style={{ display: activeTab === "pomodoro" ? "block" : "none" }}>
          <PomodoroPanel />
        </div>
      </main>

      <footer className="app-footer">
        <span>Powered by Tauri + Rust + React</span>
        {dbPath && <span className="db-path" title={dbPath}>💾 {dbPath}</span>}
      </footer>
    </div>
  );
}

export default App;
