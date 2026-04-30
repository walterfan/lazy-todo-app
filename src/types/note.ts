export type NoteColor = 'yellow' | 'green' | 'blue' | 'pink' | 'purple' | 'orange';

export interface StickyNote {
  id: number;
  title: string;
  content: string;
  color: NoteColor;
  pinned: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateNote {
  title?: string;
  content?: string;
  color?: NoteColor;
}

export interface UpdateNote {
  id: number;
  title?: string;
  content?: string;
  color?: NoteColor;
}

export interface ExportNotesResult {
  folder: string;
  files: string[];
}
