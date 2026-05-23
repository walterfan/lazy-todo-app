import { useCallback, useEffect, useRef, useState, type CSSProperties, type MouseEvent } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { StickyNote, UpdateNote, NoteColor } from "../types/note";
import type { Translator } from "../i18n";
import { MarkdownPreview } from "./MarkdownPreview";
import {
  getNoteEditorFieldBackground,
  NOTE_COLORS,
  NOTE_EDITOR_FIELD_TEXT_COLOR,
} from "../utils/noteColors";

type NoteDraft = {
  title: string;
  content: string;
  color: NoteColor;
};

function sameDraft(a: NoteDraft, b: NoteDraft) {
  return a.title === b.title && a.content === b.content && a.color === b.color;
}

interface NoteCardProps {
  note: StickyNote;
  onUpdate: (input: UpdateNote) => Promise<void>;
  onDelete: (id: number) => Promise<void>;
  onPinChange: (id: number, pinned: boolean) => Promise<void>;
  selected?: boolean;
  onSelectionChange?: (id: number, selected: boolean) => void;
  t: Translator;
}

export function NoteCard({ note, onUpdate, onDelete, onPinChange, selected = false, onSelectionChange, t }: NoteCardProps) {
  const [editing, setEditing] = useState(false);
  const [maximized, setMaximized] = useState(false);
  const [expanded, setExpanded] = useState(false);
  const [confirmingDelete, setConfirmingDelete] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const [deleteError, setDeleteError] = useState("");
  const [title, setTitle] = useState(note.title);
  const [content, setContent] = useState(note.content);
  const [color, setColor] = useState<NoteColor>(note.color);
  const draftRef = useRef<NoteDraft>({ title: note.title, content: note.content, color: note.color });
  const lastSavedRef = useRef<NoteDraft>({ title: note.title, content: note.content, color: note.color });
  const savePromiseRef = useRef<Promise<void> | null>(null);
  const editorStyle = {
    "--note-editor-field-bg": getNoteEditorFieldBackground(color),
    "--note-editor-field-text": NOTE_EDITOR_FIELD_TEXT_COLOR,
  } as CSSProperties;
  draftRef.current = { title, content, color };

  useEffect(() => {
    lastSavedRef.current = { title: note.title, content: note.content, color: note.color };
    if (!editing) {
      setTitle(note.title);
      setContent(note.content);
      setColor(note.color);
    }
  }, [note.id, note.title, note.content, note.color, editing]);

  const saveCurrentDraft = useCallback(async (rethrow = false) => {
    if (savePromiseRef.current) {
      try {
        await savePromiseRef.current;
      } catch (error) {
        if (rethrow) {
          throw error;
        }
        return;
      }
    }

    const draft = draftRef.current;
    if (sameDraft(draft, lastSavedRef.current)) {
      return;
    }

    const savedDraft = { ...draft };
    const savePromise = onUpdate({ id: note.id, ...savedDraft }).then(() => {
      lastSavedRef.current = savedDraft;
    });
    savePromiseRef.current = savePromise;

    try {
      await savePromise;
    } catch (error) {
      console.error("Failed to autosave note:", error);
      if (rethrow) {
        throw error;
      }
    } finally {
      if (savePromiseRef.current === savePromise) {
        savePromiseRef.current = null;
      }
    }
  }, [note.id, onUpdate]);

  useEffect(() => {
    if (!editing) {
      return;
    }
    const timer = window.setInterval(() => {
      void saveCurrentDraft();
    }, 3_000);
    return () => window.clearInterval(timer);
  }, [editing, saveCurrentDraft]);

  useEffect(() => {
    if (!maximized) {
      return;
    }
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setMaximized(false);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [maximized]);

  const handleSave = async () => {
    await saveCurrentDraft(true);
    setEditing(false);
    setMaximized(false);
  };

  const handleCancel = () => {
    setTitle(lastSavedRef.current.title);
    setContent(lastSavedRef.current.content);
    setColor(lastSavedRef.current.color);
    setEditing(false);
    setMaximized(false);
  };

  const handleMaximizeEdit = (event?: MouseEvent<HTMLButtonElement>) => {
    event?.preventDefault();
    event?.stopPropagation();
    setDeleteError("");
    setConfirmingDelete(false);
    setEditing(true);
    setMaximized(true);
  };

  const handleDeleteClick = (event: MouseEvent<HTMLButtonElement>) => {
    event.preventDefault();
    event.stopPropagation();
    setDeleteError("");
    setConfirmingDelete(true);
  };

  const handleCancelDelete = (event: MouseEvent<HTMLButtonElement>) => {
    event.preventDefault();
    event.stopPropagation();
    setDeleteError("");
    setConfirmingDelete(false);
  };

  const handleConfirmDelete = async (event: MouseEvent<HTMLButtonElement>) => {
    event.preventDefault();
    event.stopPropagation();
    setDeleting(true);
    setDeleteError("");
    try {
      await onDelete(note.id);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setDeleteError(message);
      setConfirmingDelete(true);
    } finally {
      setDeleting(false);
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

  const editor = (
    <div
      className={`note-card editing ${maximized ? "note-card-maximized" : ""}`}
      onClick={(event) => event.stopPropagation()}
      style={editorStyle}
    >
      <div className="note-editor-title-row">
        <input
          className="note-editor-title"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          placeholder={t("noteTitlePlaceholder")}
        />
        <button
          type="button"
          className="btn-maximize-note note-editor-maximize"
          onClick={() => setMaximized((current) => !current)}
          title={maximized ? t("restoreNoteEdit") : t("maximizeNoteEdit")}
          aria-label={maximized ? t("restoreNoteEdit") : t("maximizeNoteEdit")}
        >
          {maximized ? "🗗" : "⛶"}
        </button>
      </div>
      <textarea
        className="note-editor-content"
        value={content}
        onChange={(e) => setContent(e.target.value)}
        rows={maximized ? 18 : 8}
        autoFocus={maximized}
      />
      <div className="note-editor-footer">
        <div className="color-picker">
          {NOTE_COLORS.map((c) => (
            <button
              key={c}
              className={`color-dot color-dot-${c} ${c === color ? "active" : ""}`}
              onClick={() => setColor(c)}
              title={c}
              aria-label={c}
            />
          ))}
        </div>
        <div className="note-editor-actions">
          <button className="btn-cancel" onClick={handleCancel}>{t("cancel")}</button>
          <button className="btn-save" onClick={handleSave}>{t("save")}</button>
        </div>
      </div>
    </div>
  );

  if (editing) {
    if (maximized) {
      return (
        <div className="note-maximize-backdrop" role="dialog" aria-modal="true" aria-label={t("maximizeNoteEdit")}>
          {editor}
        </div>
      );
    }
    return editor;
  }

  return (
    <div
      className={`note-card note-color-${color} ${selected ? "selected" : ""} ${confirmingDelete ? "confirming-delete" : ""}`}
      onClick={() => setExpanded(!expanded)}
    >
      <div className="note-card-header">
        <label className="note-select" onClick={(e) => e.stopPropagation()} title={t("selectNote")}>
          <input
            type="checkbox"
            checked={selected}
            onChange={(e) => onSelectionChange?.(note.id, e.target.checked)}
            aria-label={t("selectNote")}
          />
        </label>
        {note.title && <h3 className="note-card-title">{note.title}</h3>}
        <div className="note-card-actions" onClick={(e) => e.stopPropagation()}>
          {confirmingDelete ? (
            <div className="note-delete-confirm" role="group" aria-label={t("deleteNoteConfirm")}>
              <span>{t("deleteNoteConfirm")}</span>
              <button type="button" className="btn-cancel" onClick={handleCancelDelete} disabled={deleting}>
                {t("cancel")}
              </button>
              <button type="button" className="btn-delete" onClick={handleConfirmDelete} disabled={deleting}>
                {t("delete")}
              </button>
            </div>
          ) : (
            <>
              <button
                type="button"
                className="btn-popout"
                onClick={() => invoke("open_note_window", { noteId: note.id, title: note.title || "Sticky Note" })}
                title={t("openInWindow")}
              >
                ↗️
              </button>
              <button
                type="button"
                className="btn-maximize-note"
                onClick={handleMaximizeEdit}
                title={t("maximizeNoteEdit")}
                aria-label={t("maximizeNoteEdit")}
              >
                ⛶
              </button>
              <button type="button" className="btn-edit" onClick={() => setEditing(true)} title={t("edit")}>✏️</button>
              <button
                type="button"
                className={`btn-pin-note ${note.pinned ? "active" : ""}`}
                onClick={() => void onPinChange(note.id, !note.pinned)}
                title={note.pinned ? t("unpinNote") : t("pinNote")}
                aria-pressed={note.pinned}
              >
                {note.pinned ? "📌" : "📍"}
              </button>
              <button type="button" className="btn-delete" onClick={handleDeleteClick} title={t("delete")}>🗑️</button>
            </>
          )}
        </div>
      </div>
      {note.pinned && <div className="note-pinned-label">{t("pinnedNote")}</div>}
      {deleteError && <div className="note-delete-error">{deleteError}</div>}
      <div className={`note-card-body ${expanded ? "expanded" : ""}`}>
        <MarkdownPreview content={note.content} />
      </div>
      <div className="note-card-footer">
        <span className="note-card-time">{timeAgo(note.updated_at)}</span>
      </div>
    </div>
  );
}
