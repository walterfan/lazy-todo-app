use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub priority: i32, // 1=High, 2=Medium, 3=Low
    pub completed: bool,
    pub deadline: Option<String>, // ISO 8601 format
    pub recurrence: String,       // none, daily, weekly, monthly, yearly
    pub recurrence_anchor: Option<String>,
    pub recurrence_weekday: Option<i64>,   // 1=Monday, 7=Sunday
    pub recurrence_month_day: Option<i64>, // 1-31
    pub reminder_minutes_before: Option<i64>,
    pub reminder_due_at: Option<String>,
    pub reminder_state: String, // none, upcoming, due, reminded, missed, overdue
    pub last_reminded_at: Option<String>,
    pub last_reminded_deadline: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTodo {
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<i32>,
    pub deadline: Option<String>,
    pub recurrence: Option<String>,
    pub recurrence_weekday: Option<i64>,
    pub recurrence_month_day: Option<i64>,
    pub reminder_minutes_before: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTodo {
    pub id: i64,
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority: Option<i32>,
    pub deadline: Option<String>,
    pub clear_deadline: Option<bool>,
    pub recurrence: Option<String>,
    pub recurrence_weekday: Option<i64>,
    pub recurrence_month_day: Option<i64>,
    pub reminder_minutes_before: Option<i64>,
}
