import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { StickyNote, NoteColor } from "../types/note";
import { MarkdownPreview } from "./MarkdownPreview";
import { useAlwaysOnTop } from "../hooks/useAlwaysOnTop";

const COLORS: NoteColor[] = ["yellow", "green", "blue", "pink", "purple", "orange"];

interface NoteWindowProps {
  noteId: number;
}

export function NoteWindow({ noteId }: NoteWindowProps) {
  const [note, setNote] = useState<StickyNote | null>(null);
  const [editing, setEditing] = useState(false);
  const [title, setTitle] = useState("");
  const [content, setContent] = useState("");
  const [color, setColor] = useState<NoteColor>("yellow");
  const [error, setError] = useState("");
  const { pinned, toggle: togglePinned } = useAlwaysOnTop();

  const loadNote = useCallback(async () => {
    try {
      const notes = await invoke<StickyNote[]>("list_notes");
      const found = notes.find((n) => n.id === noteId);
      if (found) {
        setNote(found);
        setTitle(found.title);
        setContent(found.content);
        setColor(found.color);
      } else {
        setError("Note not found");
      }
    } catch (err) {
      setError(String(err));
    }
  }, [noteId]);

  useEffect(() => {
    loadNote();
  }, [loadNote]);

  const handleSave = async () => {
    await invoke("update_note", { input: { id: noteId, title, content, color } });
    await loadNote();
    setEditing(false);
    const win = getCurrentWindow();
    if (title) {
      win.setTitle(title);
    }
  };

  const handleCancel = () => {
    if (note) {
      setTitle(note.title);
      setContent(note.content);
      setColor(note.color);
    }
    setEditing(false);
  };

  if (error) {
    return <div className="note-window note-color-yellow"><p>{error}</p></div>;
  }

  if (!note) {
    return <div className="note-window note-color-yellow"><p>Loading...</p></div>;
  }

  return (
    <div className={`note-window note-color-${color}`}>
      <div className="note-window-toolbar">
        <div className="note-window-color-row">
          {COLORS.map((c) => (
            <button
              key={c}
              className={`color-dot color-dot-${c} ${c === color ? "active" : ""}`}
              onClick={async () => {
                setColor(c);
                await invoke("update_note", { input: { id: noteId, color: c } });
                await loadNote();
              }}
              title={c}
            />
          ))}
        </div>
        <div className="note-window-toolbar-actions">
          <button
            className={`note-window-pin-btn ${pinned ? "active" : ""}`}
            onClick={togglePinned}
            title={pinned ? "Unpin (Cmd/Ctrl+Shift+T)" : "Always on top (Cmd/Ctrl+Shift+T)"}
            aria-pressed={pinned}
          >
            {pinned ? "📌" : "📍"}
          </button>
          {!editing && (
            <button className="note-window-edit-btn" onClick={() => setEditing(true)}>
              ✏️ Edit
            </button>
          )}
        </div>
      </div>

      {editing ? (
        <div className="note-window-editor">
          <input
            className="note-window-title-input"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="Title"
            autoFocus
          />
          <textarea
            className="note-window-content-input"
            value={content}
            onChange={(e) => setContent(e.target.value)}
            placeholder="Write your note in Markdown..."
          />
          <div className="note-window-actions">
            <button className="btn-cancel" onClick={handleCancel}>Cancel</button>
            <button className="btn-save" onClick={handleSave}>Save</button>
          </div>
        </div>
      ) : (
        <div className="note-window-view" onClick={() => setEditing(true)}>
          {note.title && <h2 className="note-window-title">{note.title}</h2>}
          <div className="note-window-content">
            <MarkdownPreview content={note.content || "*Empty note — click to edit*"} />
          </div>
        </div>
      )}
    </div>
  );
}
