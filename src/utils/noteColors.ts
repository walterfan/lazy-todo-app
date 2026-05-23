import type { NoteColor } from "../types/note";

export const NOTE_COLORS: NoteColor[] = ["yellow", "green", "blue", "pink", "purple", "orange"];
export const DEFAULT_NOTE_EDITOR_COLOR: NoteColor = "green";
export const NOTE_EDITOR_FIELD_TEXT_COLOR = "#1f2933";

const NOTE_EDITOR_FIELD_BACKGROUNDS: Record<NoteColor, string> = {
  yellow: "#fdd835",
  green: "#9acd32",
  blue: "#42a5f5",
  pink: "#ec407a",
  purple: "#ab47bc",
  orange: "#ffa726",
};

export function getNoteEditorFieldBackground(color: NoteColor): string {
  return NOTE_EDITOR_FIELD_BACKGROUNDS[color];
}
