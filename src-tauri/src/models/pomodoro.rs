use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomodoroSettings {
    pub work_minutes: i32,
    pub short_break_min: i32,
    pub long_break_min: i32,
    pub rounds_per_cycle: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayStat {
    pub date: String,
    pub count: i64,
}
