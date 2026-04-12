import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { StickyNote, UpdateNote, NoteColor } from "../types/note";
import { MarkdownPreview } from "./MarkdownPreview";

const COLORS: NoteColor[] = ["yellow", "green", "blue", "pink", "purple", "orange"];

interface NoteCardProps {
  note: StickyNote;
  onUpdate: (input: UpdateNote) => Promise<void>;
  onDelete: (id: number) => Promise<void>;
}

export function NoteCard({ note, onUpdate, onDelete }: NoteCardProps) {
  const [editing, setEditing] = useState(false);
  const [expanded, setExpanded] = useState(false);
  const [title, setTitle] = useState(note.title);
  const [content, setContent] = useState(note.content);
  const [color, setColor] = useState<NoteColor>(note.color);

  const handleSave = async () => {
    await onUpdate({ id: note.id, title, content, color });
    setEditing(false);
  };

  const handleCancel = () => {
    setTitle(note.title);
    setContent(note.content);
    setColor(note.color);
    setEditing(false);
  };

  const handleDelete = () => {
    if (window.confirm("Delete this note?")) {
      onDelete(note.id);
    }
  };

  const timeAgo = (dateStr: string) => {
    const diff = Date.now() - new Date(dateStr + "Z").getTime();
    const mins = Math.floor(diff / 60000);
    if (mins < 1) return "just now";
    if (mins < 60) return `${mins}m ago`;
    const hrs = Math.floor(mins / 60);
    if (hrs < 24) return `${hrs}h ago`;
    const days = Math.floor(hrs / 24);
    return `${days}d ago`;
  };

  if (editing) {
    return (
      <div className={`note-card note-color-${color} editing`}>
        <input
          className="note-editor-title"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          placeholder="Title"
        />
        <textarea
          className="note-editor-content"
          value={content}
          onChange={(e) => setContent(e.target.value)}
          rows={8}
        />
        <div className="note-editor-footer">
          <div className="color-picker">
            {COLORS.map((c) => (
              <button
                key={c}
                className={`color-dot color-dot-${c} ${c === color ? "active" : ""}`}
                onClick={() => setColor(c)}
                title={c}
              />
            ))}
          </div>
          <div className="note-editor-actions">
            <button className="btn-cancel" onClick={handleCancel}>Cancel</button>
            <button className="btn-save" onClick={handleSave}>Save</button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div
      className={`note-card note-color-${color}`}
      onClick={() => setExpanded(!expanded)}
    >
      <div className="note-card-header">
        {note.title && <h3 className="note-card-title">{note.title}</h3>}
        <div className="note-card-actions" onClick={(e) => e.stopPropagation()}>
          <button
            className="btn-popout"
            onClick={() => invoke("open_note_window", { noteId: note.id, title: note.title || "Sticky Note" })}
            title="Open in window"
          >
            ↗️
          </button>
          <button className="btn-edit" onClick={() => setEditing(true)} title="Edit">✏️</button>
          <button className="btn-delete" onClick={handleDelete} title="Delete">🗑️</button>
        </div>
      </div>
      <div className={`note-card-body ${expanded ? "expanded" : ""}`}>
        <MarkdownPreview content={note.content} />
      </div>
      <div className="note-card-footer">
        <span className="note-card-time">{timeAgo(note.updated_at)}</span>
      </div>
    </div>
  );
}
