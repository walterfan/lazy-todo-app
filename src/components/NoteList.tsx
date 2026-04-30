import type { StickyNote, UpdateNote } from "../types/note";
import type { DisplayStyle } from "../types/settings";
import type { Translator } from "../i18n";
import { NoteCard } from "./NoteCard";

interface NoteListProps {
  notes: StickyNote[];
  onUpdate: (input: UpdateNote) => Promise<void>;
  onDelete: (id: number) => Promise<void>;
  onPinChange: (id: number, pinned: boolean) => Promise<void>;
  displayStyle?: DisplayStyle;
  selectedNoteIds?: Set<number>;
  onSelectionChange?: (id: number, selected: boolean) => void;
  t: Translator;
}

export function NoteList({
  notes,
  onUpdate,
  onDelete,
  onPinChange,
  displayStyle = "grid",
  selectedNoteIds,
  onSelectionChange,
  t,
}: NoteListProps) {
  if (notes.length === 0) {
    return (
      <div className="empty-state">
        📝 {t("noNotes")}
      </div>
    );
  }

  return (
    <div className={displayStyle === "list" ? "note-list-view" : "note-grid"}>
      {notes.map((note) => (
        <NoteCard
          key={note.id}
          note={note}
          onUpdate={onUpdate}
          onDelete={onDelete}
          onPinChange={onPinChange}
          selected={selectedNoteIds?.has(note.id) ?? false}
          onSelectionChange={onSelectionChange}
          t={t}
        />
      ))}
    </div>
  );
}
