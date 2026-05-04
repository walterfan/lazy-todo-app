use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::secretary::SelectedAppContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentManifest {
    #[serde(alias = "plugin_id")]
    pub agent_id: String,
    #[serde(alias = "plugin_name")]
    pub agent_name: String,
    #[serde(alias = "plugin_version")]
    pub agent_version: String,
    pub author: String,
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub create_time: String,
    pub update_time: String,
    pub min_app_version: String,
    pub rag_enabled: bool,
    pub is_multi_agent_supported: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub temperature: f64,
    pub top_p: f64,
    pub rag_top_k: i64,
    pub embedding_dim: i64,
    pub response_style: String,
    #[serde(default)]
    pub ban_topics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentValidationDiagnostic {
    pub severity: String,
    pub field: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    pub agent_id: String,
    pub agent_name: String,
    pub agent_version: String,
    pub author: String,
    pub description: String,
    pub tags: Vec<String>,
    pub path: String,
    pub avatar_path: String,
    pub readme_path: String,
    pub bundled: bool,
    pub enabled: bool,
    pub lifecycle_state: String,
    pub rag_enabled: bool,
    pub is_multi_agent_supported: bool,
    pub has_rag_knowledge: bool,
    pub validation_diagnostics: Vec<AgentValidationDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinitionDetail {
    pub agent: AgentDefinition,
    pub manifest: Option<AgentManifest>,
    pub config: Option<AgentConfig>,
    pub system_prompt: String,
    pub readme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDirectorySettings {
    pub agent_directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveAgentDirectorySettings {
    pub agent_directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallAgentZipInput {
    pub zip_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSafeFileRootSettings {
    pub safe_file_roots: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveAgentSafeFileRootSettings {
    #[serde(default)]
    pub safe_file_roots: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRagChunk {
    pub chunk_id: String,
    pub agent_id: String,
    pub agent_version: String,
    pub source_hash: String,
    pub embedding_model: String,
    pub embedding_dim: i64,
    pub chunk_text: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRagStatus {
    pub agent_id: String,
    pub rag_enabled: bool,
    pub has_rag_knowledge: bool,
    pub indexed_chunks: usize,
    pub source_hash: Option<String>,
    pub current_source_hash: Option<String>,
    pub stale: bool,
    #[serde(default)]
    pub stale_reasons: Vec<String>,
    pub vector_search_available: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMigrationStatus {
    pub migration_id: String,
    pub status: String,
    pub details: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub message_id: String,
    pub session_id: String,
    pub sender_type: i64,
    pub agent_id: Option<String>,
    pub content: String,
    pub turn_index: i64,
    pub stream_status: String,
    pub error_text: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    pub session_id: String,
    pub session_type: i64,
    pub agent_ids: Vec<String>,
    pub session_title: String,
    pub memory_enabled: bool,
    pub messages: Vec<AgentMessage>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentUserIdentity {
    pub display_name: String,
    pub preferred_language: String,
    pub communication_style: String,
    pub roles: Vec<String>,
    pub goals: Vec<String>,
    pub boundaries: String,
    pub important_facts: String,
    pub enabled: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveAgentUserIdentity {
    pub display_name: String,
    pub preferred_language: String,
    pub communication_style: String,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub goals: Vec<String>,
    pub boundaries: String,
    pub important_facts: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMemory {
    pub memory_id: String,
    pub content: String,
    pub scope: String,
    pub agent_id: Option<String>,
    pub status: String,
    pub pinned: bool,
    pub source_session_id: Option<String>,
    pub source_agent_id: Option<String>,
    pub source_message_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMemoryProposal {
    pub proposal_id: String,
    pub source_session_id: Option<String>,
    pub source_agent_id: Option<String>,
    pub source_message_id: Option<String>,
    pub proposed_text: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConversationSummary {
    pub summary_id: String,
    pub session_id: String,
    pub agent_id: Option<String>,
    pub title: String,
    pub summary: String,
    pub topics: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveAgentMemory {
    pub memory_id: Option<String>,
    pub content: String,
    pub scope: String,
    pub agent_id: Option<String>,
    pub status: Option<String>,
    pub pinned: Option<bool>,
    pub source_session_id: Option<String>,
    pub source_agent_id: Option<String>,
    pub source_message_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmAgentMemoryProposalInput {
    pub proposal_id: String,
    pub accepted: bool,
    pub content: Option<String>,
    pub scope: Option<String>,
    pub agent_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentUsedContext {
    #[serde(default)]
    pub todos: Vec<i64>,
    #[serde(default)]
    pub milestones: Vec<usize>,
    #[serde(default)]
    pub notes: Vec<i64>,
    #[serde(default)]
    pub memories: Vec<String>,
    #[serde(default)]
    pub rag_chunks: Vec<String>,
    #[serde(default)]
    pub conversation_summaries: Vec<String>,
    #[serde(default)]
    pub previous_messages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendAgentMessageInput {
    pub session_id: Option<String>,
    pub agent_id: String,
    pub message: String,
    #[serde(default)]
    pub selected_context: SelectedAppContext,
    pub stream_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendAgentGroupMessageInput {
    pub session_id: Option<String>,
    #[serde(default)]
    pub agent_ids: Vec<String>,
    pub message: String,
    #[serde(default)]
    pub selected_context: SelectedAppContext,
    pub stream_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendAgentMessageResult {
    pub session: AgentSession,
    pub assistant_message: AgentMessage,
    pub used_context: AgentUsedContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBuiltinTool {
    pub name: String,
    pub description: String,
    pub permission_category: String,
    pub safety_class: String,
    pub requires_confirmation: bool,
    pub argument_schema: Value,
    pub result_schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentToolCallInput {
    pub session_id: Option<String>,
    pub agent_id: Option<String>,
    pub tool_name: String,
    #[serde(default)]
    pub arguments: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentToolCallResult {
    pub audit_id: String,
    pub action_id: Option<String>,
    pub tool_name: String,
    pub status: String,
    pub requires_confirmation: bool,
    pub result: Value,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentToolAction {
    pub action_id: String,
    pub session_id: Option<String>,
    pub agent_id: Option<String>,
    pub tool_name: String,
    pub arguments: Value,
    pub preview: Value,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmAgentToolActionInput {
    pub action_id: String,
    pub accepted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmAgentToolActionResult {
    pub action_id: String,
    pub accepted: bool,
    pub tool_name: String,
    pub status: String,
    pub result: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExternalCliTool {
    pub tool_id: String,
    pub display_name: String,
    pub executable: String,
    #[serde(default)]
    pub allowed_subcommands: Vec<String>,
    #[serde(default)]
    pub argument_schema: Value,
    pub working_directory: String,
    #[serde(default)]
    pub environment_allowlist: Vec<String>,
    pub timeout_ms: i64,
    pub output_limit: i64,
    pub safety_class: String,
    pub enabled: bool,
    pub available: bool,
    pub availability_error: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveAgentExternalCliTool {
    pub tool_id: Option<String>,
    pub display_name: String,
    pub executable: String,
    #[serde(default)]
    pub allowed_subcommands: Vec<String>,
    #[serde(default)]
    pub argument_schema: Value,
    pub working_directory: String,
    #[serde(default)]
    pub environment_allowlist: Vec<String>,
    pub timeout_ms: i64,
    pub output_limit: i64,
    pub safety_class: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExternalCliToolTestResult {
    pub executable: String,
    pub resolved_path: String,
    pub available: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExternalCliCallInput {
    pub session_id: Option<String>,
    pub agent_id: Option<String>,
    pub tool_id: String,
    #[serde(default)]
    pub arguments: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExternalCliCallResult {
    pub audit_id: String,
    pub tool_id: String,
    pub status: String,
    pub confirmation_status: String,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: i64,
    pub timed_out: bool,
    pub truncated: bool,
    pub message: String,
}
