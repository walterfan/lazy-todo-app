export type NoteColor = 'yellow' | 'green' | 'blue' | 'pink' | 'purple' | 'orange';

export interface StickyNote {
  id: number;
  title: string;
  content: string;
  color: NoteColor;
  pinned: boolean;
  created_at: string;
  updated_at: string;
  file_path?: string | null;
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

export interface NoteTemplate {
  id: string;
  name: string;
  title: string;
  body: string;
  source: 'builtin' | 'file';
  path?: string | null;
}
