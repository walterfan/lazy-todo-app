import type { StickyNote, UpdateNote } from "../types/note";
import { NoteCard } from "./NoteCard";

interface NoteListProps {
  notes: StickyNote[];
  onUpdate: (input: UpdateNote) => Promise<void>;
  onDelete: (id: number) => Promise<void>;
}

export function NoteList({ notes, onUpdate, onDelete }: NoteListProps) {
  if (notes.length === 0) {
    return (
      <div className="empty-state">
        📝 No sticky notes yet. Create your first memo!
      </div>
    );
  }

  return (
    <div className="note-grid">
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
