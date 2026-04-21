import { useState, useEffect, useMemo } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useTodos } from "./hooks/useTodos";
import { useNotes } from "./hooks/useNotes";
import { useSettings } from "./hooks/useSettings";
import { useAlwaysOnTop } from "./hooks/useAlwaysOnTop";
import { AddTodo } from "./components/AddTodo";
import { TodoList } from "./components/TodoList";
import { NoteEditor } from "./components/NoteEditor";
import { NoteList } from "./components/NoteList";
import { PomodoroPanel } from "./components/PomodoroPanel";
import { ToolboxPanel } from "./components/toolbox/ToolboxPanel";
import { SettingsPanel } from "./components/SettingsPanel";
import "./App.css";

type Tab = "todos" | "notes" | "pomodoro" | "toolbox" | "settings";

function App() {
  const [activeTab, setActiveTab] = useState<Tab>("todos");
  const [noteAutoFocus, setNoteAutoFocus] = useState(false);
  const [dbPath, setDbPath] = useState("");
  const [searchQuery, setSearchQuery] = useState("");
  const { todos, loading: todosLoading, addTodo, toggleTodo, updateTodo, deleteTodo } = useTodos();
  const { notes, loading: notesLoading, addNote, updateNote, deleteNote } = useNotes();
  const { settings, updateSettings } = useSettings();
  const { pinned, toggle: togglePinned } = useAlwaysOnTop();
  const activeTodos = useMemo(() => todos.filter((t) => !t.completed), [todos]);

  const filteredTodos = useMemo(() => {
    const q = searchQuery.toLowerCase().trim();
    if (!q) return activeTodos;
    return activeTodos.filter(
      (t) =>
        t.title.toLowerCase().includes(q) ||
        t.description.toLowerCase().includes(q)
    );
  }, [activeTodos, searchQuery]);

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

  const NAV_ITEMS: { key: Tab; icon: string; label: string }[] = [
    { key: "todos", icon: "✅", label: "Todos" },
    { key: "notes", icon: "📝", label: "Notes" },
    { key: "pomodoro", icon: "🍅", label: "Pomodoro" },
    { key: "toolbox", icon: "🧰", label: "Toolbox" },
  ];

  const handleQuit = () => invoke("quit_app");

  return (
    <div className="app-layout">
      <aside className="sidebar">
        <div className="sidebar-brand">📋</div>
        <nav className="sidebar-nav">
          {NAV_ITEMS.map((item) => (
            <button
              key={item.key}
              className={`nav-item ${activeTab === item.key ? "active" : ""}`}
              onClick={() => setActiveTab(item.key)}
              title={item.label}
            >
              <span className="nav-icon">{item.icon}</span>
              <span className="nav-label">{item.label}</span>
            </button>
          ))}
        </nav>
        <div className="sidebar-bottom">
          <button
            className={`nav-item nav-pin ${pinned ? "active" : ""}`}
            onClick={togglePinned}
            title={pinned ? "Unpin window (Cmd/Ctrl+Shift+T)" : "Always on top (Cmd/Ctrl+Shift+T)"}
            aria-pressed={pinned}
          >
            <span className="nav-icon">{pinned ? "📌" : "📍"}</span>
            <span className="nav-label">{pinned ? "Pinned" : "Pin"}</span>
          </button>
          <button
            className={`nav-item ${activeTab === "settings" ? "active" : ""}`}
            onClick={() => setActiveTab("settings")}
            title="Settings"
          >
            <span className="nav-icon">⚙️</span>
            <span className="nav-label">Settings</span>
          </button>
          <button
            className="nav-item nav-quit"
            onClick={handleQuit}
            title="Quit"
          >
            <span className="nav-icon">🚪</span>
            <span className="nav-label">Quit</span>
          </button>
        </div>
      </aside>

      <div className="content-area">
        <header className="content-header">
          <h2 className="content-title">
            {activeTab === "settings" ? "⚙️ Settings" : (
              <>
                {NAV_ITEMS.find((i) => i.key === activeTab)?.icon}{" "}
                {NAV_ITEMS.find((i) => i.key === activeTab)?.label}
              </>
            )}
          </h2>
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
        </header>

        <main className="content-main">
          {activeTab === "todos" && (
            <>
              <AddTodo onAdd={addTodo} />
              {todosLoading ? (
                <div className="loading">Loading...</div>
              ) : (
                <>
                  {searchQuery && (
                    <div className="search-result-count">
                      Found {filteredTodos.length} of {activeTodos.length} active tasks
                    </div>
                  )}
                  <TodoList
                    todos={filteredTodos}
                    onToggle={toggleTodo}
                    onUpdate={updateTodo}
                    onDelete={deleteTodo}
                    displayStyle={settings.todo_display}
                  />
                </>
              )}
            </>
          )}

          {activeTab === "notes" && (
            <>
              <NoteEditor onAdd={addNote} autoFocus={noteAutoFocus} template={settings.note_template} />
              {notesLoading ? (
                <div className="loading">Loading...</div>
              ) : (
                <>
                  {searchQuery && (
                    <div className="search-result-count">
                      Found {filteredNotes.length} of {notes.length} notes
                    </div>
                  )}
                  <NoteList
                    notes={filteredNotes}
                    onUpdate={updateNote}
                    onDelete={deleteNote}
                    displayStyle={settings.note_display}
                  />
                </>
              )}
            </>
          )}

          <div style={{ display: activeTab === "pomodoro" ? "block" : "none" }}>
            <PomodoroPanel />
          </div>

          <div style={{ display: activeTab === "toolbox" ? "block" : "none" }}>
            <ToolboxPanel />
          </div>

          {activeTab === "settings" && (
            <SettingsPanel settings={settings} dbPath={dbPath} onUpdate={updateSettings} />
          )}
        </main>
      </div>
    </div>
  );
}

export default App;
