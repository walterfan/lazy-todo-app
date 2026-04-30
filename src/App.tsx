import { useState, useEffect, useMemo } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useTodos } from "./hooks/useTodos";
import { useNotes } from "./hooks/useNotes";
import { useSettings } from "./hooks/useSettings";
import { useSecretary } from "./hooks/useSecretary";
import { useAgents } from "./hooks/useAgents";
import { useAlwaysOnTop } from "./hooks/useAlwaysOnTop";
import { AddTodo } from "./components/AddTodo";
import { TodoList } from "./components/TodoList";
import { NoteEditor } from "./components/NoteEditor";
import { NoteList } from "./components/NoteList";
import { PomodoroPanel } from "./components/PomodoroPanel";
import { ToolboxPanel } from "./components/toolbox/ToolboxPanel";
import { SettingsPanel } from "./components/SettingsPanel";
import { AgentsPanel } from "./components/AgentsPanel";
import { useTranslation } from "react-i18next";
import "./i18n";
import "./App.css";

type Tab = "todos" | "notes" | "pomodoro" | "toolbox" | "agents" | "settings";

function App() {
  const [activeTab, setActiveTab] = useState<Tab>("todos");
  const [noteAutoFocus, setNoteAutoFocus] = useState(false);
  const [dbPath, setDbPath] = useState("");
  const [searchQuery, setSearchQuery] = useState("");
  const [notePage, setNotePage] = useState(1);
  const [selectedNoteIds, setSelectedNoteIds] = useState<Set<number>>(() => new Set());
  const [noteExportStatus, setNoteExportStatus] = useState("");
  const { todos, loading: todosLoading, addTodo, toggleTodo, updateTodo, deleteTodo } = useTodos();
  const { notes, loading: notesLoading, addNote, updateNote, deleteNote, setNotePinned, exportNotes } = useNotes();
  const { settings, updateSettings } = useSettings();
  const secretary = useSecretary();
  const agents = useAgents();
  const { pinned, toggle: togglePinned } = useAlwaysOnTop();
  const { t, i18n } = useTranslation();
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

  const notePageSize = Math.max(1, Math.min(200, Number(settings.note_page_size) || 10));
  const notePageCount = Math.max(1, Math.ceil(filteredNotes.length / notePageSize));
  const paginatedNotes = useMemo(() => {
    const start = (notePage - 1) * notePageSize;
    return filteredNotes.slice(start, start + notePageSize);
  }, [filteredNotes, notePage, notePageSize]);

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

  useEffect(() => {
    setNotePage(1);
  }, [activeTab, searchQuery, notePageSize]);

  useEffect(() => {
    setNotePage((current) => Math.min(current, notePageCount));
  }, [notePageCount]);

  useEffect(() => {
    void i18n.changeLanguage(settings.language || "en");
  }, [i18n, settings.language]);

  const visibleSelectedNoteIds = useMemo(
    () => paginatedNotes.filter((note) => selectedNoteIds.has(note.id)).map((note) => note.id),
    [paginatedNotes, selectedNoteIds]
  );

  const NAV_ITEMS: { key: Tab; icon: string; label: string }[] = [
    { key: "todos", icon: "✅", label: t("todos") },
    { key: "notes", icon: "📝", label: t("notes") },
    { key: "pomodoro", icon: "🍅", label: t("pomodoro") },
    { key: "toolbox", icon: "🧰", label: t("toolbox") },
    { key: "agents", icon: "🗂️", label: t("agents") },
  ];

  const handleQuit = () => invoke("quit_app");

  const setNoteSelected = (id: number, selected: boolean) => {
    setSelectedNoteIds((current) => {
      const next = new Set(current);
      if (selected) {
        next.add(id);
      } else {
        next.delete(id);
      }
      return next;
    });
    setNoteExportStatus("");
  };

  const selectVisibleNotes = () => {
    setSelectedNoteIds((current) => {
      const next = new Set(current);
      paginatedNotes.forEach((note) => next.add(note.id));
      return next;
    });
    setNoteExportStatus("");
  };

  const clearSelectedNotes = () => {
    setSelectedNoteIds(new Set());
    setNoteExportStatus("");
  };

  const handleDeleteNote = async (id: number) => {
    await deleteNote(id);
    setSelectedNoteIds((current) => {
      const next = new Set(current);
      next.delete(id);
      return next;
    });
  };

  const saveSelectedNotes = async () => {
    const ids = Array.from(selectedNoteIds);
    if (ids.length === 0) return;
    setNoteExportStatus(t("savingSelectedNotes"));
    try {
      const result = await exportNotes(ids);
      setNoteExportStatus(t("savedNoteFiles", { count: result.files.length, plural: result.files.length === 1 ? "" : "s", folder: result.folder }));
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setNoteExportStatus(t("saveFailed", { message }));
    }
  };

  return (
    <div className="app-layout">
      <aside className="sidebar">
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
            <span className="nav-label">{pinned ? t("pinned") : t("pin")}</span>
          </button>
          <button
            className={`nav-item ${activeTab === "settings" ? "active" : ""}`}
            onClick={() => setActiveTab("settings")}
            title={t("settings")}
          >
            <span className="nav-icon">⚙️</span>
            <span className="nav-label">{t("settings")}</span>
          </button>
          <button
            className="nav-item nav-quit"
            onClick={handleQuit}
            title={t("quit")}
          >
            <span className="nav-icon">🚪</span>
            <span className="nav-label">{t("quit")}</span>
          </button>
        </div>
      </aside>

      <div className="content-area">
        <header className="content-header">
          <h2 className="content-title">
            {activeTab === "settings" ? `⚙️ ${t("settings")}` : (
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
                placeholder={activeTab === "todos" ? t("searchTasks") : t("searchNotes")}
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
              <AddTodo onAdd={addTodo} t={t} />
              {todosLoading ? (
                <div className="loading">{t("loading")}</div>
              ) : (
                <>
                  {searchQuery && (
                    <div className="search-result-count">
                      {t("foundTasks", { count: filteredTodos.length, total: activeTodos.length })}
                    </div>
                  )}
                  <TodoList
                    todos={filteredTodos}
                    onToggle={toggleTodo}
                    onUpdate={updateTodo}
                    onDelete={deleteTodo}
                    displayStyle={settings.todo_display}
                    t={t}
                  />
                </>
              )}
            </>
          )}

          {activeTab === "notes" && (
            <>
              <NoteEditor onAdd={addNote} autoFocus={noteAutoFocus} template={settings.note_template} t={t} />
              {notesLoading ? (
                <div className="loading">{t("loading")}</div>
              ) : (
                <>
                  {searchQuery && (
                    <div className="search-result-count">
                      {t("foundNotes", { count: filteredNotes.length, total: notes.length })}
                    </div>
                  )}
                  <div className="note-selection-toolbar">
                    <div>
                      <strong>{selectedNoteIds.size}</strong> {t("selected")}
                      {visibleSelectedNoteIds.length !== selectedNoteIds.size && selectedNoteIds.size > 0 && (
                        <span> · {visibleSelectedNoteIds.length} {t("visible")}</span>
                      )}
                    </div>
                    <div className="note-selection-actions">
                      <button
                        type="button"
                        onClick={selectVisibleNotes}
                        disabled={paginatedNotes.length === 0 || visibleSelectedNoteIds.length === paginatedNotes.length}
                      >
                        {t("selectVisible")}
                      </button>
                      <button type="button" onClick={clearSelectedNotes} disabled={selectedNoteIds.size === 0}>
                        {t("clear")}
                      </button>
                      <button type="button" onClick={saveSelectedNotes} disabled={selectedNoteIds.size === 0}>
                        {t("saveToFolder")}
                      </button>
                    </div>
                  </div>
                  {noteExportStatus && <div className="note-export-status">{noteExportStatus}</div>}
                  <NoteList
                    notes={paginatedNotes}
                    onUpdate={updateNote}
                    onDelete={handleDeleteNote}
                    onPinChange={setNotePinned}
                    displayStyle="list"
                    selectedNoteIds={selectedNoteIds}
                    onSelectionChange={setNoteSelected}
                    t={t}
                  />
                  <div className="note-pagination">
                    <span>
                      {t("notePageStatus", { page: notePage, pages: notePageCount, total: filteredNotes.length })}
                    </span>
                    <div className="note-pagination-actions">
                      <button type="button" onClick={() => setNotePage(1)} disabled={notePage <= 1}>
                        {t("firstPage")}
                      </button>
                      <button type="button" onClick={() => setNotePage((page) => Math.max(1, page - 1))} disabled={notePage <= 1}>
                        {t("previousPage")}
                      </button>
                      <button type="button" onClick={() => setNotePage((page) => Math.min(notePageCount, page + 1))} disabled={notePage >= notePageCount}>
                        {t("nextPage")}
                      </button>
                      <button type="button" onClick={() => setNotePage(notePageCount)} disabled={notePage >= notePageCount}>
                        {t("lastPage")}
                      </button>
                    </div>
                  </div>
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

          <div style={{ display: activeTab === "agents" ? "block" : "none" }}>
            <AgentsPanel
              agents={agents}
              onRecordMessageToNote={(content, title) => addNote({ title, content, color: "blue" })}
              t={t}
            />
          </div>

          {activeTab === "settings" && (
            <SettingsPanel settings={settings} dbPath={dbPath} agents={agents} secretary={secretary} onUpdate={updateSettings} t={t} />
          )}
        </main>
      </div>
    </div>
  );
}

export default App;
