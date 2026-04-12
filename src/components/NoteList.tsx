import type { StickyNote, UpdateNote } from "../types/note";
import type { DisplayStyle } from "../types/settings";
import { NoteCard } from "./NoteCard";

interface NoteListProps {
  notes: StickyNote[];
  onUpdate: (input: UpdateNote) => Promise<void>;
  onDelete: (id: number) => Promise<void>;
  displayStyle?: DisplayStyle;
}

export function NoteList({ notes, onUpdate, onDelete, displayStyle = "grid" }: NoteListProps) {
  if (notes.length === 0) {
    return (
      <div className="empty-state">
        📝 No sticky notes yet. Create your first memo!
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
        />
      ))}
    </div>
  );
}
