use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseQueryInput {
    pub sql: String,
    pub db_path: Option<String>,
    pub max_rows: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DatabaseQueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub row_count: usize,
    pub truncated: bool,
    pub elapsed_ms: u128,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseExecuteInput {
    pub sql: String,
    pub db_path: Option<String>,
    #[serde(default)]
    pub commit: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DatabaseExecuteResult {
    pub statement_kind: String,
    pub rows_affected: usize,
    pub committed: bool,
    pub elapsed_ms: u128,
    pub backup_path: String,
}
