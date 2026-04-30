use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StickyNote {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub color: String,
    pub pinned: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateNote {
    pub title: Option<String>,
    pub content: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNote {
    pub id: i64,
    pub title: Option<String>,
    pub content: Option<String>,
    pub color: Option<String>,
}
