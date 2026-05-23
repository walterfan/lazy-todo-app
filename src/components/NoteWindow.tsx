import { useState, useEffect, useCallback, useRef, type CSSProperties } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useTranslation } from "react-i18next";
import type { StickyNote, NoteColor } from "../types/note";
import { MarkdownPreview } from "./MarkdownPreview";
import { useAlwaysOnTop } from "../hooks/useAlwaysOnTop";
import { useSettings } from "../hooks/useSettings";
import {
  DEFAULT_NOTE_EDITOR_COLOR,
  getNoteEditorFieldBackground,
  NOTE_COLORS,
  NOTE_EDITOR_FIELD_TEXT_COLOR,
} from "../utils/noteColors";
import "../i18n";

type NoteDraft = {
  title: string;
  content: string;
  color: NoteColor;
};

function sameDraft(a: NoteDraft, b: NoteDraft) {
  return a.title === b.title && a.content === b.content && a.color === b.color;
}

interface NoteWindowProps {
  noteId: number;
}

export function NoteWindow({ noteId }: NoteWindowProps) {
  const { t, i18n } = useTranslation();
  const { settings } = useSettings();
  const [note, setNote] = useState<StickyNote | null>(null);
  const [editing, setEditing] = useState(false);
  const [title, setTitle] = useState("");
  const [content, setContent] = useState("");
  const [color, setColor] = useState<NoteColor>(DEFAULT_NOTE_EDITOR_COLOR);
  const [error, setError] = useState("");
  const { pinned, toggle: togglePinned } = useAlwaysOnTop();
  const draftRef = useRef<NoteDraft>({ title: "", content: "", color: DEFAULT_NOTE_EDITOR_COLOR });
  const lastSavedRef = useRef<NoteDraft>({ title: "", content: "", color: DEFAULT_NOTE_EDITOR_COLOR });
  const savePromiseRef = useRef<Promise<void> | null>(null);
  const editorStyle = {
    "--note-editor-field-bg": getNoteEditorFieldBackground(color),
    "--note-editor-field-text": NOTE_EDITOR_FIELD_TEXT_COLOR,
  } as CSSProperties;
  draftRef.current = { title, content, color };

  useEffect(() => {
    void i18n.changeLanguage(settings.language || "en");
  }, [i18n, settings.language]);

  const loadNote = useCallback(async () => {
    try {
      const notes = await invoke<StickyNote[]>("list_notes");
      const found = notes.find((n) => n.id === noteId);
      if (found) {
        setNote(found);
        const latest = { title: found.title, content: found.content, color: found.color };
        lastSavedRef.current = latest;
        if (!editing) {
          setTitle(found.title);
          setContent(found.content);
          setColor(found.color);
        }
      } else {
        setError(t("noteNotFound"));
      }
    } catch (err) {
      setError(String(err));
    }
  }, [editing, noteId, t]);

  useEffect(() => {
    loadNote();
  }, [loadNote]);

  const saveCurrentDraft = useCallback(async (rethrow = false) => {
    if (!note) {
      return;
    }
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
    const savePromise = invoke("update_note", {
      input: { id: noteId, ...savedDraft },
    }).then(() => {
      lastSavedRef.current = savedDraft;
      setNote((current) => (current ? { ...current, ...savedDraft } : current));
    });
    savePromiseRef.current = savePromise;

    try {
      await savePromise;
    } catch (err) {
      console.error("Failed to autosave note:", err);
      if (rethrow) {
        throw err;
      }
    } finally {
      if (savePromiseRef.current === savePromise) {
        savePromiseRef.current = null;
      }
    }
  }, [note, noteId]);

  useEffect(() => {
    if (!editing) {
      return;
    }
    const timer = window.setInterval(() => {
      void saveCurrentDraft();
    }, 3_000);
    return () => window.clearInterval(timer);
  }, [editing, saveCurrentDraft]);

  const handleSave = async () => {
    await saveCurrentDraft(true);
    await loadNote();
    setEditing(false);
    const win = getCurrentWindow();
    const savedTitle = draftRef.current.title;
    if (savedTitle) {
      win.setTitle(savedTitle);
    }
  };

  const handleCancel = () => {
    setTitle(lastSavedRef.current.title);
    setContent(lastSavedRef.current.content);
    setColor(lastSavedRef.current.color);
    setEditing(false);
  };

  if (error) {
    return <div className="note-window note-color-yellow"><p>{error}</p></div>;
  }

  if (!note) {
    return <div className="note-window note-color-yellow"><p>{t("loading")}</p></div>;
  }

  return (
    <div className={`note-window ${editing ? "" : `note-color-${color}`}`} style={editorStyle}>
      <div className="note-window-toolbar">
        <div className="note-window-color-row">
          {NOTE_COLORS.map((c) => (
            <button
              key={c}
              className={`color-dot color-dot-${c} ${c === color ? "active" : ""}`}
              onClick={async () => {
                setColor(c);
                if (editing) {
                  return;
                }
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
            title={pinned ? t("unpinShortcut") : t("alwaysOnTop")}
            aria-pressed={pinned}
          >
            {pinned ? "📌" : "📍"}
          </button>
          {!editing && (
            <button className="note-window-edit-btn" onClick={() => setEditing(true)}>
              ✏️ {t("edit")}
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
            placeholder={t("title")}
            autoFocus
          />
          <textarea
            className="note-window-content-input"
            value={content}
            onChange={(e) => setContent(e.target.value)}
            placeholder={t("writeNoteMarkdown")}
          />
          <div className="note-window-actions">
            <button className="btn-cancel" onClick={handleCancel}>{t("cancel")}</button>
            <button className="btn-save" onClick={handleSave}>{t("save")}</button>
          </div>
        </div>
      ) : (
        <div className="note-window-view" onClick={() => setEditing(true)}>
          {note.title && <h2 className="note-window-title">{note.title}</h2>}
          <div className="note-window-content">
            <MarkdownPreview content={note.content || t("emptyNoteEdit")} />
          </div>
        </div>
      )}
    </div>
  );
}
