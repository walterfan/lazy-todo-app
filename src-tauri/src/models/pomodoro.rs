use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomodoroMilestone {
    pub name: String,
    pub deadline: String,
    #[serde(default = "default_milestone_status")]
    pub status: String,
}

fn default_milestone_status() -> String {
    "active".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomodoroSettings {
    pub work_minutes: i32,
    pub short_break_min: i32,
    pub long_break_min: i32,
    pub rounds_per_cycle: i32,
    pub milestones: Vec<PomodoroMilestone>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayStat {
    pub date: String,
    pub count: i64,
}
