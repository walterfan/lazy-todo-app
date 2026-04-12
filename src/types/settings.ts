export type DisplayStyle = "list" | "grid";

export interface AppSettings {
  page_size: number;
  todo_display: DisplayStyle;
  note_display: DisplayStyle;
  note_template: string;
  note_folder: string;
}
