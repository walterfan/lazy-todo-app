use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub priority: i32, // 1=High, 2=Medium, 3=Low
    pub completed: bool,
    pub deadline: Option<String>, // ISO 8601 format
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTodo {
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<i32>,
    pub deadline: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTodo {
    pub id: i64,
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority: Option<i32>,
    pub deadline: Option<String>,
}
