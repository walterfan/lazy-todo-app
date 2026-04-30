import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { StickyNote, CreateNote, UpdateNote, ExportNotesResult } from "../types/note";

export function useNotes() {
  const [notes, setNotes] = useState<StickyNote[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    try {
      const data = await invoke<StickyNote[]>("list_notes");
      setNotes(data);
    } catch (err) {
      console.error("Failed to load notes:", err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const addNote = async (input: CreateNote) => {
    await invoke("add_note", { input });
    await refresh();
  };

  const updateNote = async (input: UpdateNote) => {
    await invoke("update_note", { input });
    await refresh();
  };

  const deleteNote = async (id: number) => {
    await invoke("delete_note", { id });
    await refresh();
  };

  const setNotePinned = async (id: number, pinned: boolean) => {
    await invoke("set_note_pinned", { id, pinned });
    await refresh();
  };

  const exportNotes = async (noteIds: number[]) => {
    return invoke<ExportNotesResult>("export_notes_to_folder", {
      input: { note_ids: noteIds },
    });
  };

  return { notes, loading, addNote, updateNote, deleteNote, setNotePinned, exportNotes, refresh };
}
