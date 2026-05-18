use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub page_size: i32,
    pub note_page_size: i32,
    pub todo_display: String,  // "list" or "grid"
    pub note_display: String,  // "list" or "grid"
    pub note_template: String, // legacy default markdown template (preserved for migration)
    pub note_folder: String,   // folder path where notes are mirrored as Markdown files
    pub language: String,      // "en" or "zh"
    #[serde(default)]
    pub note_template_files: Vec<String>, // explicit list of template file paths
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            page_size: 10,
            note_page_size: 10,
            todo_display: "list".to_string(),
            note_display: "list".to_string(),
            note_template: String::new(),
            note_folder: String::new(),
            language: "en".to_string(),
            note_template_files: Vec::new(),
        }
    }
}
