export type DisplayStyle = "list" | "grid";
export type AppLanguage = "en" | "zh";

export interface AppSettings {
  page_size: number;
  note_page_size: number;
  todo_display: DisplayStyle;
  note_display: DisplayStyle;
  note_template: string;
  note_folder: string;
  language: AppLanguage;
}
