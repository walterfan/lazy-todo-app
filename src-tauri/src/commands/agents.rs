use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::{IpAddr, ToSocketAddrs};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use chrono::Local;
use serde::Serialize;
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::db::Database;
use crate::models::agents::{
    AgentBuiltinTool, AgentConfig, AgentConversationSummary, AgentExternalCliCallInput,
    AgentExternalCliCallResult, AgentExternalCliTool, AgentExternalCliToolTestResult,
    AgentManifest, AgentMemory, AgentMemoryProposal, AgentMessage, AgentMigrationStatus,
    AgentPlugin, AgentPluginDetail, AgentPluginDirectorySettings, AgentRagChunk, AgentRagStatus,
    AgentSafeFileRootSettings, AgentSession, AgentToolAction, AgentToolCallInput,
    AgentToolCallResult, AgentUsedContext, AgentUserIdentity, AgentValidationDiagnostic,
    ConfirmAgentMemoryProposalInput, ConfirmAgentToolActionInput, ConfirmAgentToolActionResult,
    InstallAgentPluginZipInput, SaveAgentExternalCliTool, SaveAgentMemory,
    SaveAgentPluginDirectorySettings, SaveAgentSafeFileRootSettings, SaveAgentUserIdentity,
    SendAgentGroupMessageInput, SendAgentMessageInput, SendAgentMessageResult,
};
use crate::models::secretary::{
    EffectiveLlmSettings, MilestoneContext, NoteContext, SecretaryAppContext, SecretaryMessage,
    SelectedAppContext, TodoContext,
};

const REQUIRED_PLUGIN_FILES: [&str; 5] = [
    "manifest.json",
    "system_prompt.md",
    "config.json",
    "avatar.png",
    "README.md",
];
const MAX_TEXT_FILE_BYTES: u64 = 512 * 1024;
const MAX_AVATAR_BYTES: u64 = 2 * 1024 * 1024;
const MAX_NOTE_CHARS: usize = 1200;
const MAX_PROMPT_SECTION_CHARS: usize = 4000;
const MAX_PREVIOUS_MESSAGE_CHARS: usize = 500;
const MAX_FILE_TOOL_BYTES: u64 = 256 * 1024;
const MAX_WEB_FETCH_BYTES: usize = 512 * 1024;
const MAX_WEB_FETCH_TEXT_CHARS: usize = 120_000;
const WEB_FETCH_TIMEOUT_SECS: u64 = 15;
const MAX_WEB_FETCH_REDIRECTS: usize = 5;
const MIN_CLI_TIMEOUT_MS: i64 = 1_000;
const MAX_CLI_TIMEOUT_MS: i64 = 300_000;
const MIN_CLI_OUTPUT_LIMIT: i64 = 1_024;
const MAX_CLI_OUTPUT_LIMIT: i64 = 65_536;
const RAG_CHUNK_TARGET_CHARS: usize = 900;
const RAG_CHUNK_OVERLAP_CHARS: usize = 120;
const MAX_AGENT_TOOL_ROUNDS: usize = 2;
const MAX_PLUGIN_ZIP_ENTRY_BYTES: u64 = 8 * 1024 * 1024;
const SECRETARY_MIGRATION_ID: &str = "secretary_to_agents_v1";

#[derive(Clone, Serialize)]
struct AgentStreamChunk {
    stream_id: String,
    agent_id: Option<String>,
    content: String,
}

#[derive(Clone, Serialize)]
struct AgentStreamError {
    stream_id: String,
    error: String,
}

#[derive(Clone, Serialize)]
struct AgentStreamFinish {
    stream_id: String,
    result: SendAgentMessageResult,
}

#[derive(Debug, Clone, Default)]
struct LlmStreamOutput {
    content: String,
    tool_calls: Vec<LlmToolCall>,
}

#[derive(Debug, Clone)]
struct LlmToolCall {
    id: String,
    name: String,
    arguments: String,
}

#[derive(Debug, Clone, Default)]
struct PartialLlmToolCall {
    id: String,
    name: String,
    arguments: String,
}

#[derive(Debug, Clone, Default)]
struct LlmStreamEvent {
    content: Option<String>,
    tool_calls: Vec<(usize, PartialLlmToolCall)>,
}

#[tauri::command]
pub fn get_agent_plugin_directory_settings(
    db: State<'_, Database>,
) -> Result<AgentPluginDirectorySettings, String> {
    db.get_agent_plugin_directory_settings()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_agent_plugin_directory_settings(
    db: State<'_, Database>,
    input: SaveAgentPluginDirectorySettings,
) -> Result<AgentPluginDirectorySettings, String> {
    db.save_agent_plugin_directory_settings(&input)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_agent_safe_file_root_settings(
    db: State<'_, Database>,
) -> Result<AgentSafeFileRootSettings, String> {
    db.get_agent_safe_file_root_settings()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_agent_safe_file_root_settings(
    db: State<'_, Database>,
    input: SaveAgentSafeFileRootSettings,
) -> Result<AgentSafeFileRootSettings, String> {
    let safe_file_roots = normalize_safe_roots(input.safe_file_roots)?;
    db.save_agent_safe_file_root_settings(&SaveAgentSafeFileRootSettings { safe_file_roots })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_agent_migration_status(db: State<'_, Database>) -> Result<AgentMigrationStatus, String> {
    db.get_agent_migration_status(SECRETARY_MIGRATION_ID)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn run_agent_secretary_migration(
    db: State<'_, Database>,
) -> Result<AgentMigrationStatus, String> {
    migrate_secretary_to_agents(&db)
}

#[tauri::command]
pub fn list_agent_external_cli_tools(
    db: State<'_, Database>,
) -> Result<Vec<AgentExternalCliTool>, String> {
    let tools = db
        .list_agent_external_cli_tools()
        .map_err(|e| e.to_string())?;
    Ok(tools.into_iter().map(annotate_cli_availability).collect())
}

#[tauri::command]
pub fn save_agent_external_cli_tool(
    db: State<'_, Database>,
    input: SaveAgentExternalCliTool,
) -> Result<AgentExternalCliTool, String> {
    let input = normalize_external_cli_tool(input)?;
    validate_external_cli_registration(&input)?;
    let tool = db
        .save_agent_external_cli_tool(&input)
        .map_err(|e| e.to_string())?;
    Ok(annotate_cli_availability(tool))
}

#[tauri::command]
pub fn set_agent_external_cli_tool_enabled(
    db: State<'_, Database>,
    tool_id: String,
    enabled: bool,
) -> Result<AgentExternalCliTool, String> {
    if enabled {
        let tool = db
            .get_agent_external_cli_tool(tool_id.trim())
            .map_err(|e| e.to_string())?;
        validate_cli_executable(&tool.executable)?;
    }
    let tool = db
        .set_agent_external_cli_tool_enabled(tool_id.trim(), enabled)
        .map_err(|e| e.to_string())?;
    Ok(annotate_cli_availability(tool))
}

#[tauri::command]
pub fn delete_agent_external_cli_tool(
    db: State<'_, Database>,
    tool_id: String,
) -> Result<(), String> {
    db.delete_agent_external_cli_tool(tool_id.trim())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn test_agent_external_cli_tool_registration(
    input: SaveAgentExternalCliTool,
) -> Result<AgentExternalCliToolTestResult, String> {
    let input = normalize_external_cli_tool(input)?;
    validate_external_cli_registration(&input)?;
    let resolved_path = validate_cli_executable(&input.executable)?;
    Ok(AgentExternalCliToolTestResult {
        executable: input.executable,
        resolved_path: resolved_path.to_string_lossy().to_string(),
        available: true,
        message: "External CLI registration is available.".to_string(),
    })
}

#[tauri::command]
pub fn execute_agent_external_cli_tool(
    db: State<'_, Database>,
    input: AgentExternalCliCallInput,
) -> Result<AgentExternalCliCallResult, String> {
    execute_external_cli_tool(&db, input)
}

#[tauri::command]
pub fn install_agent_external_cli_presets(
    db: State<'_, Database>,
) -> Result<Vec<AgentExternalCliTool>, String> {
    install_external_cli_presets(&db)
}

#[tauri::command]
pub fn list_agents(app: AppHandle, db: State<'_, Database>) -> Result<Vec<AgentPlugin>, String> {
    scan_and_persist_agents(&app, &db)?;
    db.list_agent_plugins().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn refresh_agents(app: AppHandle, db: State<'_, Database>) -> Result<Vec<AgentPlugin>, String> {
    scan_and_persist_agents(&app, &db)?;
    db.list_agent_plugins().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_agent_enabled(
    db: State<'_, Database>,
    plugin_id: String,
    enabled: bool,
) -> Result<(), String> {
    db.set_agent_plugin_enabled(&plugin_id, enabled)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn uninstall_agent_plugin(
    app: AppHandle,
    db: State<'_, Database>,
    plugin_id: String,
) -> Result<Vec<AgentPlugin>, String> {
    scan_and_persist_agents(&app, &db)?;
    uninstall_plugin_by_id(&db, &plugin_id)?;
    scan_and_persist_agents(&app, &db)?;
    let _ = app.emit(
        "agent-plugins-changed",
        json!({ "plugin_id": plugin_id.trim() }),
    );
    db.list_agent_plugins().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn install_agent_plugin_zip(
    app: AppHandle,
    db: State<'_, Database>,
    input: InstallAgentPluginZipInput,
) -> Result<AgentPlugin, String> {
    let plugin = install_plugin_zip(&db, &input.zip_path)?;
    scan_and_persist_agents(&app, &db)?;
    let _ = app.emit(
        "agent-plugins-changed",
        json!({ "plugin_id": plugin.plugin_id }),
    );
    find_agent_plugin(&db, &plugin.plugin_id)
}

#[tauri::command]
pub fn get_agent_plugin_detail(
    app: AppHandle,
    db: State<'_, Database>,
    plugin_id: String,
) -> Result<AgentPluginDetail, String> {
    scan_and_persist_agents(&app, &db)?;
    let plugin = db
        .list_agent_plugins()
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|plugin| plugin.plugin_id == plugin_id)
        .ok_or_else(|| format!("Agent plugin not found: {plugin_id}"))?;
    let root = PathBuf::from(&plugin.path);
    let manifest = read_manifest(&root).ok();
    let config = read_config(&root).ok();
    let system_prompt = read_bounded_text(&root.join("system_prompt.md")).unwrap_or_default();
    let readme = read_bounded_text(&root.join("README.md")).unwrap_or_default();
    Ok(AgentPluginDetail {
        plugin,
        manifest,
        config,
        system_prompt,
        readme,
    })
}

#[tauri::command]
pub fn get_agent_rag_status(
    app: AppHandle,
    db: State<'_, Database>,
    plugin_id: String,
) -> Result<AgentRagStatus, String> {
    scan_and_persist_agents(&app, &db)?;
    let plugin = find_agent_plugin(&db, &plugin_id)?;
    build_rag_status(&db, &plugin)
}

#[tauri::command]
pub fn rebuild_agent_rag_index(
    app: AppHandle,
    db: State<'_, Database>,
    plugin_id: String,
) -> Result<AgentRagStatus, String> {
    scan_and_persist_agents(&app, &db)?;
    let plugin = find_agent_plugin(&db, &plugin_id)?;
    rebuild_rag_for_plugin(&db, &plugin)?;
    build_rag_status(&db, &plugin)
}

#[tauri::command]
pub fn rebuild_all_agent_rag_indexes(
    app: AppHandle,
    db: State<'_, Database>,
) -> Result<Vec<AgentRagStatus>, String> {
    scan_and_persist_agents(&app, &db)?;
    let plugins = db.list_agent_plugins().map_err(|e| e.to_string())?;
    let mut statuses = Vec::new();
    for plugin in plugins {
        if plugin.lifecycle_state == "invalid" {
            continue;
        }
        rebuild_rag_for_plugin(&db, &plugin)?;
        statuses.push(build_rag_status(&db, &plugin)?);
    }
    Ok(statuses)
}

#[tauri::command]
pub fn retrieve_agent_rag_chunks(
    app: AppHandle,
    db: State<'_, Database>,
    plugin_id: String,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<AgentRagChunk>, String> {
    scan_and_persist_agents(&app, &db)?;
    let plugin = find_agent_plugin(&db, &plugin_id)?;
    if !plugin.rag_enabled {
        return Ok(Vec::new());
    }
    retrieve_rag_chunks_for_plugin(&db, &plugin, &query, limit)
}

fn validate_agent_group_plugins(
    db: &Database,
    agent_ids: &[String],
) -> Result<Vec<AgentPlugin>, String> {
    let mut seen = HashSet::new();
    let mut plugins = Vec::new();
    for agent_id in agent_ids
        .iter()
        .map(|id| id.trim())
        .filter(|id| !id.is_empty())
    {
        if !seen.insert(agent_id.to_string()) {
            continue;
        }
        let plugin = find_agent_plugin(db, agent_id)?;
        if !plugin.enabled || plugin.lifecycle_state == "invalid" {
            return Err(format!("Agent is not available: {agent_id}"));
        }
        plugins.push(plugin);
    }
    if plugins.is_empty() {
        return Err("Select at least one Agent.".to_string());
    }
    if plugins.len() > 1 {
        for plugin in &plugins {
            if !plugin.is_multi_agent_supported {
                return Err(format!(
                    "Agent does not support group chat: {}",
                    plugin.plugin_id
                ));
            }
        }
    }
    Ok(plugins)
}

#[tauri::command]
pub fn start_agent_session(
    app: AppHandle,
    db: State<'_, Database>,
    agent_id: String,
) -> Result<AgentSession, String> {
    scan_and_persist_agents(&app, &db)?;
    let plugin = find_agent_plugin(&db, &agent_id)?;
    if !plugin.enabled || plugin.lifecycle_state == "invalid" {
        return Err(format!("Agent is not available: {agent_id}"));
    }
    let session = AgentSession {
        session_id: format!("agent-session-{}", Local::now().timestamp_millis()),
        session_type: 1,
        agent_ids: vec![agent_id],
        session_title: format!("Chat with {}", plugin.plugin_name),
        memory_enabled: true,
        messages: Vec::new(),
        created_at: String::new(),
        updated_at: String::new(),
    };
    db.save_agent_session(&session).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn start_agent_group_session(
    app: AppHandle,
    db: State<'_, Database>,
    agent_ids: Vec<String>,
) -> Result<AgentSession, String> {
    scan_and_persist_agents(&app, &db)?;
    let plugins = validate_agent_group_plugins(&db, &agent_ids)?;
    let plugin_names = plugins
        .iter()
        .map(|plugin| plugin.plugin_name.clone())
        .collect::<Vec<_>>();
    let session = AgentSession {
        session_id: format!("agent-session-{}", Local::now().timestamp_millis()),
        session_type: if plugins.len() > 1 { 2 } else { 1 },
        agent_ids: plugins
            .iter()
            .map(|plugin| plugin.plugin_id.clone())
            .collect(),
        session_title: if plugins.len() > 1 {
            format!("Group chat: {}", plugin_names.join(", "))
        } else {
            format!(
                "Chat with {}",
                plugin_names.first().cloned().unwrap_or_default()
            )
        },
        memory_enabled: true,
        messages: Vec::new(),
        created_at: String::new(),
        updated_at: String::new(),
    };
    db.save_agent_session(&session).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_agent_sessions(db: State<'_, Database>) -> Result<Vec<AgentSession>, String> {
    db.list_agent_sessions().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn load_agent_session(
    db: State<'_, Database>,
    session_id: String,
) -> Result<AgentSession, String> {
    db.get_agent_session(&session_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reset_agent_session(
    db: State<'_, Database>,
    session_id: String,
) -> Result<AgentSession, String> {
    db.reset_agent_session(session_id.trim())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_agent_session(
    db: State<'_, Database>,
    session_id: String,
) -> Result<Vec<AgentSession>, String> {
    db.delete_agent_session(session_id.trim())
        .map_err(|e| e.to_string())?;
    db.list_agent_sessions().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_agent_transcript(
    db: State<'_, Database>,
    session_id: String,
) -> Result<String, String> {
    export_transcript(&db, &session_id)
}

#[tauri::command]
pub fn save_agent_message_to_file(
    db: State<'_, Database>,
    message_id: String,
) -> Result<String, String> {
    save_message_to_markdown_file(&db, &message_id)
}

#[tauri::command]
pub fn delete_agent_message(
    db: State<'_, Database>,
    message_id: String,
) -> Result<AgentSession, String> {
    let message = db
        .get_agent_message(message_id.trim())
        .map_err(|e| e.to_string())?;
    db.delete_agent_message(message_id.trim())
        .map_err(|e| e.to_string())?;
    let _ = refresh_conversation_summary(&db, &message.session_id);
    db.get_agent_session(&message.session_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_agent_user_identity(db: State<'_, Database>) -> Result<AgentUserIdentity, String> {
    db.get_agent_user_identity().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_agent_user_identity(
    db: State<'_, Database>,
    input: SaveAgentUserIdentity,
) -> Result<AgentUserIdentity, String> {
    db.save_agent_user_identity(&input)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_agent_memories(
    db: State<'_, Database>,
    agent_id: Option<String>,
) -> Result<Vec<AgentMemory>, String> {
    db.list_agent_memories(agent_id.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_agent_memory(
    db: State<'_, Database>,
    input: SaveAgentMemory,
) -> Result<AgentMemory, String> {
    if input.content.trim().is_empty() {
        return Err("Memory content cannot be empty.".to_string());
    }
    db.save_agent_memory(&input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_agent_memory(db: State<'_, Database>, memory_id: String) -> Result<(), String> {
    db.delete_agent_memory(&memory_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_agent_memory_pinned(
    db: State<'_, Database>,
    memory_id: String,
    pinned: bool,
) -> Result<AgentMemory, String> {
    db.set_agent_memory_pinned(&memory_id, pinned)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_agent_memory_status(
    db: State<'_, Database>,
    memory_id: String,
    status: String,
) -> Result<AgentMemory, String> {
    db.set_agent_memory_status(&memory_id, &status)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_agent_memory_proposals(
    db: State<'_, Database>,
) -> Result<Vec<AgentMemoryProposal>, String> {
    db.list_agent_memory_proposals().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn confirm_agent_memory_proposal(
    db: State<'_, Database>,
    input: ConfirmAgentMemoryProposalInput,
) -> Result<AgentMemoryProposal, String> {
    confirm_memory_proposal(&db, input)
}

#[tauri::command]
pub fn get_agent_app_context(
    db: State<'_, Database>,
    selected_context: SelectedAppContext,
) -> Result<SecretaryAppContext, String> {
    build_agent_app_context(&db, &selected_context)
}

#[tauri::command]
pub fn list_agent_builtin_tools() -> Vec<AgentBuiltinTool> {
    builtin_tools()
}

#[tauri::command]
pub fn list_pending_agent_tool_actions(
    db: State<'_, Database>,
) -> Result<Vec<AgentToolAction>, String> {
    db.list_pending_agent_tool_actions()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn execute_agent_builtin_tool(
    db: State<'_, Database>,
    input: AgentToolCallInput,
) -> Result<AgentToolCallResult, String> {
    execute_builtin_tool(&db, input)
}

#[tauri::command]
pub fn confirm_agent_tool_action(
    db: State<'_, Database>,
    input: ConfirmAgentToolActionInput,
) -> Result<ConfirmAgentToolActionResult, String> {
    confirm_pending_tool_action(&db, input)
}

#[tauri::command]
pub async fn send_agent_message_stream(
    app: AppHandle,
    db: State<'_, Database>,
    input: SendAgentMessageInput,
) -> Result<SendAgentMessageResult, String> {
    let stream_id = input
        .stream_id
        .clone()
        .unwrap_or_else(|| format!("agent-{}", Local::now().timestamp_millis()));
    let result = send_agent_message_stream_inner(&app, db, input, &stream_id).await;
    match result {
        Ok(result) => {
            let _ = app.emit(
                "agent-stream-finish",
                AgentStreamFinish {
                    stream_id,
                    result: result.clone(),
                },
            );
            Ok(result)
        }
        Err(error) => {
            let _ = app.emit(
                "agent-stream-error",
                AgentStreamError {
                    stream_id,
                    error: error.clone(),
                },
            );
            Err(error)
        }
    }
}

#[tauri::command]
pub async fn send_agent_group_message_stream(
    app: AppHandle,
    db: State<'_, Database>,
    input: SendAgentGroupMessageInput,
) -> Result<SendAgentMessageResult, String> {
    let stream_id = input
        .stream_id
        .clone()
        .unwrap_or_else(|| format!("agent-group-{}", Local::now().timestamp_millis()));
    let result = send_agent_group_message_stream_inner(&app, db, input, &stream_id).await;
    match result {
        Ok(result) => {
            let _ = app.emit(
                "agent-stream-finish",
                AgentStreamFinish {
                    stream_id,
                    result: result.clone(),
                },
            );
            Ok(result)
        }
        Err(error) => {
            let _ = app.emit(
                "agent-stream-error",
                AgentStreamError {
                    stream_id,
                    error: error.clone(),
                },
            );
            Err(error)
        }
    }
}

fn builtin_tools() -> Vec<AgentBuiltinTool> {
    vec![
        read_tool(
            "read_note",
            "Read one sticky note by note_id, or list notes when note_id is omitted.",
            json!({
                "type": "object",
                "properties": {
                    "note_id": { "type": "integer" }
                }
            }),
            json!({ "type": "object", "properties": { "notes": { "type": "array" } } }),
        ),
        read_tool(
            "read_todo_list",
            "Read todo items through the app-owned Todo list path.",
            json!({
                "type": "object",
                "properties": {
                    "include_completed": { "type": "boolean" }
                }
            }),
            json!({ "type": "object", "properties": { "todos": { "type": "array" } } }),
        ),
        read_tool(
            "read_milestones",
            "Read Pomodoro milestones through the app-owned milestone path.",
            json!({ "type": "object", "properties": {} }),
            json!({ "type": "object", "properties": { "milestones": { "type": "array" } } }),
        ),
        read_tool(
            "web_fetch",
            "Fetch a public HTTP or HTTPS web page by URL and return extracted text for translation, analysis, or summarization. Local, private, and non-text targets are blocked.",
            json!({
                "type": "object",
                "required": ["url"],
                "properties": {
                    "url": { "type": "string" },
                    "max_chars": { "type": "integer", "minimum": 1, "maximum": MAX_WEB_FETCH_TEXT_CHARS }
                }
            }),
            json!({
                "type": "object",
                "properties": {
                    "url": { "type": "string" },
                    "final_url": { "type": "string" },
                    "status": { "type": "integer" },
                    "content_type": { "type": "string" },
                    "title": { "type": "string" },
                    "text": { "type": "string" },
                    "truncated": { "type": "boolean" }
                }
            }),
        ),
        write_tool(
            "write_note",
            "Propose a sticky note edit. The note is changed only after user confirmation.",
            json!({
                "type": "object",
                "required": ["note_id"],
                "properties": {
                    "note_id": { "type": "integer" },
                    "title": { "type": "string" },
                    "content": { "type": "string" },
                    "color": { "type": "string" }
                }
            }),
            json!({ "type": "object", "properties": { "note": { "type": "object" } } }),
        ),
        write_tool(
            "add_todo_item",
            "Propose adding a todo item. The item is created only after user confirmation.",
            json!({
                "type": "object",
                "required": ["title"],
                "properties": {
                    "title": { "type": "string" },
                    "description": { "type": "string" },
                    "priority": { "type": "integer", "minimum": 1, "maximum": 3 },
                    "deadline": { "type": "string" }
                }
            }),
            json!({ "type": "object", "properties": { "todo": { "type": "object" } } }),
        ),
        write_tool(
            "change_todo_item",
            "Propose changing a todo item. The item is changed only after user confirmation.",
            json!({
                "type": "object",
                "required": ["todo_id"],
                "properties": {
                    "todo_id": { "type": "integer" },
                    "title": { "type": "string" },
                    "description": { "type": "string" },
                    "priority": { "type": "integer", "minimum": 1, "maximum": 3 },
                    "deadline": { "type": "string" },
                    "completed": { "type": "boolean" }
                }
            }),
            json!({ "type": "object", "properties": { "todo": { "type": "object" } } }),
        ),
        write_tool(
            "change_milestone",
            "Propose changing a Pomodoro milestone. The milestone is changed only after user confirmation.",
            json!({
                "type": "object",
                "required": ["index"],
                "properties": {
                    "index": { "type": "integer" },
                    "name": { "type": "string" },
                    "deadline": { "type": "string" },
                    "status": { "type": "string", "enum": ["active", "completed", "cancelled"] }
                }
            }),
            json!({ "type": "object", "properties": { "milestone": { "type": "object" } } }),
        ),
        write_tool(
            "propose_memory",
            "Propose a durable app memory. The memory is stored only after user confirmation.",
            json!({
                "type": "object",
                "required": ["proposed_text"],
                "properties": {
                    "proposed_text": { "type": "string" }
                }
            }),
            json!({ "type": "object", "properties": { "proposal": { "type": "object" } } }),
        ),
        AgentBuiltinTool {
            name: "read_file".to_string(),
            description: "Read a UTF-8 text file from a configured safe root with size and binary safeguards.".to_string(),
            permission_category: "file".to_string(),
            safety_class: "read".to_string(),
            requires_confirmation: false,
            argument_schema: json!({
                "type": "object",
                "required": ["path"],
                "properties": {
                    "path": { "type": "string" }
                }
            }),
            result_schema: json!({ "type": "object", "properties": { "content": { "type": "string" } } }),
        },
        AgentBuiltinTool {
            name: "write_file".to_string(),
            description: "Propose writing a UTF-8 text file under a configured safe root. The file is changed only after user confirmation.".to_string(),
            permission_category: "file".to_string(),
            safety_class: "write".to_string(),
            requires_confirmation: true,
            argument_schema: json!({
                "type": "object",
                "required": ["path", "content"],
                "properties": {
                    "path": { "type": "string" },
                    "content": { "type": "string" }
                }
            }),
            result_schema: json!({ "type": "object", "properties": { "path": { "type": "string" } } }),
        },
    ]
}

fn read_tool(
    name: &str,
    description: &str,
    argument_schema: Value,
    result_schema: Value,
) -> AgentBuiltinTool {
    AgentBuiltinTool {
        name: name.to_string(),
        description: description.to_string(),
        permission_category: "app_context".to_string(),
        safety_class: "read".to_string(),
        requires_confirmation: false,
        argument_schema,
        result_schema,
    }
}

fn write_tool(
    name: &str,
    description: &str,
    argument_schema: Value,
    result_schema: Value,
) -> AgentBuiltinTool {
    AgentBuiltinTool {
        name: name.to_string(),
        description: description.to_string(),
        permission_category: "app_write".to_string(),
        safety_class: "write".to_string(),
        requires_confirmation: true,
        argument_schema,
        result_schema,
    }
}

fn execute_builtin_tool(
    db: &Database,
    input: AgentToolCallInput,
) -> Result<AgentToolCallResult, String> {
    let tool_name = input.tool_name.trim();
    if !builtin_tools().iter().any(|tool| tool.name == tool_name) {
        return Err(format!("Unknown Agent built-in tool: {tool_name}"));
    }
    match tool_name {
        "read_note" => finish_tool_call(
            db,
            &input,
            None,
            "completed",
            read_note_tool(db, &input.arguments)?,
        ),
        "read_todo_list" => finish_tool_call(
            db,
            &input,
            None,
            "completed",
            read_todo_list_tool(db, &input.arguments)?,
        ),
        "read_milestones" => {
            finish_tool_call(db, &input, None, "completed", read_milestones_tool(db)?)
        }
        "web_fetch" => finish_tool_call(
            db,
            &input,
            None,
            "completed",
            web_fetch_tool(&input.arguments)?,
        ),
        "read_file" => finish_tool_call(
            db,
            &input,
            None,
            "completed",
            read_file_tool(db, &input.arguments)?,
        ),
        "write_note" | "add_todo_item" | "change_todo_item" | "change_milestone" | "write_file" => {
            let action = create_pending_tool_action(db, &input)?;
            let result = json!({
                "action_id": action.action_id,
                "preview": action.preview,
            });
            finish_tool_call(
                db,
                &input,
                Some(&action.action_id),
                "pending_confirmation",
                result,
            )
        }
        "propose_memory" => {
            let proposal = create_memory_proposal(
                db,
                input.session_id.clone(),
                input.agent_id.clone(),
                None,
                required_string(&input.arguments, "proposed_text")?,
            )?;
            finish_tool_call(
                db,
                &input,
                None,
                "pending_confirmation",
                json!({ "proposal": proposal }),
            )
        }
        _ => Err(format!("Unsupported Agent built-in tool: {tool_name}")),
    }
}

fn finish_tool_call(
    db: &Database,
    input: &AgentToolCallInput,
    action_id: Option<&str>,
    status: &str,
    result: Value,
) -> Result<AgentToolCallResult, String> {
    let audit_id = format!(
        "agent-tool-audit-{}",
        Local::now().timestamp_nanos_opt().unwrap_or_default()
    );
    db.insert_agent_builtin_tool_audit(
        &audit_id,
        input.session_id.as_deref(),
        input.agent_id.as_deref(),
        input.tool_name.trim(),
        action_id,
        &input.arguments,
        status,
        &result,
    )
    .map_err(|e| e.to_string())?;
    let requires_confirmation = status == "pending_confirmation";
    Ok(AgentToolCallResult {
        audit_id,
        action_id: action_id.map(str::to_string),
        tool_name: input.tool_name.trim().to_string(),
        status: status.to_string(),
        requires_confirmation,
        result,
        message: if requires_confirmation {
            "Tool call created a proposed action and is waiting for user confirmation.".to_string()
        } else {
            "Tool call completed.".to_string()
        },
    })
}

fn read_note_tool(db: &Database, arguments: &Value) -> Result<Value, String> {
    let notes = db.list_notes().map_err(|e| e.to_string())?;
    let note_id = optional_i64(arguments, "note_id")?;
    let mut notes = notes
        .into_iter()
        .filter(|note| note_id.map(|id| note.id == id).unwrap_or(true))
        .map(|mut note| {
            if note.content.chars().count() > MAX_NOTE_CHARS {
                note.content = truncate_chars(&note.content, MAX_NOTE_CHARS);
            }
            note
        })
        .collect::<Vec<_>>();
    if note_id.is_some() && notes.is_empty() {
        return Err(format!(
            "Sticky note not found: {}",
            note_id.unwrap_or_default()
        ));
    }
    notes.truncate(20);
    Ok(json!({ "notes": notes }))
}

fn read_todo_list_tool(db: &Database, arguments: &Value) -> Result<Value, String> {
    let include_completed = arguments
        .get("include_completed")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let todos = db
        .list_todos()
        .map_err(|e| e.to_string())?
        .into_iter()
        .filter(|todo| include_completed || !todo.completed)
        .collect::<Vec<_>>();
    Ok(json!({ "todos": todos }))
}

fn read_milestones_tool(db: &Database) -> Result<Value, String> {
    let milestones = db
        .get_pomodoro_settings()
        .map_err(|e| e.to_string())?
        .milestones
        .into_iter()
        .enumerate()
        .map(|(index, milestone)| MilestoneContext::from_milestone(index, milestone))
        .collect::<Vec<_>>();
    Ok(json!({ "milestones": milestones }))
}

fn web_fetch_tool(arguments: &Value) -> Result<Value, String> {
    let requested_url = required_raw_string(arguments, "url")?;
    let mut url = parse_web_fetch_url(&requested_url)?;
    let max_chars = optional_i64(arguments, "max_chars")?
        .unwrap_or(MAX_WEB_FETCH_TEXT_CHARS as i64)
        .clamp(1, MAX_WEB_FETCH_TEXT_CHARS as i64) as usize;
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(WEB_FETCH_TIMEOUT_SECS))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent(format!(
            "LazyTodoApp/{} AgentWebFetch",
            env!("CARGO_PKG_VERSION")
        ))
        .build()
        .map_err(|e| format!("Cannot create web fetch client: {e}"))?;

    let mut redirects = 0usize;
    loop {
        validate_web_fetch_url(&url)?;
        let response = client
            .get(url.clone())
            .header(
                reqwest::header::ACCEPT,
                "text/html,application/xhtml+xml,application/xml;q=0.9,text/plain;q=0.8,application/json;q=0.7,*/*;q=0.1",
            )
            .send()
            .map_err(|e| format!("web_fetch request failed: {e}"))?;
        let status = response.status();
        if status.is_redirection() {
            redirects += 1;
            if redirects > MAX_WEB_FETCH_REDIRECTS {
                return Err(format!(
                    "web_fetch followed more than {MAX_WEB_FETCH_REDIRECTS} redirects."
                ));
            }
            let location = response
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|value| value.to_str().ok())
                .ok_or_else(|| {
                    "web_fetch redirect did not include a Location header.".to_string()
                })?;
            url = url
                .join(location)
                .map_err(|e| format!("web_fetch redirect URL is invalid: {e}"))?;
            continue;
        }

        if !status.is_success() {
            return Err(format!("web_fetch failed with HTTP status {status}."));
        }
        let final_url = response.url().to_string();
        validate_web_fetch_url(response.url())?;
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_string();
        validate_web_fetch_content_type(&content_type)?;
        if let Some(length) = response.content_length() {
            if length > MAX_WEB_FETCH_BYTES as u64 {
                return Err(format!(
                    "web_fetch response is too large: {length} bytes > {MAX_WEB_FETCH_BYTES} bytes"
                ));
            }
        }

        let mut bytes = Vec::new();
        let mut reader = response.take((MAX_WEB_FETCH_BYTES + 1) as u64);
        reader
            .read_to_end(&mut bytes)
            .map_err(|e| format!("Cannot read web_fetch response: {e}"))?;
        let body_truncated = bytes.len() > MAX_WEB_FETCH_BYTES;
        if body_truncated {
            bytes.truncate(MAX_WEB_FETCH_BYTES);
        }
        if bytes.contains(&0) {
            return Err("web_fetch response appears to be binary.".to_string());
        }
        let body = String::from_utf8_lossy(&bytes).to_string();
        let is_html =
            content_type.to_ascii_lowercase().contains("html") || looks_like_html_document(&body);
        let title = if is_html {
            extract_html_title(&body)
        } else {
            String::new()
        };
        let text = if is_html {
            extract_html_text(&body)
        } else {
            normalize_extracted_text(&body)
        };
        let text_truncated = text.chars().count() > max_chars;
        let text = if text_truncated {
            truncate_chars(&text, max_chars)
        } else {
            text
        };

        return Ok(json!({
            "url": requested_url.trim(),
            "final_url": final_url,
            "status": status.as_u16(),
            "content_type": content_type,
            "title": title,
            "text": text,
            "bytes": bytes.len(),
            "truncated": body_truncated || text_truncated,
        }));
    }
}

fn web_fetch_url_from_user_message(message: &str) -> Option<String> {
    message.split_whitespace().find_map(|token| {
        let candidate = token
            .trim_matches(|ch: char| {
                matches!(
                    ch,
                    '<' | '>' | '"' | '\'' | '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';'
                )
            })
            .trim_end_matches(|ch: char| matches!(ch, '.' | '?' | '!' | ':' | ',' | ';'));
        if candidate.starts_with("http://") || candidate.starts_with("https://") {
            Some(candidate.to_string())
        } else {
            None
        }
    })
}

fn parse_web_fetch_url(raw_url: &str) -> Result<reqwest::Url, String> {
    let url = reqwest::Url::parse(raw_url.trim())
        .map_err(|e| format!("web_fetch URL is invalid: {e}"))?;
    validate_web_fetch_url(&url)?;
    Ok(url)
}

fn validate_web_fetch_url(url: &reqwest::Url) -> Result<(), String> {
    match url.scheme() {
        "http" | "https" => {}
        scheme => {
            return Err(format!(
                "web_fetch only supports http and https URLs, not {scheme}."
            ))
        }
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("web_fetch URLs must not include credentials.".to_string());
    }
    let host = url
        .host_str()
        .ok_or_else(|| "web_fetch URL must include a host.".to_string())?
        .trim()
        .to_ascii_lowercase();
    if host == "localhost" || host.ends_with(".localhost") {
        return Err("web_fetch blocks localhost targets.".to_string());
    }
    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_blocked_web_fetch_ip(ip) {
            return Err("web_fetch blocks local, private, and link-local addresses.".to_string());
        }
        return Ok(());
    }
    let port = url.port_or_known_default().unwrap_or(80);
    let addresses = (host.as_str(), port)
        .to_socket_addrs()
        .map_err(|e| format!("Cannot resolve web_fetch host `{host}`: {e}"))?
        .collect::<Vec<_>>();
    if addresses.is_empty() {
        return Err(format!("Cannot resolve web_fetch host `{host}`."));
    }
    if addresses
        .iter()
        .any(|address| is_blocked_web_fetch_ip(address.ip()))
    {
        return Err(
            "web_fetch blocks hosts that resolve to local, private, or link-local addresses."
                .to_string(),
        );
    }
    Ok(())
}

fn is_blocked_web_fetch_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || ip.is_broadcast()
                || ip.is_unspecified()
        }
        IpAddr::V6(ip) => {
            let first = ip.segments()[0];
            ip.is_loopback()
                || ip.is_unspecified()
                || (first & 0xfe00) == 0xfc00
                || (first & 0xffc0) == 0xfe80
        }
    }
}

fn validate_web_fetch_content_type(content_type: &str) -> Result<(), String> {
    if content_type.trim().is_empty() {
        return Ok(());
    }
    let content_type = content_type.to_ascii_lowercase();
    let allowed = content_type.starts_with("text/")
        || content_type.contains("html")
        || content_type.contains("xml")
        || content_type.contains("json")
        || content_type.contains("rss")
        || content_type.contains("atom");
    if allowed {
        Ok(())
    } else {
        Err(format!(
            "web_fetch only accepts text-like responses, got `{content_type}`."
        ))
    }
}

fn looks_like_html_document(value: &str) -> bool {
    let lower = value
        .chars()
        .take(512)
        .collect::<String>()
        .to_ascii_lowercase();
    lower.contains("<html") || lower.contains("<!doctype html") || lower.contains("<body")
}

fn extract_html_title(html: &str) -> String {
    let lower = html.to_ascii_lowercase();
    let Some(start) = lower.find("<title") else {
        return String::new();
    };
    let Some(open_end) = lower[start..].find('>').map(|index| start + index + 1) else {
        return String::new();
    };
    let Some(close) = lower[open_end..]
        .find("</title>")
        .map(|index| open_end + index)
    else {
        return String::new();
    };
    normalize_extracted_text(&decode_html_entities(&html[open_end..close]))
}

fn extract_html_text(html: &str) -> String {
    let without_comments = remove_html_comments(html);
    let without_hidden = remove_html_element(&without_comments, "script");
    let without_hidden = remove_html_element(&without_hidden, "style");
    let without_hidden = remove_html_element(&without_hidden, "noscript");
    let without_hidden = remove_html_element(&without_hidden, "svg");
    let mut text = String::with_capacity(without_hidden.len());
    let mut in_tag = false;
    let mut tag = String::new();
    for ch in without_hidden.chars() {
        if in_tag {
            if ch == '>' {
                if is_html_block_tag(&tag) {
                    text.push('\n');
                }
                tag.clear();
                in_tag = false;
            } else if tag.len() < 32 {
                tag.push(ch.to_ascii_lowercase());
            }
            continue;
        }
        if ch == '<' {
            in_tag = true;
            tag.clear();
        } else {
            text.push(ch);
        }
    }
    normalize_extracted_text(&decode_html_entities(&text))
}

fn remove_html_comments(html: &str) -> String {
    let mut output = String::with_capacity(html.len());
    let mut remainder = html;
    while let Some(start) = remainder.find("<!--") {
        output.push_str(&remainder[..start]);
        let after_start = &remainder[start + 4..];
        if let Some(end) = after_start.find("-->") {
            remainder = &after_start[end + 3..];
        } else {
            return output;
        }
    }
    output.push_str(remainder);
    output
}

fn remove_html_element(html: &str, tag_name: &str) -> String {
    let lower = html.to_ascii_lowercase();
    let mut output = String::with_capacity(html.len());
    let mut cursor = 0usize;
    let open_pattern = format!("<{tag_name}");
    let close_pattern = format!("</{tag_name}>");
    while let Some(relative_start) = lower[cursor..].find(&open_pattern) {
        let start = cursor + relative_start;
        output.push_str(&html[cursor..start]);
        if let Some(relative_end) = lower[start..].find(&close_pattern) {
            cursor = start + relative_end + close_pattern.len();
        } else {
            cursor = html.len();
            break;
        }
    }
    output.push_str(&html[cursor..]);
    output
}

fn is_html_block_tag(tag: &str) -> bool {
    let tag = tag
        .trim_start_matches('/')
        .split_whitespace()
        .next()
        .unwrap_or_default();
    matches!(
        tag,
        "address"
            | "article"
            | "aside"
            | "blockquote"
            | "br"
            | "dd"
            | "div"
            | "dl"
            | "dt"
            | "figcaption"
            | "footer"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "header"
            | "hr"
            | "li"
            | "main"
            | "nav"
            | "ol"
            | "p"
            | "pre"
            | "section"
            | "table"
            | "tbody"
            | "td"
            | "tfoot"
            | "th"
            | "thead"
            | "tr"
            | "ul"
    )
}

fn decode_html_entities(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '&' {
            output.push(ch);
            continue;
        }
        let mut entity = String::new();
        let mut terminated = false;
        let mut aborted = false;
        while let Some(next) = chars.peek().copied() {
            if next == ';' {
                chars.next();
                terminated = true;
                break;
            }
            if entity.len() >= 16 || next.is_whitespace() || next == '&' {
                output.push('&');
                output.push_str(&entity);
                entity.clear();
                aborted = true;
                break;
            }
            entity.push(next);
            chars.next();
        }
        if aborted {
            continue;
        }
        if !terminated {
            output.push('&');
            output.push_str(&entity);
            continue;
        }
        if entity.is_empty() {
            output.push('&');
            output.push(';');
            continue;
        }
        match decode_html_entity(&entity) {
            Some(decoded) => output.push(decoded),
            None => {
                output.push('&');
                output.push_str(&entity);
                output.push(';');
            }
        }
    }
    output
}

fn decode_html_entity(entity: &str) -> Option<char> {
    match entity {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        "nbsp" => Some(' '),
        _ if entity.starts_with("#x") || entity.starts_with("#X") => {
            u32::from_str_radix(&entity[2..], 16)
                .ok()
                .and_then(char::from_u32)
        }
        _ if entity.starts_with('#') => entity[1..].parse::<u32>().ok().and_then(char::from_u32),
        _ => None,
    }
}

fn normalize_extracted_text(value: &str) -> String {
    value
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn read_file_tool(db: &Database, arguments: &Value) -> Result<Value, String> {
    let requested = required_raw_string(arguments, "path")?;
    let path = resolve_existing_safe_file(db, &requested)?;
    let metadata = fs::metadata(&path).map_err(|e| format!("Cannot read file metadata: {e}"))?;
    if !metadata.is_file() {
        return Err("Path is not a regular file.".to_string());
    }
    if metadata.len() > MAX_FILE_TOOL_BYTES {
        return Err(format!(
            "File is too large for Agent read_file: {} bytes > {} bytes",
            metadata.len(),
            MAX_FILE_TOOL_BYTES
        ));
    }
    let bytes = fs::read(&path).map_err(|e| format!("Cannot read file: {e}"))?;
    if bytes.contains(&0) {
        return Err("File appears to be binary and cannot be read by Agent tools.".to_string());
    }
    let content = String::from_utf8(bytes).map_err(|_| {
        "File is not valid UTF-8 text and cannot be read by Agent tools.".to_string()
    })?;
    Ok(json!({
        "path": path.to_string_lossy().to_string(),
        "bytes": content.len(),
        "content": content,
    }))
}

fn create_pending_tool_action(
    db: &Database,
    input: &AgentToolCallInput,
) -> Result<AgentToolAction, String> {
    let preview = preview_tool_action(db, input.tool_name.trim(), &input.arguments)?;
    let action = AgentToolAction {
        action_id: format!(
            "agent-tool-action-{}",
            Local::now().timestamp_nanos_opt().unwrap_or_default()
        ),
        session_id: input.session_id.clone(),
        agent_id: input.agent_id.clone(),
        tool_name: input.tool_name.trim().to_string(),
        arguments: input.arguments.clone(),
        preview,
        status: "pending".to_string(),
        created_at: String::new(),
        updated_at: String::new(),
    };
    db.save_agent_tool_action(&action)
        .map_err(|e| e.to_string())
}

fn preview_tool_action(db: &Database, tool_name: &str, arguments: &Value) -> Result<Value, String> {
    match tool_name {
        "write_note" => {
            let note_id = required_i64(arguments, "note_id")?;
            let note = find_note(db, note_id)?;
            Ok(json!({
                "kind": "note_edit",
                "note_id": note.id,
                "before": note,
                "after": {
                    "id": note_id,
                    "title": optional_string(arguments, "title").unwrap_or_else(|| note.title.clone()),
                    "content": optional_string(arguments, "content").unwrap_or_else(|| note.content.clone()),
                    "color": optional_string(arguments, "color").unwrap_or_else(|| note.color.clone()),
                }
            }))
        }
        "add_todo_item" => {
            let title = required_string(arguments, "title")?;
            Ok(json!({
                "kind": "todo_add",
                "after": {
                    "title": title,
                    "description": optional_string(arguments, "description").unwrap_or_default(),
                    "priority": optional_i64(arguments, "priority")?.unwrap_or(2).clamp(1, 3),
                    "deadline": optional_string(arguments, "deadline"),
                }
            }))
        }
        "change_todo_item" => {
            let todo_id = required_i64(arguments, "todo_id")?;
            let todo = find_todo(db, todo_id)?;
            Ok(json!({
                "kind": "todo_change",
                "todo_id": todo.id,
                "before": todo,
                "after": {
                    "id": todo_id,
                    "title": optional_string(arguments, "title").unwrap_or_else(|| todo.title.clone()),
                    "description": optional_string(arguments, "description").unwrap_or_else(|| todo.description.clone()),
                    "priority": optional_i64(arguments, "priority")?.map(|value| value.clamp(1, 3)).unwrap_or(todo.priority as i64),
                    "deadline": optional_string(arguments, "deadline").or(todo.deadline.clone()),
                    "completed": arguments.get("completed").and_then(Value::as_bool).unwrap_or(todo.completed),
                }
            }))
        }
        "change_milestone" => {
            let index = required_usize(arguments, "index")?;
            let settings = db.get_pomodoro_settings().map_err(|e| e.to_string())?;
            let Some(milestone) = settings.milestones.get(index).cloned() else {
                return Err(format!("Milestone not found at index {index}"));
            };
            Ok(json!({
                "kind": "milestone_change",
                "index": index,
                "before": MilestoneContext::from_milestone(index, milestone.clone()),
                "after": {
                    "index": index,
                    "name": optional_string(arguments, "name").unwrap_or(milestone.name),
                    "deadline": optional_string(arguments, "deadline").unwrap_or(milestone.deadline),
                    "status": normalize_milestone_status(optional_string(arguments, "status").as_deref().unwrap_or(&milestone.status)),
                }
            }))
        }
        "write_file" => {
            let requested = required_raw_string(arguments, "path")?;
            let content = required_raw_string(arguments, "content")?;
            validate_tool_file_content(&content)?;
            let path = resolve_safe_write_path(db, &requested)?;
            let before = if path.exists() {
                let metadata = fs::metadata(&path)
                    .map_err(|e| format!("Cannot read existing file metadata: {e}"))?;
                if metadata.len() > MAX_FILE_TOOL_BYTES {
                    json!({
                        "exists": true,
                        "bytes": metadata.len(),
                        "content": "[existing file is too large to preview]"
                    })
                } else {
                    match fs::read(&path)
                        .ok()
                        .and_then(|bytes| String::from_utf8(bytes).ok())
                    {
                        Some(existing) if !existing.as_bytes().contains(&0) => json!({
                            "exists": true,
                            "bytes": existing.len(),
                            "content": existing,
                        }),
                        _ => json!({
                            "exists": true,
                            "content": "[existing file is not valid UTF-8 text]"
                        }),
                    }
                }
            } else {
                json!({ "exists": false })
            };
            Ok(json!({
                "kind": "file_write",
                "path": path.to_string_lossy().to_string(),
                "before": before,
                "after": {
                    "bytes": content.len(),
                    "content": content,
                }
            }))
        }
        _ => Err(format!(
            "Tool does not create a pending action: {tool_name}"
        )),
    }
}

fn confirm_pending_tool_action(
    db: &Database,
    input: ConfirmAgentToolActionInput,
) -> Result<ConfirmAgentToolActionResult, String> {
    let action = db
        .get_agent_tool_action(&input.action_id)
        .map_err(|e| e.to_string())?;
    if action.status != "pending" {
        return Err(format!("Tool action is not pending: {}", action.action_id));
    }
    if !input.accepted {
        let action = db
            .set_agent_tool_action_status(&action.action_id, "rejected")
            .map_err(|e| e.to_string())?;
        let result = json!({ "rejected": true, "preview": action.preview });
        insert_confirmation_audit(db, &action, "rejected", &result)?;
        return Ok(ConfirmAgentToolActionResult {
            action_id: action.action_id,
            accepted: false,
            tool_name: action.tool_name,
            status: "rejected".to_string(),
            result,
        });
    }

    let result = execute_confirmed_action(db, &action)?;
    let action = db
        .set_agent_tool_action_status(&action.action_id, "completed")
        .map_err(|e| e.to_string())?;
    insert_confirmation_audit(db, &action, "completed", &result)?;
    Ok(ConfirmAgentToolActionResult {
        action_id: action.action_id,
        accepted: true,
        tool_name: action.tool_name,
        status: "completed".to_string(),
        result,
    })
}

fn execute_confirmed_action(db: &Database, action: &AgentToolAction) -> Result<Value, String> {
    if let Some(tool_id) = external_cli_tool_id_from_function(&action.tool_name) {
        return execute_confirmed_external_cli_action(db, action, &tool_id);
    }
    match action.tool_name.as_str() {
        "write_note" => {
            let note_id = required_i64(&action.arguments, "note_id")?;
            let title = optional_string(&action.arguments, "title");
            let content = optional_string(&action.arguments, "content");
            let color = optional_string(&action.arguments, "color");
            let note = db
                .update_note(
                    note_id,
                    title.as_deref(),
                    content.as_deref(),
                    color.as_deref(),
                )
                .map_err(|e| e.to_string())?;
            Ok(json!({ "note": note }))
        }
        "add_todo_item" => {
            let title = required_string(&action.arguments, "title")?;
            let description = optional_string(&action.arguments, "description").unwrap_or_default();
            let priority = optional_i64(&action.arguments, "priority")?
                .unwrap_or(2)
                .clamp(1, 3) as i32;
            let deadline = optional_string(&action.arguments, "deadline");
            let todo = db
                .add_todo(
                    &title,
                    &description,
                    priority,
                    deadline.as_deref(),
                    None,
                    None,
                    None,
                    None,
                )
                .map_err(|e| e.to_string())?;
            Ok(json!({ "todo": todo }))
        }
        "change_todo_item" => {
            let todo_id = required_i64(&action.arguments, "todo_id")?;
            let title = optional_string(&action.arguments, "title");
            let description = optional_string(&action.arguments, "description");
            let priority =
                optional_i64(&action.arguments, "priority")?.map(|value| value.clamp(1, 3) as i32);
            let deadline = optional_string(&action.arguments, "deadline");
            let mut todo = db
                .update_todo(
                    todo_id,
                    title.as_deref(),
                    description.as_deref(),
                    priority,
                    deadline.as_deref(),
                    false,
                    None,
                    None,
                    None,
                    None,
                )
                .map_err(|e| e.to_string())?;
            if let Some(completed) = action.arguments.get("completed").and_then(Value::as_bool) {
                if todo.completed != completed {
                    todo = db.toggle_todo(todo_id).map_err(|e| e.to_string())?;
                }
            }
            Ok(json!({ "todo": todo }))
        }
        "change_milestone" => {
            let index = required_usize(&action.arguments, "index")?;
            let mut settings = db.get_pomodoro_settings().map_err(|e| e.to_string())?;
            let Some(milestone) = settings.milestones.get_mut(index) else {
                return Err(format!("Milestone not found at index {index}"));
            };
            if let Some(name) = optional_string(&action.arguments, "name") {
                milestone.name = name;
            }
            if let Some(deadline) = optional_string(&action.arguments, "deadline") {
                milestone.deadline = deadline;
            }
            if let Some(status) = optional_string(&action.arguments, "status") {
                milestone.status = normalize_milestone_status(&status);
            }
            db.save_pomodoro_settings(&settings)
                .map_err(|e| e.to_string())?;
            let milestone = settings
                .milestones
                .get(index)
                .cloned()
                .ok_or_else(|| format!("Milestone not found at index {index}"))?;
            Ok(json!({ "milestone": MilestoneContext::from_milestone(index, milestone) }))
        }
        "write_file" => {
            let requested = required_raw_string(&action.arguments, "path")?;
            let content = required_raw_string(&action.arguments, "content")?;
            validate_tool_file_content(&content)?;
            let path = resolve_safe_write_path(db, &requested)?;
            let mut file =
                fs::File::create(&path).map_err(|e| format!("Cannot write file: {e}"))?;
            file.write_all(content.as_bytes())
                .map_err(|e| format!("Cannot write file: {e}"))?;
            Ok(json!({
                "path": path.to_string_lossy().to_string(),
                "bytes": content.len(),
                "written": true,
            }))
        }
        _ => Err(format!(
            "Unsupported pending tool action: {}",
            action.tool_name
        )),
    }
}

fn execute_confirmed_external_cli_action(
    db: &Database,
    action: &AgentToolAction,
    tool_id: &str,
) -> Result<Value, String> {
    let input = AgentExternalCliCallInput {
        session_id: action.session_id.clone(),
        agent_id: action.agent_id.clone(),
        tool_id: tool_id.to_string(),
        arguments: action.arguments.clone(),
    };
    let tool = annotate_cli_availability(
        db.get_agent_external_cli_tool(tool_id)
            .map_err(|e| e.to_string())?,
    );
    if !tool.enabled {
        return Err(format!("External CLI tool is disabled: {tool_id}"));
    }
    if !tool.available {
        let result =
            external_cli_error_result(tool_id, "unavailable", "accepted", tool.availability_error);
        audit_external_cli_call(db, &result, &input)?;
        return Ok(json!(result));
    }
    let argv = match cli_argv_from_arguments(&tool, &input.arguments) {
        Ok(argv) => argv,
        Err(error) => {
            let result = external_cli_error_result(tool_id, "validation_error", "accepted", error);
            audit_external_cli_call(db, &result, &input)?;
            return Ok(json!(result));
        }
    };
    let mut result = run_external_cli_command(&tool, &argv)?;
    result.confirmation_status = "accepted".to_string();
    audit_external_cli_call(db, &result, &input)?;
    Ok(json!(result))
}

fn insert_confirmation_audit(
    db: &Database,
    action: &AgentToolAction,
    status: &str,
    result: &Value,
) -> Result<(), String> {
    let audit_id = format!(
        "agent-tool-audit-{}",
        Local::now().timestamp_nanos_opt().unwrap_or_default()
    );
    db.insert_agent_builtin_tool_audit(
        &audit_id,
        action.session_id.as_deref(),
        action.agent_id.as_deref(),
        &action.tool_name,
        Some(&action.action_id),
        &action.arguments,
        status,
        result,
    )
    .map_err(|e| e.to_string())
}

fn install_external_cli_presets(db: &Database) -> Result<Vec<AgentExternalCliTool>, String> {
    for preset in external_cli_presets() {
        if validate_cli_executable(&preset.executable).is_ok() {
            let input = normalize_external_cli_tool(preset)?;
            validate_external_cli_registration(&input)?;
            db.save_agent_external_cli_tool(&input)
                .map_err(|e| e.to_string())?;
        }
    }
    let tools = db
        .list_agent_external_cli_tools()
        .map_err(|e| e.to_string())?;
    Ok(tools.into_iter().map(annotate_cli_availability).collect())
}

fn external_cli_presets() -> Vec<SaveAgentExternalCliTool> {
    vec![
        external_cli_help_preset("helper_help", "DevHelper Help", "helper"),
        external_cli_help_preset("skill_help", "Skill CLI Help", "skill"),
    ]
}

fn external_cli_help_preset(
    tool_id: &str,
    display_name: &str,
    executable: &str,
) -> SaveAgentExternalCliTool {
    SaveAgentExternalCliTool {
        tool_id: Some(tool_id.to_string()),
        display_name: display_name.to_string(),
        executable: executable.to_string(),
        allowed_subcommands: vec!["--help".to_string()],
        argument_schema: json!({
            "type": "object",
            "required": ["subcommand"],
            "properties": {
                "subcommand": {
                    "type": "string",
                    "description": "Only --help is allowed for this read-only preset."
                }
            }
        }),
        working_directory: String::new(),
        environment_allowlist: Vec::new(),
        timeout_ms: 10_000,
        output_limit: 12_000,
        safety_class: "read".to_string(),
        enabled: true,
    }
}

fn normalize_external_cli_tool(
    input: SaveAgentExternalCliTool,
) -> Result<SaveAgentExternalCliTool, String> {
    let display_name = input.display_name.trim().to_string();
    let executable = input.executable.trim().to_string();
    let generated_id = slugify_tool_id(if display_name.is_empty() {
        &executable
    } else {
        &display_name
    });
    let tool_id = input
        .tool_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or(generated_id);
    let working_directory = normalize_optional_directory(&input.working_directory)?;
    let allowed_subcommands = input
        .allowed_subcommands
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    let environment_allowlist = input
        .environment_allowlist
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    Ok(SaveAgentExternalCliTool {
        tool_id: Some(tool_id),
        display_name,
        executable,
        allowed_subcommands,
        argument_schema: normalize_cli_argument_schema(input.argument_schema),
        working_directory,
        environment_allowlist,
        timeout_ms: input.timeout_ms,
        output_limit: input.output_limit,
        safety_class: input.safety_class.trim().to_lowercase(),
        enabled: input.enabled,
    })
}

fn validate_external_cli_registration(input: &SaveAgentExternalCliTool) -> Result<(), String> {
    let tool_id = input.tool_id.as_deref().unwrap_or_default();
    if !valid_plugin_id(tool_id) {
        return Err("External CLI tool_id must use lowercase letters, digits, and underscores, up to 32 characters.".to_string());
    }
    if input.display_name.trim().is_empty() {
        return Err("External CLI display_name is required.".to_string());
    }
    if input.executable.trim().is_empty() {
        return Err("External CLI executable is required.".to_string());
    }
    validate_cli_executable(&input.executable)?;
    validate_cli_argument_schema(&input.argument_schema)?;
    validate_cli_tokens("allowed_subcommands", &input.allowed_subcommands)?;
    validate_cli_environment_names(&input.environment_allowlist)?;
    if !input.working_directory.trim().is_empty() {
        let metadata = fs::metadata(input.working_directory.trim())
            .map_err(|e| format!("Cannot read working directory metadata: {e}"))?;
        if !metadata.is_dir() {
            return Err("External CLI working_directory must be a directory.".to_string());
        }
    }
    if !(MIN_CLI_TIMEOUT_MS..=MAX_CLI_TIMEOUT_MS).contains(&input.timeout_ms) {
        return Err(format!(
            "External CLI timeout_ms must be between {MIN_CLI_TIMEOUT_MS} and {MAX_CLI_TIMEOUT_MS}."
        ));
    }
    if !(MIN_CLI_OUTPUT_LIMIT..=MAX_CLI_OUTPUT_LIMIT).contains(&input.output_limit) {
        return Err(format!(
            "External CLI output_limit must be between {MIN_CLI_OUTPUT_LIMIT} and {MAX_CLI_OUTPUT_LIMIT}."
        ));
    }
    if !matches!(
        input.safety_class.as_str(),
        "read" | "write" | "networked" | "sensitive" | "destructive"
    ) {
        return Err(
            "External CLI safety_class must be read, write, networked, sensitive, or destructive."
                .to_string(),
        );
    }
    Ok(())
}

fn annotate_cli_availability(mut tool: AgentExternalCliTool) -> AgentExternalCliTool {
    match validate_cli_executable(&tool.executable) {
        Ok(_) => {
            tool.available = true;
            tool.availability_error.clear();
        }
        Err(error) => {
            tool.available = false;
            tool.availability_error = error;
        }
    }
    tool
}

fn validate_cli_executable(value: &str) -> Result<PathBuf, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("External CLI executable is required.".to_string());
    }
    let candidate = PathBuf::from(trimmed);
    if candidate.components().count() > 1 || candidate.is_absolute() {
        return validate_executable_path(candidate);
    }
    let paths = env::var_os("PATH").unwrap_or_default();
    for root in env::split_paths(&paths) {
        let path = root.join(trimmed);
        if path.exists() {
            return validate_executable_path(path);
        }
    }
    Err(format!(
        "External CLI executable not found on PATH: {trimmed}"
    ))
}

fn validate_executable_path(path: PathBuf) -> Result<PathBuf, String> {
    let canonical = fs::canonicalize(&path).map_err(|e| {
        format!(
            "External CLI executable is unavailable: {}: {e}",
            path.display()
        )
    })?;
    let metadata = fs::metadata(&canonical).map_err(|e| {
        format!(
            "Cannot read external CLI executable metadata {}: {e}",
            canonical.display()
        )
    })?;
    if !metadata.is_file() {
        return Err(format!(
            "External CLI executable is not a file: {}",
            canonical.display()
        ));
    }
    #[cfg(unix)]
    {
        if metadata.permissions().mode() & 0o111 == 0 {
            return Err(format!(
                "External CLI executable is not executable: {}",
                canonical.display()
            ));
        }
    }
    Ok(canonical)
}

fn normalize_cli_argument_schema(value: Value) -> Value {
    if value.is_null()
        || value
            .as_object()
            .map(|object| object.is_empty())
            .unwrap_or(false)
    {
        json!({ "type": "object", "properties": {} })
    } else {
        value
    }
}

fn validate_cli_argument_schema(value: &Value) -> Result<(), String> {
    let Some(object) = value.as_object() else {
        return Err("External CLI argument_schema must be a JSON object.".to_string());
    };
    if let Some(schema_type) = object.get("type").and_then(Value::as_str) {
        if schema_type != "object" {
            return Err("External CLI argument_schema.type must be object.".to_string());
        }
    }
    if let Some(properties) = object.get("properties") {
        if !properties.is_object() {
            return Err("External CLI argument_schema.properties must be an object.".to_string());
        }
    }
    if let Some(required) = object.get("required") {
        let Some(items) = required.as_array() else {
            return Err("External CLI argument_schema.required must be an array.".to_string());
        };
        if items
            .iter()
            .any(|item| item.as_str().unwrap_or("").is_empty())
        {
            return Err(
                "External CLI argument_schema.required must contain field names.".to_string(),
            );
        }
    }
    Ok(())
}

fn validate_cli_tokens(field: &str, values: &[String]) -> Result<(), String> {
    for value in values {
        if value.chars().any(char::is_whitespace)
            || value.contains(';')
            || value.contains('|')
            || value.contains('&')
            || value.contains('`')
            || value.contains('$')
        {
            return Err(format!(
                "External CLI {field} entries must be single argv tokens without shell metacharacters."
            ));
        }
    }
    Ok(())
}

fn validate_cli_environment_names(values: &[String]) -> Result<(), String> {
    for value in values {
        let mut chars = value.chars();
        let Some(first) = chars.next() else {
            continue;
        };
        if !(first.is_ascii_alphabetic() || first == '_')
            || !chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        {
            return Err(format!("Invalid environment allowlist name: {value}"));
        }
    }
    Ok(())
}

fn normalize_optional_directory(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }
    let canonical = fs::canonicalize(trimmed)
        .map_err(|e| format!("Working directory does not exist: {trimmed}: {e}"))?;
    let metadata = fs::metadata(&canonical).map_err(|e| {
        format!(
            "Cannot read working directory metadata {}: {e}",
            canonical.display()
        )
    })?;
    if !metadata.is_dir() {
        return Err(format!(
            "Working directory is not a directory: {}",
            canonical.display()
        ));
    }
    Ok(canonical.to_string_lossy().to_string())
}

fn slugify_tool_id(value: &str) -> String {
    let mut result = String::new();
    let mut last_was_underscore = false;
    for ch in value.chars() {
        let next = if ch.is_ascii_alphanumeric() {
            Some(ch.to_ascii_lowercase())
        } else if ch == '_' || ch == '-' || ch.is_whitespace() {
            Some('_')
        } else {
            None
        };
        let Some(next) = next else {
            continue;
        };
        if next == '_' {
            if result.is_empty() || last_was_underscore {
                continue;
            }
            last_was_underscore = true;
        } else {
            last_was_underscore = false;
        }
        result.push(next);
        if result.len() >= 32 {
            break;
        }
    }
    let result = result.trim_matches('_').to_string();
    if result.is_empty() {
        "external_cli".to_string()
    } else {
        result
    }
}

fn execute_external_cli_tool(
    db: &Database,
    input: AgentExternalCliCallInput,
) -> Result<AgentExternalCliCallResult, String> {
    let tool_id = input.tool_id.trim();
    let tool = annotate_cli_availability(
        db.get_agent_external_cli_tool(tool_id)
            .map_err(|e| e.to_string())?,
    );
    if !tool.enabled {
        return Err(format!("External CLI tool is disabled: {tool_id}"));
    }
    if !tool.available {
        let result = external_cli_error_result(
            tool_id,
            "unavailable",
            "not_required",
            tool.availability_error,
        );
        audit_external_cli_call(db, &result, &input)?;
        return Ok(result);
    }
    let argv = match cli_argv_from_arguments(&tool, &input.arguments) {
        Ok(argv) => argv,
        Err(error) => {
            let result =
                external_cli_error_result(tool_id, "validation_error", "not_required", error);
            audit_external_cli_call(db, &result, &input)?;
            return Ok(result);
        }
    };
    if tool.safety_class != "read" {
        let action = create_pending_external_cli_action(db, &input, &tool, &argv)?;
        let result = external_cli_error_result(
            tool_id,
            "requires_confirmation",
            "required",
            format!(
                "External CLI tool requires confirmation before execution. Proposed action: {}",
                action.action_id
            ),
        );
        audit_external_cli_call(db, &result, &input)?;
        return Ok(result);
    }

    let result = run_external_cli_command(&tool, &argv)?;
    audit_external_cli_call(db, &result, &input)?;
    Ok(result)
}

fn create_pending_external_cli_action(
    db: &Database,
    input: &AgentExternalCliCallInput,
    tool: &AgentExternalCliTool,
    argv: &[String],
) -> Result<AgentToolAction, String> {
    let preview = json!({
        "kind": "external_cli",
        "tool_id": tool.tool_id,
        "display_name": tool.display_name,
        "executable": tool.executable,
        "argv": argv,
        "working_directory": tool.working_directory,
        "safety_class": tool.safety_class,
        "timeout_ms": tool.timeout_ms,
        "output_limit": tool.output_limit,
    });
    let action = AgentToolAction {
        action_id: format!(
            "agent-tool-action-{}",
            Local::now().timestamp_nanos_opt().unwrap_or_default()
        ),
        session_id: input.session_id.clone(),
        agent_id: input.agent_id.clone(),
        tool_name: external_cli_function_name(&tool.tool_id),
        arguments: input.arguments.clone(),
        preview,
        status: "pending".to_string(),
        created_at: String::new(),
        updated_at: String::new(),
    };
    db.save_agent_tool_action(&action)
        .map_err(|e| e.to_string())
}

fn audit_external_cli_call(
    db: &Database,
    result: &AgentExternalCliCallResult,
    input: &AgentExternalCliCallInput,
) -> Result<(), String> {
    let masked_arguments = mask_sensitive_json(&input.arguments);
    db.insert_agent_external_cli_audit(
        result,
        input.session_id.as_deref(),
        input.agent_id.as_deref(),
        &masked_arguments,
    )
    .map_err(|e| e.to_string())
}

fn cli_argv_from_arguments(
    tool: &AgentExternalCliTool,
    arguments: &Value,
) -> Result<Vec<String>, String> {
    validate_cli_call_arguments_schema(&tool.argument_schema, arguments)?;
    let object = arguments
        .as_object()
        .ok_or_else(|| "External CLI arguments must be a JSON object.".to_string())?;
    let mut argv = Vec::new();
    if let Some(subcommand) = object.get("subcommand").and_then(Value::as_str) {
        let subcommand = subcommand.trim().to_string();
        validate_cli_tokens("subcommand", std::slice::from_ref(&subcommand))?;
        if !tool.allowed_subcommands.is_empty()
            && !tool
                .allowed_subcommands
                .iter()
                .any(|allowed| allowed == &subcommand)
        {
            return Err(format!(
                "External CLI subcommand is not allowed: {subcommand}"
            ));
        }
        argv.push(subcommand);
    }
    if let Some(args) = object.get("args") {
        let Some(items) = args.as_array() else {
            return Err("External CLI args must be an array of strings.".to_string());
        };
        let mut tokens = Vec::with_capacity(items.len());
        for item in items {
            let Some(token) = item.as_str() else {
                return Err("External CLI args must be an array of strings.".to_string());
            };
            tokens.push(token.trim().to_string());
        }
        validate_cli_tokens("args", &tokens)?;
        argv.extend(tokens);
    }
    Ok(argv)
}

fn validate_cli_call_arguments_schema(schema: &Value, arguments: &Value) -> Result<(), String> {
    let Some(values) = arguments.as_object() else {
        return Err("External CLI arguments must be a JSON object.".to_string());
    };
    let properties = schema
        .get("properties")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    for required in schema
        .get("required")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
    {
        if !values.contains_key(required) {
            return Err(format!("External CLI argument is required: {required}"));
        }
    }
    for (name, value) in values {
        let Some(property) = properties.get(name) else {
            return Err(format!("External CLI argument is not allowed: {name}"));
        };
        validate_cli_value_type(name, value, property)?;
    }
    Ok(())
}

fn validate_cli_value_type(name: &str, value: &Value, schema: &Value) -> Result<(), String> {
    let Some(schema_type) = schema.get("type").and_then(Value::as_str) else {
        return Ok(());
    };
    let valid = match schema_type {
        "string" => value.is_string(),
        "boolean" => value.is_boolean(),
        "integer" => value.as_i64().is_some(),
        "number" => value.as_f64().is_some(),
        "array" => value.is_array(),
        "object" => value.is_object(),
        _ => true,
    };
    if !valid {
        return Err(format!(
            "External CLI argument `{name}` does not match schema type `{schema_type}`."
        ));
    }
    if schema_type == "array" {
        if let Some(item_type) = schema.pointer("/items/type").and_then(Value::as_str) {
            let Some(items) = value.as_array() else {
                return Err(format!("External CLI argument `{name}` must be an array."));
            };
            for item in items {
                validate_cli_value_type(name, item, &json!({ "type": item_type }))?;
            }
        }
    }
    Ok(())
}

fn run_external_cli_command(
    tool: &AgentExternalCliTool,
    argv: &[String],
) -> Result<AgentExternalCliCallResult, String> {
    let executable = validate_cli_executable(&tool.executable)?;
    let start = Instant::now();
    let mut command = Command::new(executable);
    command.args(argv);
    command.stdin(Stdio::null());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    command.env_clear();
    for name in &tool.environment_allowlist {
        if let Ok(value) = env::var(name) {
            command.env(name, value);
        }
    }
    if !tool.working_directory.trim().is_empty() {
        command.current_dir(tool.working_directory.trim());
    }
    let mut child = command
        .spawn()
        .map_err(|e| format!("Failed to spawn external CLI tool: {e}"))?;
    let timeout = Duration::from_millis(tool.timeout_ms as u64);
    let mut timed_out = false;
    loop {
        if child
            .try_wait()
            .map_err(|e| format!("Failed to poll external CLI tool: {e}"))?
            .is_some()
        {
            break;
        }
        if start.elapsed() >= timeout {
            timed_out = true;
            let _ = child.kill();
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    let output = child
        .wait_with_output()
        .map_err(|e| format!("Failed to collect external CLI output: {e}"))?;
    let duration_ms = start.elapsed().as_millis().min(i64::MAX as u128) as i64;
    let (stdout, stdout_truncated) = bounded_output(&output.stdout, tool.output_limit as usize);
    let (stderr, stderr_truncated) = bounded_output(&output.stderr, tool.output_limit as usize);
    let status = if timed_out {
        "timeout"
    } else if output.status.success() {
        "completed"
    } else {
        "failed"
    };
    Ok(AgentExternalCliCallResult {
        audit_id: format!(
            "agent-cli-audit-{}",
            Local::now().timestamp_nanos_opt().unwrap_or_default()
        ),
        tool_id: tool.tool_id.clone(),
        status: status.to_string(),
        confirmation_status: "not_required".to_string(),
        exit_code: output.status.code(),
        stdout,
        stderr,
        duration_ms,
        timed_out,
        truncated: stdout_truncated || stderr_truncated,
        message: if timed_out {
            "External CLI tool timed out.".to_string()
        } else if output.status.success() {
            "External CLI tool completed.".to_string()
        } else {
            "External CLI tool exited with a non-zero status.".to_string()
        },
    })
}

fn bounded_output(bytes: &[u8], limit: usize) -> (String, bool) {
    let truncated = bytes.len() > limit;
    let bounded = if truncated { &bytes[..limit] } else { bytes };
    (String::from_utf8_lossy(bounded).to_string(), truncated)
}

fn external_cli_error_result(
    tool_id: &str,
    status: &str,
    confirmation_status: &str,
    message: String,
) -> AgentExternalCliCallResult {
    AgentExternalCliCallResult {
        audit_id: format!(
            "agent-cli-audit-{}",
            Local::now().timestamp_nanos_opt().unwrap_or_default()
        ),
        tool_id: tool_id.to_string(),
        status: status.to_string(),
        confirmation_status: confirmation_status.to_string(),
        exit_code: None,
        stdout: String::new(),
        stderr: message.clone(),
        duration_ms: 0,
        timed_out: false,
        truncated: false,
        message,
    }
}

fn mask_sensitive_json(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, value)| {
                    if is_sensitive_key(key) {
                        (key.clone(), Value::String("***".to_string()))
                    } else {
                        (key.clone(), mask_sensitive_json(value))
                    }
                })
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(mask_sensitive_json).collect()),
        _ => value.clone(),
    }
}

fn is_sensitive_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    [
        "secret",
        "password",
        "passwd",
        "token",
        "api_key",
        "apikey",
        "credential",
    ]
    .iter()
    .any(|needle| key.contains(needle))
}

fn normalize_safe_roots(roots: Vec<String>) -> Result<Vec<String>, String> {
    let mut normalized = Vec::new();
    let mut seen = HashSet::new();
    for root in roots {
        let trimmed = root.trim();
        if trimmed.is_empty() {
            continue;
        }
        let canonical = fs::canonicalize(trimmed)
            .map_err(|e| format!("Safe root does not exist: {trimmed}: {e}"))?;
        let metadata = fs::metadata(&canonical).map_err(|e| {
            format!(
                "Cannot read safe root metadata {}: {e}",
                canonical.display()
            )
        })?;
        if !metadata.is_dir() {
            return Err(format!(
                "Safe root is not a directory: {}",
                canonical.display()
            ));
        }
        let value = canonical.to_string_lossy().to_string();
        if seen.insert(value.clone()) {
            normalized.push(value);
        }
    }
    Ok(normalized)
}

fn configured_safe_roots(db: &Database) -> Result<Vec<PathBuf>, String> {
    let roots = db
        .get_agent_safe_file_root_settings()
        .map_err(|e| e.to_string())?
        .safe_file_roots;
    if roots.is_empty() {
        return Err("No Agent safe file roots are configured.".to_string());
    }
    roots
        .into_iter()
        .map(|root| {
            fs::canonicalize(&root).map_err(|e| format!("Safe root is unavailable: {root}: {e}"))
        })
        .collect()
}

fn resolve_existing_safe_file(db: &Database, requested: &str) -> Result<PathBuf, String> {
    let path = candidate_safe_path(db, requested)?;
    let canonical = fs::canonicalize(&path).map_err(|e| {
        format!(
            "File does not exist or cannot be resolved: {}: {e}",
            path.display()
        )
    })?;
    ensure_path_inside_safe_roots(db, &canonical)?;
    Ok(canonical)
}

fn resolve_safe_write_path(db: &Database, requested: &str) -> Result<PathBuf, String> {
    let path = candidate_safe_path(db, requested)?;
    let parent = path
        .parent()
        .ok_or_else(|| "File path must include a parent directory.".to_string())?;
    let canonical_parent = fs::canonicalize(parent)
        .map_err(|e| format!("File parent directory does not exist or cannot be resolved: {e}"))?;
    ensure_path_inside_safe_roots(db, &canonical_parent)?;
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "File path must include a valid UTF-8 file name.".to_string())?;
    if file_name.is_empty() || file_name == "." || file_name == ".." {
        return Err("File path has an unsafe file name.".to_string());
    }
    Ok(canonical_parent.join(file_name))
}

fn candidate_safe_path(db: &Database, requested: &str) -> Result<PathBuf, String> {
    let trimmed = requested.trim();
    if trimmed.is_empty() {
        return Err("File path is required.".to_string());
    }
    let requested_path = PathBuf::from(trimmed);
    if requested_path.is_absolute() {
        return Ok(requested_path);
    }
    let roots = configured_safe_roots(db)?;
    let Some(root) = roots.first() else {
        return Err("No Agent safe file roots are configured.".to_string());
    };
    Ok(root.join(requested_path))
}

fn ensure_path_inside_safe_roots(db: &Database, canonical_path: &Path) -> Result<(), String> {
    let roots = configured_safe_roots(db)?;
    if roots.iter().any(|root| canonical_path.starts_with(root)) {
        return Ok(());
    }
    Err(format!(
        "Path is outside configured Agent safe roots: {}",
        canonical_path.display()
    ))
}

fn validate_tool_file_content(content: &str) -> Result<(), String> {
    if content.as_bytes().contains(&0) {
        return Err(
            "File content appears to be binary and cannot be written by Agent tools.".to_string(),
        );
    }
    if content.len() as u64 > MAX_FILE_TOOL_BYTES {
        return Err(format!(
            "File content is too large for Agent write_file: {} bytes > {} bytes",
            content.len(),
            MAX_FILE_TOOL_BYTES
        ));
    }
    Ok(())
}

fn find_note(db: &Database, note_id: i64) -> Result<crate::models::note::StickyNote, String> {
    db.list_notes()
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|note| note.id == note_id)
        .ok_or_else(|| format!("Sticky note not found: {note_id}"))
}

fn find_todo(db: &Database, todo_id: i64) -> Result<crate::models::todo::Todo, String> {
    db.list_todos()
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|todo| todo.id == todo_id)
        .ok_or_else(|| format!("Todo not found: {todo_id}"))
}

fn required_i64(arguments: &Value, field: &str) -> Result<i64, String> {
    optional_i64(arguments, field)?
        .ok_or_else(|| format!("Missing required integer argument: {field}"))
}

fn required_usize(arguments: &Value, field: &str) -> Result<usize, String> {
    let value = required_i64(arguments, field)?;
    if value < 0 {
        return Err(format!("{field} must be zero or greater"));
    }
    Ok(value as usize)
}

fn optional_i64(arguments: &Value, field: &str) -> Result<Option<i64>, String> {
    let Some(value) = arguments.get(field) else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    value
        .as_i64()
        .ok_or_else(|| format!("{field} must be an integer"))
        .map(Some)
}

fn required_string(arguments: &Value, field: &str) -> Result<String, String> {
    optional_string(arguments, field)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("Missing required string argument: {field}"))
}

fn required_raw_string(arguments: &Value, field: &str) -> Result<String, String> {
    arguments
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| format!("Missing required string argument: {field}"))
}

fn optional_string(arguments: &Value, field: &str) -> Option<String> {
    arguments
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn normalize_milestone_status(value: &str) -> String {
    match value {
        "completed" => "completed".to_string(),
        "cancelled" => "cancelled".to_string(),
        _ => "active".to_string(),
    }
}

fn rebuild_rag_for_plugin(db: &Database, plugin: &AgentPlugin) -> Result<(), String> {
    if !plugin.rag_enabled {
        db.delete_agent_rag_chunks(&plugin.plugin_id)
            .map_err(|e| e.to_string())?;
        return Ok(());
    }
    if !plugin.has_rag_knowledge {
        db.delete_agent_rag_chunks(&plugin.plugin_id)
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    let root = PathBuf::from(&plugin.path);
    let config = read_config(&root)?;
    let content = read_rag_knowledge(&root)?;
    let source_hash = stable_hash_hex(content.as_bytes());
    let chunks = chunk_rag_knowledge(&content)
        .into_iter()
        .enumerate()
        .map(|(index, chunk_text)| AgentRagChunk {
            chunk_id: format!("{}:{}:{index:04}", plugin.plugin_id, source_hash),
            plugin_id: plugin.plugin_id.clone(),
            plugin_version: plugin.plugin_version.clone(),
            source_hash: source_hash.clone(),
            embedding_model: "pending".to_string(),
            embedding_dim: config.embedding_dim,
            chunk_text,
            created_at: String::new(),
        })
        .collect::<Vec<_>>();
    db.replace_agent_rag_chunks(&plugin.plugin_id, &chunks)
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn build_rag_status(db: &Database, plugin: &AgentPlugin) -> Result<AgentRagStatus, String> {
    let chunks = db
        .list_agent_rag_chunks(&plugin.plugin_id)
        .map_err(|e| e.to_string())?;
    let root = PathBuf::from(&plugin.path);
    let current_source_hash = read_rag_knowledge(&root)
        .ok()
        .map(|content| stable_hash_hex(content.as_bytes()));
    let source_hash = chunks.first().map(|chunk| chunk.source_hash.clone());
    let config = read_config(&root).ok();
    let mut stale_reasons = Vec::new();
    if !chunks.is_empty() && source_hash != current_source_hash {
        stale_reasons.push("knowledge source hash changed".to_string());
    }
    if chunks
        .iter()
        .any(|chunk| chunk.plugin_version != plugin.plugin_version)
    {
        stale_reasons.push("plugin version changed".to_string());
    }
    if let Some(config) = config {
        if chunks
            .iter()
            .any(|chunk| chunk.embedding_dim != config.embedding_dim)
        {
            stale_reasons.push("embedding dimension changed".to_string());
        }
    }
    let stale = !stale_reasons.is_empty();
    Ok(AgentRagStatus {
        plugin_id: plugin.plugin_id.clone(),
        rag_enabled: plugin.rag_enabled,
        has_rag_knowledge: plugin.has_rag_knowledge,
        indexed_chunks: chunks.len(),
        source_hash,
        current_source_hash,
        stale,
        stale_reasons,
        vector_search_available: sqlite_vec_available(),
        message: rag_status_message(plugin, chunks.len(), stale),
    })
}

pub fn scan_agents_on_startup(app: &AppHandle, db: &Database) -> Result<(), String> {
    scan_and_persist_agents(app, db)?;
    let _ = migrate_secretary_to_agents(db);
    Ok(())
}

fn find_agent_plugin(db: &Database, plugin_id: &str) -> Result<AgentPlugin, String> {
    db.list_agent_plugins()
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|plugin| plugin.plugin_id == plugin_id)
        .ok_or_else(|| format!("Agent plugin not found: {plugin_id}"))
}

fn migrate_secretary_to_agents(db: &Database) -> Result<AgentMigrationStatus, String> {
    let current = db
        .get_agent_migration_status(SECRETARY_MIGRATION_ID)
        .map_err(|e| e.to_string())?;
    if current.status == "completed" {
        return Ok(current);
    }

    let result = migrate_secretary_to_agents_inner(db);
    match result {
        Ok(details) => db
            .save_agent_migration_status(SECRETARY_MIGRATION_ID, "completed", &details.to_string())
            .map_err(|e| e.to_string()),
        Err(error) => {
            let details = json!({ "error": error });
            db.save_agent_migration_status(SECRETARY_MIGRATION_ID, "failed", &details.to_string())
                .map_err(|e| e.to_string())
        }
    }
}

fn migrate_secretary_to_agents_inner(db: &Database) -> Result<Value, String> {
    let settings = db.get_secretary_settings().map_err(|e| e.to_string())?;
    let personas = db.list_secretary_personas().map_err(|e| e.to_string())?;
    let profiles = db.list_secretary_profiles().map_err(|e| e.to_string())?;
    let memories = db.list_secretary_memories().map_err(|e| e.to_string())?;
    let reminders = db.list_secretary_reminders().map_err(|e| e.to_string())?;
    let conversations = db
        .list_secretary_conversations()
        .map_err(|e| e.to_string())?;

    let active_profile = settings
        .active_profile_id
        .and_then(|id| profiles.iter().find(|profile| profile.id == id))
        .or_else(|| profiles.first());
    let active_persona = settings
        .active_persona_id
        .and_then(|id| personas.iter().find(|persona| persona.id == id))
        .or_else(|| {
            active_profile
                .and_then(|profile| profile.persona_id)
                .and_then(|id| personas.iter().find(|persona| persona.id == id))
        })
        .or_else(|| personas.first());

    let existing_agent_memories = db
        .list_agent_memories(Some("secretary"))
        .map_err(|e| e.to_string())?;
    let mut migrated_profile_memory = false;
    if existing_agent_memories
        .iter()
        .all(|memory| memory.memory_id != "secretary-migrated-profile")
    {
        if let Some(memory) = migrated_secretary_profile_memory(active_persona, active_profile) {
            db.save_agent_memory(&memory).map_err(|e| e.to_string())?;
            migrated_profile_memory = true;
        }
    }

    let mut migrated_memories = 0usize;
    for memory in &memories {
        if memory.content.trim().is_empty() {
            continue;
        }
        let scope = if memory.scope == "global" {
            "global".to_string()
        } else {
            "agent".to_string()
        };
        db.save_agent_memory(&SaveAgentMemory {
            memory_id: Some(format!("secretary-memory-{}", memory.id)),
            content: memory.content.clone(),
            scope,
            agent_id: Some("secretary".to_string()),
            status: Some(memory.status.clone()),
            pinned: Some(memory.pinned),
            source_session_id: memory
                .source_conversation_id
                .map(|id| format!("secretary-conversation-{id}")),
            source_agent_id: Some("secretary".to_string()),
            source_message_id: None,
        })
        .map_err(|e| e.to_string())?;
        migrated_memories += 1;
    }

    let mut migrated_conversations = 0usize;
    let mut migrated_messages = 0usize;
    for conversation in &conversations {
        let session_id = format!("secretary-conversation-{}", conversation.id);
        let session = AgentSession {
            session_id: session_id.clone(),
            session_type: 1,
            agent_ids: vec!["secretary".to_string()],
            session_title: if conversation.title.trim().is_empty() {
                format!("Migrated Secretary conversation {}", conversation.id)
            } else {
                conversation.title.clone()
            },
            memory_enabled: true,
            messages: Vec::new(),
            created_at: String::new(),
            updated_at: String::new(),
        };
        db.save_agent_session(&session).map_err(|e| e.to_string())?;
        for (index, message) in conversation.messages.iter().enumerate() {
            if message.content.trim().is_empty() {
                continue;
            }
            let agent_message =
                migrated_secretary_message(&session_id, conversation.id, index, message);
            db.append_agent_message_if_missing(&agent_message)
                .map_err(|e| e.to_string())?;
            migrated_messages += 1;
        }
        migrated_conversations += 1;
    }

    Ok(json!({
        "active_persona_id": settings.active_persona_id,
        "active_profile_id": settings.active_profile_id,
        "personas_detected": personas.len(),
        "profiles_detected": profiles.len(),
        "memories_detected": memories.len(),
        "reminders_detected": reminders.len(),
        "conversations_detected": conversations.len(),
        "profile_memory_migrated": migrated_profile_memory,
        "memories_migrated": migrated_memories,
        "conversations_migrated": migrated_conversations,
        "messages_migrated": migrated_messages,
        "reminders_preserved_in_secretary_tables": reminders.len()
    }))
}

fn migrated_secretary_profile_memory(
    persona: Option<&crate::models::secretary::SecretaryPersona>,
    profile: Option<&crate::models::secretary::SecretaryProfile>,
) -> Option<SaveAgentMemory> {
    if persona.is_none() && profile.is_none() {
        return None;
    }
    let mut lines = Vec::new();
    if let Some(profile) = profile {
        push_non_empty(&mut lines, "Secretary profile", &profile.name);
        push_non_empty(&mut lines, "Role", &profile.role);
        push_non_empty(&mut lines, "Domain", &profile.domain);
    }
    if let Some(persona) = persona {
        push_non_empty(&mut lines, "Persona", &persona.name);
        push_non_empty(&mut lines, "Voice", &persona.voice);
        push_non_empty(&mut lines, "Values", &persona.values);
        push_non_empty(&mut lines, "Style", &persona.style);
        push_non_empty(&mut lines, "Boundaries", &persona.boundaries);
    }
    if lines.is_empty() {
        return None;
    }
    Some(SaveAgentMemory {
        memory_id: Some("secretary-migrated-profile".to_string()),
        content: format!("Migrated Secretary setup:\n{}", lines.join("\n")),
        scope: "agent".to_string(),
        agent_id: Some("secretary".to_string()),
        status: Some("active".to_string()),
        pinned: Some(true),
        source_session_id: None,
        source_agent_id: Some("secretary".to_string()),
        source_message_id: None,
    })
}

fn migrated_secretary_message(
    session_id: &str,
    conversation_id: i64,
    index: usize,
    message: &SecretaryMessage,
) -> AgentMessage {
    let sender_type = match message.role.as_str() {
        "assistant" | "agent" | "secretary" => 2,
        _ => 1,
    };
    AgentMessage {
        message_id: format!("secretary-conversation-{conversation_id}-message-{index}"),
        session_id: session_id.to_string(),
        sender_type,
        agent_id: if sender_type == 2 {
            Some("secretary".to_string())
        } else {
            None
        },
        content: message.content.clone(),
        turn_index: (index + 1) as i64,
        stream_status: "final".to_string(),
        error_text: String::new(),
        created_at: String::new(),
    }
}

fn create_memory_proposal(
    db: &Database,
    source_session_id: Option<String>,
    source_agent_id: Option<String>,
    source_message_id: Option<String>,
    proposed_text: String,
) -> Result<AgentMemoryProposal, String> {
    if proposed_text.trim().is_empty() {
        return Err("Proposed memory cannot be empty.".to_string());
    }
    let proposal = AgentMemoryProposal {
        proposal_id: format!(
            "agent-memory-proposal-{}",
            Local::now().timestamp_nanos_opt().unwrap_or_default()
        ),
        source_session_id,
        source_agent_id,
        source_message_id,
        proposed_text: truncate_chars(proposed_text.trim(), MAX_PROMPT_SECTION_CHARS),
        status: "pending".to_string(),
        created_at: String::new(),
        updated_at: String::new(),
    };
    db.save_agent_memory_proposal(&proposal)
        .map_err(|e| e.to_string())
}

fn confirm_memory_proposal(
    db: &Database,
    input: ConfirmAgentMemoryProposalInput,
) -> Result<AgentMemoryProposal, String> {
    let mut proposal = db
        .get_agent_memory_proposal(&input.proposal_id)
        .map_err(|e| e.to_string())?;
    if proposal.status != "pending" {
        return Err(format!(
            "Memory proposal is not pending: {}",
            proposal.proposal_id
        ));
    }
    if input.accepted {
        let content = input
            .content
            .as_deref()
            .unwrap_or(&proposal.proposed_text)
            .trim()
            .to_string();
        if content.is_empty() {
            return Err("Confirmed memory content cannot be empty.".to_string());
        }
        let scope = input.scope.unwrap_or_else(|| "global".to_string());
        let agent_id = input
            .agent_id
            .or_else(|| proposal.source_agent_id.clone())
            .filter(|value| !value.trim().is_empty());
        db.save_agent_memory(&SaveAgentMemory {
            memory_id: None,
            content,
            scope,
            agent_id,
            status: Some("active".to_string()),
            pinned: Some(false),
            source_session_id: proposal.source_session_id.clone(),
            source_agent_id: proposal.source_agent_id.clone(),
            source_message_id: proposal.source_message_id.clone(),
        })
        .map_err(|e| e.to_string())?;
        proposal.status = "accepted".to_string();
    } else {
        proposal.status = "rejected".to_string();
    }
    db.save_agent_memory_proposal(&proposal)
        .map_err(|e| e.to_string())
}

fn refresh_conversation_summary(
    db: &Database,
    session_id: &str,
) -> Result<AgentConversationSummary, String> {
    let session = db
        .get_agent_session(session_id)
        .map_err(|e| e.to_string())?;
    let messages = session
        .messages
        .iter()
        .filter(|message| !message.content.trim().is_empty())
        .collect::<Vec<_>>();
    let summary_text = summarize_messages(&messages);
    let topics = conversation_topics(&messages);
    let summary = AgentConversationSummary {
        summary_id: format!("agent-summary-{}", session.session_id),
        session_id: session.session_id.clone(),
        agent_id: session.agent_ids.first().cloned(),
        title: session.session_title.clone(),
        summary: summary_text,
        topics,
        created_at: String::new(),
        updated_at: String::new(),
    };
    db.save_agent_conversation_summary(&summary)
        .map_err(|e| e.to_string())
}

fn summarize_messages(messages: &[&AgentMessage]) -> String {
    if messages.is_empty() {
        return "No conversation messages yet.".to_string();
    }
    bounded_section(
        &messages
            .iter()
            .rev()
            .take(8)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .map(|message| {
                let speaker = if message.sender_type == 2 {
                    message.agent_id.as_deref().unwrap_or("agent")
                } else {
                    "user"
                };
                format!(
                    "{}: {}",
                    speaker,
                    truncate_chars(&message.content, MAX_PREVIOUS_MESSAGE_CHARS)
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

fn conversation_topics(messages: &[&AgentMessage]) -> Vec<String> {
    let stop = [
        "about", "after", "agent", "again", "because", "before", "could", "from", "have", "into",
        "please", "that", "the", "this", "with", "would", "your",
    ];
    let mut topics = Vec::new();
    for word in messages
        .iter()
        .flat_map(|message| message.content.split(|ch: char| !ch.is_alphanumeric()))
        .map(|word| word.trim().to_ascii_lowercase())
        .filter(|word| word.len() >= 4 && !stop.contains(&word.as_str()))
    {
        if !topics.contains(&word) {
            topics.push(word);
        }
        if topics.len() >= 8 {
            break;
        }
    }
    topics
}

fn export_transcript(db: &Database, session_id: &str) -> Result<String, String> {
    let session = db
        .get_agent_session(session_id.trim())
        .map_err(|e| e.to_string())?;
    let actions = db
        .list_agent_tool_actions_for_session(&session.session_id)
        .map_err(|e| e.to_string())?;
    let mut lines = Vec::new();
    lines.push(format!("# {}", session.session_title));
    lines.push(String::new());
    lines.push(format!("- Session ID: `{}`", session.session_id));
    lines.push(format!("- Type: {}", session.session_type));
    lines.push(format!("- Agents: {}", session.agent_ids.join(", ")));
    lines.push(format!("- Created: {}", session.created_at));
    lines.push(format!("- Updated: {}", session.updated_at));
    lines.push(String::new());
    lines.push("## Messages".to_string());
    for message in &session.messages {
        let speaker = if message.sender_type == 2 {
            message.agent_id.as_deref().unwrap_or("agent")
        } else {
            "user"
        };
        lines.push(String::new());
        lines.push(format!(
            "### {} · turn {} · {}",
            speaker, message.turn_index, message.created_at
        ));
        if message.stream_status != "final" || !message.error_text.is_empty() {
            lines.push(format!(
                "_status: {}; error: {}_",
                message.stream_status, message.error_text
            ));
        }
        lines.push(message.content.clone());
    }
    if !actions.is_empty() {
        lines.push(String::new());
        lines.push("## Tool Actions".to_string());
        for action in actions {
            lines.push(String::new());
            lines.push(format!(
                "- `{}` `{}` status `{}` created {}",
                action.action_id, action.tool_name, action.status, action.created_at
            ));
            lines.push("  Arguments:".to_string());
            lines.push(format!(
                "  ```json\n{}\n  ```",
                format_json(&action.arguments)
            ));
            lines.push("  Preview:".to_string());
            lines.push(format!(
                "  ```json\n{}\n  ```",
                format_json(&action.preview)
            ));
        }
    }
    Ok(lines.join("\n"))
}

fn save_message_to_markdown_file(db: &Database, message_id: &str) -> Result<String, String> {
    let message = db
        .get_agent_message(message_id.trim())
        .map_err(|e| e.to_string())?;
    let session = db
        .get_agent_session(&message.session_id)
        .map_err(|e| e.to_string())?;
    let settings = db.get_app_settings().map_err(|e| e.to_string())?;
    let folder = if settings.note_folder.trim().is_empty() {
        dirs::document_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("lazy-todo-app")
            .join("agent-messages")
    } else {
        PathBuf::from(settings.note_folder.trim())
    };
    fs::create_dir_all(&folder).map_err(|e| {
        format!(
            "Cannot create message export folder {}: {e}",
            folder.display()
        )
    })?;
    let speaker = if message.sender_type == 2 {
        message.agent_id.as_deref().unwrap_or("agent")
    } else {
        "user"
    };
    let file_name = format!(
        "{}-{}-{}.md",
        Local::now().format("%Y%m%d-%H%M%S"),
        sanitize_file_stem(speaker),
        sanitize_file_stem(&message.message_id)
    );
    let path = folder.join(file_name);
    let content = format!(
        "# Agent Message\n\n- Session: {}\n- Title: {}\n- Speaker: {}\n- Agent: {}\n- Created: {}\n- Turn: {}\n\n{}",
        message.session_id,
        session.session_title,
        speaker,
        message.agent_id.as_deref().unwrap_or(""),
        message.created_at,
        message.turn_index,
        message.content
    );
    fs::write(&path, content)
        .map_err(|e| format!("Cannot save Agent message to {}: {e}", path.display()))?;
    Ok(path.to_string_lossy().to_string())
}

fn sanitize_file_stem(value: &str) -> String {
    let mut stem = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>();
    while stem.contains("--") {
        stem = stem.replace("--", "-");
    }
    let stem = stem.trim_matches('-');
    if stem.is_empty() {
        "message".to_string()
    } else {
        stem.chars().take(80).collect()
    }
}

fn format_json(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".to_string())
}

fn user_plugin_root(db: &Database) -> Result<PathBuf, String> {
    let settings = db
        .get_agent_plugin_directory_settings()
        .map_err(|e| e.to_string())?;
    let configured = settings.plugin_directory.trim();
    if configured.is_empty() {
        return Err("Configure an Agent plugin directory before installing plugins.".to_string());
    }
    fs::create_dir_all(configured)
        .map_err(|e| format!("Cannot create Agent plugin directory {configured}: {e}"))?;
    let root = fs::canonicalize(configured)
        .map_err(|e| format!("Cannot resolve Agent plugin directory {configured}: {e}"))?;
    if !root.is_dir() {
        return Err(format!(
            "Agent plugin directory is not a directory: {}",
            root.display()
        ));
    }
    Ok(root)
}

fn uninstall_plugin_by_id(db: &Database, plugin_id: &str) -> Result<(), String> {
    let plugin_id = plugin_id.trim();
    if plugin_id.is_empty() {
        return Err("Agent plugin ID is required.".to_string());
    }
    let plugin = find_agent_plugin(db, plugin_id)?;
    if plugin.bundled {
        return Err("Bundled Agent plugins cannot be uninstalled.".to_string());
    }

    let user_root = user_plugin_root(db)?;
    let plugin_path = fs::canonicalize(&plugin.path)
        .map_err(|e| format!("Cannot resolve plugin path {}: {e}", plugin.path))?;
    ensure_direct_child_dir(&user_root, &plugin_path)?;
    fs::remove_dir_all(&plugin_path).map_err(|e| {
        format!(
            "Cannot remove plugin directory {}: {e}",
            plugin_path.display()
        )
    })?;
    db.delete_agent_rag_chunks(plugin_id)
        .map_err(|e| e.to_string())?;
    db.mark_agent_plugin_uninstalled(plugin_id)
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn install_plugin_zip(db: &Database, zip_path: &str) -> Result<AgentPlugin, String> {
    let zip_path = zip_path.trim();
    if zip_path.is_empty() {
        return Err("Plugin ZIP path is required.".to_string());
    }
    let zip_path = fs::canonicalize(zip_path)
        .map_err(|e| format!("Cannot resolve plugin ZIP path {zip_path}: {e}"))?;
    if !zip_path.is_file() {
        return Err(format!(
            "Plugin ZIP path is not a file: {}",
            zip_path.display()
        ));
    }

    let user_root = user_plugin_root(db)?;
    let zip_file = fs::File::open(&zip_path)
        .map_err(|e| format!("Cannot open plugin ZIP {}: {e}", zip_path.display()))?;
    let mut archive =
        zip::ZipArchive::new(zip_file).map_err(|e| format!("Cannot read plugin ZIP: {e}"))?;
    let top_folder = zip_top_folder(&mut archive)?;
    validate_zip_top_folder(&top_folder)?;

    let staging = user_root.join(format!(
        ".installing_{}_{}",
        top_folder,
        Local::now().timestamp_nanos_opt().unwrap_or_default()
    ));
    let final_path = user_root.join(&top_folder);
    if final_path.exists() {
        return Err(format!(
            "Agent plugin folder already exists: {}",
            final_path.display()
        ));
    }
    fs::create_dir_all(&staging)
        .map_err(|e| format!("Cannot create plugin staging directory: {e}"))?;
    let result = extract_plugin_zip_entries(&mut archive, &staging).and_then(|_| {
        let plugin_dir = staging.join(&top_folder);
        let mut seen_ids = HashSet::new();
        let plugin = validate_plugin_dir(&plugin_dir, false, &mut seen_ids);
        if plugin.lifecycle_state == "invalid" {
            let diagnostics = plugin
                .validation_diagnostics
                .iter()
                .map(|diagnostic| diagnostic.message.clone())
                .collect::<Vec<_>>()
                .join("; ");
            return Err(format!("Installed plugin is invalid: {diagnostics}"));
        }
        fs::rename(&plugin_dir, &final_path)
            .map_err(|e| format!("Cannot move plugin into place: {e}"))?;
        Ok(AgentPlugin {
            path: final_path.to_string_lossy().to_string(),
            avatar_path: final_path.join("avatar.png").to_string_lossy().to_string(),
            readme_path: final_path.join("README.md").to_string_lossy().to_string(),
            ..plugin
        })
    });
    let _ = fs::remove_dir_all(&staging);
    let plugin = result?;
    db.upsert_agent_plugin(&plugin).map_err(|e| e.to_string())?;
    Ok(plugin)
}

fn ensure_direct_child_dir(root: &Path, path: &Path) -> Result<(), String> {
    if !path.starts_with(root) {
        return Err(format!(
            "Plugin path is outside the configured Agent plugin directory: {}",
            path.display()
        ));
    }
    let parent = path.parent().ok_or_else(|| {
        format!(
            "Plugin path has no parent directory and cannot be uninstalled: {}",
            path.display()
        )
    })?;
    if parent != root {
        return Err(format!(
            "Only direct child plugin folders can be uninstalled: {}",
            path.display()
        ));
    }
    if !path.is_dir() {
        return Err(format!(
            "Plugin path is not a directory: {}",
            path.display()
        ));
    }
    Ok(())
}

fn zip_top_folder(archive: &mut zip::ZipArchive<fs::File>) -> Result<String, String> {
    let mut top_folders = HashSet::new();
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .map_err(|e| format!("Cannot read ZIP entry {index}: {e}"))?;
        let enclosed = entry
            .enclosed_name()
            .ok_or_else(|| format!("Unsafe ZIP entry path: {}", entry.name()))?;
        let mut components = enclosed.components();
        let Some(first) = components.next() else {
            return Err("ZIP contains an empty entry path.".to_string());
        };
        let std::path::Component::Normal(folder) = first else {
            return Err(format!(
                "ZIP entry is not inside a plugin folder: {}",
                entry.name()
            ));
        };
        let folder = folder
            .to_str()
            .ok_or_else(|| "ZIP top-level folder is not valid UTF-8.".to_string())?
            .to_string();
        let has_child = components.next().is_some();
        if !has_child && !entry.is_dir() {
            return Err(format!(
                "ZIP must contain exactly one plugin folder; found root file {}",
                entry.name()
            ));
        }
        top_folders.insert(folder);
        if top_folders.len() > 1 {
            return Err("ZIP must contain exactly one top-level plugin folder.".to_string());
        }
    }
    top_folders
        .into_iter()
        .next()
        .ok_or_else(|| "Plugin ZIP is empty.".to_string())
}

fn validate_zip_top_folder(folder: &str) -> Result<(), String> {
    if folder.is_empty()
        || folder == "."
        || folder == ".."
        || folder.contains('/')
        || folder.contains('\\')
    {
        return Err(format!("Plugin ZIP folder name is unsafe: {folder}"));
    }
    Ok(())
}

fn extract_plugin_zip_entries(
    archive: &mut zip::ZipArchive<fs::File>,
    destination: &Path,
) -> Result<(), String> {
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|e| format!("Cannot read ZIP entry {index}: {e}"))?;
        let enclosed = entry
            .enclosed_name()
            .ok_or_else(|| format!("Unsafe ZIP entry path: {}", entry.name()))?;
        if entry.size() > MAX_PLUGIN_ZIP_ENTRY_BYTES {
            return Err(format!("ZIP entry is too large: {}", entry.name()));
        }
        if let Some(mode) = entry.unix_mode() {
            if mode & 0o170000 == 0o120000 {
                return Err(format!("ZIP symlinks are not allowed: {}", entry.name()));
            }
        }
        let out_path = destination.join(enclosed);
        if !out_path.starts_with(destination) {
            return Err(format!("ZIP entry escapes destination: {}", entry.name()));
        }
        if entry.is_dir() {
            fs::create_dir_all(&out_path)
                .map_err(|e| format!("Cannot create ZIP directory {}: {e}", out_path.display()))?;
        } else {
            let parent = out_path.parent().ok_or_else(|| {
                format!("ZIP entry has no parent directory: {}", out_path.display())
            })?;
            fs::create_dir_all(parent)
                .map_err(|e| format!("Cannot create ZIP parent {}: {e}", parent.display()))?;
            let mut outfile = fs::File::create(&out_path)
                .map_err(|e| format!("Cannot create ZIP output {}: {e}", out_path.display()))?;
            std::io::copy(&mut entry, &mut outfile)
                .map_err(|e| format!("Cannot extract ZIP entry {}: {e}", entry.name()))?;
        }
    }
    Ok(())
}

async fn send_agent_message_stream_inner(
    app: &AppHandle,
    db: State<'_, Database>,
    input: SendAgentMessageInput,
    stream_id: &str,
) -> Result<SendAgentMessageResult, String> {
    validate_agent_llm_config(&db)?;
    let effective = resolve_effective_agent_llm_settings(&db)?;
    scan_and_persist_agents(app, &db)?;
    let plugin = find_agent_plugin(&db, &input.agent_id)?;
    if !plugin.enabled || plugin.lifecycle_state == "invalid" {
        return Err(format!("Agent is not available: {}", input.agent_id));
    }

    let mut session = match input.session_id.as_deref() {
        Some(session_id) => db
            .get_agent_session(session_id)
            .map_err(|e| e.to_string())?,
        None => start_agent_session(app.clone(), db.clone(), input.agent_id.clone())?,
    };
    let next_turn = session
        .messages
        .iter()
        .map(|message| message.turn_index)
        .max()
        .unwrap_or(0)
        + 1;
    let user_message = AgentMessage {
        message_id: format!(
            "agent-message-{}",
            Local::now().timestamp_nanos_opt().unwrap_or_default()
        ),
        session_id: session.session_id.clone(),
        sender_type: 1,
        agent_id: None,
        content: input.message.trim().to_string(),
        turn_index: next_turn,
        stream_status: "final".to_string(),
        error_text: String::new(),
        created_at: String::new(),
    };
    db.append_agent_message(&user_message)
        .map_err(|e| e.to_string())?;
    session.messages.push(user_message);

    let (system_prompt, used_context) = build_agent_system_prompt(
        &db,
        &plugin,
        &session,
        &input.selected_context,
        &input.message,
    )?;
    let assistant_content = match call_agent_llm_stream(
        &db,
        &effective,
        &system_prompt,
        &session.messages,
        &session.session_id,
        &plugin.plugin_id,
        app,
        stream_id,
    )
    .await
    {
        Ok(content) => content,
        Err(error) => {
            let error_message = AgentMessage {
                message_id: format!(
                    "agent-message-{}",
                    Local::now().timestamp_nanos_opt().unwrap_or_default()
                ),
                session_id: session.session_id.clone(),
                sender_type: 2,
                agent_id: Some(plugin.plugin_id.clone()),
                content: String::new(),
                turn_index: next_turn,
                stream_status: "error".to_string(),
                error_text: error.clone(),
                created_at: String::new(),
            };
            db.append_agent_message(&error_message)
                .map_err(|e| e.to_string())?;
            let _ = refresh_conversation_summary(&db, &session.session_id);
            return Err(error);
        }
    };
    let assistant_message = AgentMessage {
        message_id: format!(
            "agent-message-{}",
            Local::now().timestamp_nanos_opt().unwrap_or_default()
        ),
        session_id: session.session_id.clone(),
        sender_type: 2,
        agent_id: Some(plugin.plugin_id.clone()),
        content: assistant_content,
        turn_index: next_turn,
        stream_status: "final".to_string(),
        error_text: String::new(),
        created_at: String::new(),
    };
    db.append_agent_message(&assistant_message)
        .map_err(|e| e.to_string())?;
    refresh_conversation_summary(&db, &session.session_id)?;
    let session = db
        .get_agent_session(&session.session_id)
        .map_err(|e| e.to_string())?;
    Ok(SendAgentMessageResult {
        session,
        assistant_message,
        used_context,
    })
}

async fn send_agent_group_message_stream_inner(
    app: &AppHandle,
    db: State<'_, Database>,
    input: SendAgentGroupMessageInput,
    stream_id: &str,
) -> Result<SendAgentMessageResult, String> {
    validate_agent_llm_config(&db)?;
    let effective = resolve_effective_agent_llm_settings(&db)?;
    scan_and_persist_agents(app, &db)?;

    let mut session = match input.session_id.as_deref() {
        Some(session_id) => db
            .get_agent_session(session_id)
            .map_err(|e| e.to_string())?,
        None => start_agent_group_session(app.clone(), db.clone(), input.agent_ids.clone())?,
    };

    let target_agent_ids = if input.agent_ids.is_empty() {
        session.agent_ids.clone()
    } else {
        input.agent_ids.clone()
    };
    let plugins = validate_agent_group_plugins(&db, &target_agent_ids)?;
    let next_agent_ids = plugins
        .iter()
        .map(|plugin| plugin.plugin_id.clone())
        .collect::<Vec<_>>();
    if session.agent_ids != next_agent_ids {
        session.agent_ids = next_agent_ids;
        session.session_type = if session.agent_ids.len() > 1 { 2 } else { 1 };
        session = db.save_agent_session(&session).map_err(|e| e.to_string())?;
    }

    let next_turn = session
        .messages
        .iter()
        .map(|message| message.turn_index)
        .max()
        .unwrap_or(0)
        + 1;
    let user_message = AgentMessage {
        message_id: format!(
            "agent-message-{}",
            Local::now().timestamp_nanos_opt().unwrap_or_default()
        ),
        session_id: session.session_id.clone(),
        sender_type: 1,
        agent_id: None,
        content: input.message.trim().to_string(),
        turn_index: next_turn,
        stream_status: "final".to_string(),
        error_text: String::new(),
        created_at: String::new(),
    };
    db.append_agent_message(&user_message)
        .map_err(|e| e.to_string())?;
    session.messages.push(user_message);

    let mut last_assistant_message: Option<AgentMessage> = None;
    let mut merged_context = AgentUsedContext::default();
    for plugin in plugins {
        let (system_prompt, used_context) = build_agent_system_prompt(
            &db,
            &plugin,
            &session,
            &input.selected_context,
            &input.message,
        )?;
        merge_used_context(&mut merged_context, used_context);
        let assistant_content = match call_agent_llm_stream(
            &db,
            &effective,
            &system_prompt,
            &session.messages,
            &session.session_id,
            &plugin.plugin_id,
            app,
            stream_id,
        )
        .await
        {
            Ok(content) => content,
            Err(error) => {
                let error_message = AgentMessage {
                    message_id: format!(
                        "agent-message-{}",
                        Local::now().timestamp_nanos_opt().unwrap_or_default()
                    ),
                    session_id: session.session_id.clone(),
                    sender_type: 2,
                    agent_id: Some(plugin.plugin_id.clone()),
                    content: String::new(),
                    turn_index: next_turn,
                    stream_status: "error".to_string(),
                    error_text: error.clone(),
                    created_at: String::new(),
                };
                db.append_agent_message(&error_message)
                    .map_err(|e| e.to_string())?;
                let _ = refresh_conversation_summary(&db, &session.session_id);
                return Err(error);
            }
        };
        let assistant_message = AgentMessage {
            message_id: format!(
                "agent-message-{}",
                Local::now().timestamp_nanos_opt().unwrap_or_default()
            ),
            session_id: session.session_id.clone(),
            sender_type: 2,
            agent_id: Some(plugin.plugin_id.clone()),
            content: assistant_content,
            turn_index: next_turn,
            stream_status: "final".to_string(),
            error_text: String::new(),
            created_at: String::new(),
        };
        db.append_agent_message(&assistant_message)
            .map_err(|e| e.to_string())?;
        session.messages.push(assistant_message.clone());
        last_assistant_message = Some(assistant_message);
    }

    refresh_conversation_summary(&db, &session.session_id)?;
    let session = db
        .get_agent_session(&session.session_id)
        .map_err(|e| e.to_string())?;
    let assistant_message =
        last_assistant_message.ok_or_else(|| "No Agent responded.".to_string())?;
    Ok(SendAgentMessageResult {
        session,
        assistant_message,
        used_context: merged_context,
    })
}

fn merge_used_context(target: &mut AgentUsedContext, source: AgentUsedContext) {
    append_unique_i64(&mut target.todos, source.todos);
    append_unique_usize(&mut target.milestones, source.milestones);
    append_unique_i64(&mut target.notes, source.notes);
    append_unique_string(&mut target.memories, source.memories);
    append_unique_string(&mut target.rag_chunks, source.rag_chunks);
    append_unique_string(
        &mut target.conversation_summaries,
        source.conversation_summaries,
    );
    append_unique_string(&mut target.previous_messages, source.previous_messages);
}

fn append_unique_i64(target: &mut Vec<i64>, source: Vec<i64>) {
    let mut seen = target.iter().copied().collect::<HashSet<_>>();
    for item in source {
        if seen.insert(item) {
            target.push(item);
        }
    }
}

fn append_unique_usize(target: &mut Vec<usize>, source: Vec<usize>) {
    let mut seen = target.iter().copied().collect::<HashSet<_>>();
    for item in source {
        if seen.insert(item) {
            target.push(item);
        }
    }
}

fn append_unique_string(target: &mut Vec<String>, source: Vec<String>) {
    let mut seen = target.iter().cloned().collect::<HashSet<_>>();
    for item in source {
        if seen.insert(item.clone()) {
            target.push(item);
        }
    }
}

fn build_agent_system_prompt(
    db: &Database,
    plugin: &AgentPlugin,
    session: &AgentSession,
    selected_context: &SelectedAppContext,
    query: &str,
) -> Result<(String, AgentUsedContext), String> {
    let root = PathBuf::from(&plugin.path);
    let prompt = read_bounded_text(&root.join("system_prompt.md"))?;
    let config = read_config(&root)?;
    let ban_topics = if config.ban_topics.is_empty() {
        "none".to_string()
    } else {
        config.ban_topics.join(", ")
    };
    let identity = db.get_agent_user_identity().map_err(|e| e.to_string())?;
    let memories = if session.memory_enabled {
        db.relevant_agent_memories(&plugin.plugin_id, 8)
            .map_err(|e| e.to_string())?
    } else {
        Vec::new()
    };
    let app_context = build_agent_app_context(db, selected_context)?;
    let conversation_summaries = if session.memory_enabled {
        db.relevant_agent_conversation_summaries(&plugin.plugin_id, &session.session_id, 4)
            .map_err(|e| e.to_string())?
    } else {
        Vec::new()
    };
    let previous_messages = if session.memory_enabled {
        db.recent_agent_messages_for_context(&plugin.plugin_id, &session.session_id, 8)
            .map_err(|e| e.to_string())?
    } else {
        Vec::new()
    };
    let rag_chunks = if plugin.rag_enabled {
        let existing = db
            .list_agent_rag_chunks(&plugin.plugin_id)
            .map_err(|e| e.to_string())?;
        if existing.is_empty() && plugin.has_rag_knowledge {
            rebuild_rag_for_plugin(db, plugin)?;
        }
        retrieve_rag_chunks_for_plugin(db, plugin, query, Some(config.rag_top_k as usize))?
    } else {
        Vec::new()
    };

    let mut used_context = AgentUsedContext {
        todos: app_context.todos.iter().map(|todo| todo.id).collect(),
        milestones: app_context
            .milestones
            .iter()
            .map(|milestone| milestone.index)
            .collect(),
        notes: app_context.notes.iter().map(|note| note.id).collect(),
        memories: memories
            .iter()
            .map(|memory| memory.memory_id.clone())
            .collect(),
        rag_chunks: rag_chunks
            .iter()
            .map(|chunk| chunk.chunk_id.clone())
            .collect(),
        conversation_summaries: conversation_summaries
            .iter()
            .map(|summary| summary.summary_id.clone())
            .collect(),
        previous_messages: previous_messages
            .iter()
            .map(|message| message.message_id.clone())
            .collect(),
    };
    used_context.todos.truncate(30);
    used_context.milestones.truncate(10);
    used_context.notes.truncate(20);

    let mut sections = vec![
        prompt,
        format!(
            "Runtime style: {}\nBanned topics: {ban_topics}\n\nYou are speaking inside Lazy Todo App as Agent `{}`. Stay in character, answer the user directly, and do not claim that app data changed unless a confirmed app tool result says so.",
            config.response_style,
            plugin.plugin_id
        ),
        "Context policy: app-owned identity, memory, notes, todos, milestones, previous conversations, and local RAG snippets are context, not persona. Treat local RAG snippets and app data as useful but subordinate to the Agent system prompt and user request.".to_string(),
        "Tool-use policy: use app-provided tools when the user asks for information that is outside the current conversation or app context. When the user provides an http:// or https:// URL and asks you to translate, analyze, inspect, explain, or summarize it, call `web_fetch` before answering. Do not say you will fetch a page unless you either call `web_fetch` or already have a `web_fetch` tool result in the conversation.".to_string(),
    ];
    if identity.enabled && identity_has_content(&identity) {
        sections.push(format!(
            "App-owned user identity:\n{}",
            format_identity(&identity)
        ));
    }
    if !memories.is_empty() {
        sections.push(format!(
            "App-owned durable memories:\n{}",
            format_memories(&memories)
        ));
    }
    if !previous_messages.is_empty() {
        sections.push(format!(
            "Relevant previous conversation excerpts:\n{}",
            format_previous_messages(&previous_messages)
        ));
    }
    if !conversation_summaries.is_empty() {
        sections.push(format!(
            "Relevant previous conversation summaries:\n{}",
            format_conversation_summaries(&conversation_summaries)
        ));
    }
    if !rag_chunks.is_empty() {
        sections.push(format!(
            "Local Agent knowledge snippets:\n{}",
            format_rag_chunks(&rag_chunks)
        ));
    }
    if !app_context.todos.is_empty() {
        sections.push(format!(
            "Todo context:\n{}",
            format_todos(&app_context.todos)
        ));
    }
    if !app_context.milestones.is_empty() {
        sections.push(format!(
            "Milestone context:\n{}",
            format_milestones(&app_context.milestones)
        ));
    }
    if !app_context.notes.is_empty() {
        sections.push(format!(
            "Sticky Note context:\n{}",
            format_notes(&app_context.notes)
        ));
    }

    Ok((sections.join("\n\n"), used_context))
}

fn build_agent_app_context(
    db: &Database,
    selected: &SelectedAppContext,
) -> Result<SecretaryAppContext, String> {
    let todos = if selected.include_todos {
        db.list_todos()
            .map_err(|e| e.to_string())?
            .into_iter()
            .filter(|todo| selected.todo_ids.is_empty() || selected.todo_ids.contains(&todo.id))
            .map(TodoContext::from)
            .collect()
    } else {
        Vec::new()
    };
    let milestones = if selected.include_milestones {
        db.get_pomodoro_settings()
            .map_err(|e| e.to_string())?
            .milestones
            .into_iter()
            .enumerate()
            .filter(|(index, _)| {
                selected.milestone_indexes.is_empty() || selected.milestone_indexes.contains(index)
            })
            .map(|(index, milestone)| MilestoneContext::from_milestone(index, milestone))
            .collect()
    } else {
        Vec::new()
    };
    let notes = if selected.include_notes {
        db.list_notes()
            .map_err(|e| e.to_string())?
            .into_iter()
            .filter(|note| selected.note_ids.is_empty() || selected.note_ids.contains(&note.id))
            .map(|mut note| {
                if note.content.chars().count() > MAX_NOTE_CHARS {
                    note.content = note
                        .content
                        .chars()
                        .take(MAX_NOTE_CHARS)
                        .collect::<String>()
                        + "...";
                }
                NoteContext::from(note)
            })
            .collect()
    } else {
        Vec::new()
    };
    Ok(SecretaryAppContext {
        todos,
        milestones,
        notes,
    })
}

fn retrieve_rag_chunks_for_plugin(
    db: &Database,
    plugin: &AgentPlugin,
    query: &str,
    limit: Option<usize>,
) -> Result<Vec<AgentRagChunk>, String> {
    let mut scored = db
        .list_agent_rag_chunks(&plugin.plugin_id)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|chunk| {
            let score = lexical_score(query, &chunk.chunk_text);
            (score, chunk)
        })
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| left.1.chunk_id.cmp(&right.1.chunk_id))
    });
    let take = limit.unwrap_or(5).clamp(1, 20);
    Ok(scored
        .into_iter()
        .filter(|(score, _)| *score > 0)
        .take(take)
        .map(|(_, chunk)| chunk)
        .collect())
}

fn identity_has_content(identity: &AgentUserIdentity) -> bool {
    !identity.display_name.trim().is_empty()
        || !identity.preferred_language.trim().is_empty()
        || !identity.communication_style.trim().is_empty()
        || !identity.roles.is_empty()
        || !identity.goals.is_empty()
        || !identity.boundaries.trim().is_empty()
        || !identity.important_facts.trim().is_empty()
}

fn format_identity(identity: &AgentUserIdentity) -> String {
    let mut lines = Vec::new();
    push_non_empty(&mut lines, "Name", &identity.display_name);
    push_non_empty(
        &mut lines,
        "Preferred language",
        &identity.preferred_language,
    );
    push_non_empty(
        &mut lines,
        "Communication style",
        &identity.communication_style,
    );
    if !identity.roles.is_empty() {
        lines.push(format!("Roles: {}", identity.roles.join(", ")));
    }
    if !identity.goals.is_empty() {
        lines.push(format!("Goals: {}", identity.goals.join(", ")));
    }
    push_non_empty(&mut lines, "Boundaries", &identity.boundaries);
    push_non_empty(&mut lines, "Important facts", &identity.important_facts);
    bounded_section(&lines.join("\n"))
}

fn format_memories(memories: &[AgentMemory]) -> String {
    bounded_section(
        &memories
            .iter()
            .map(|memory| {
                let scope = memory.agent_id.as_deref().unwrap_or(&memory.scope);
                let pinned = if memory.pinned { " pinned" } else { "" };
                format!("- [{}{pinned}] {}", scope, memory.content)
            })
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

fn format_previous_messages(messages: &[AgentMessage]) -> String {
    bounded_section(
        &messages
            .iter()
            .map(|message| {
                let speaker = if message.sender_type == 2 {
                    message.agent_id.as_deref().unwrap_or("agent")
                } else {
                    "user"
                };
                format!(
                    "- {}: {}",
                    speaker,
                    truncate_chars(&message.content, MAX_PREVIOUS_MESSAGE_CHARS)
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

fn format_conversation_summaries(summaries: &[AgentConversationSummary]) -> String {
    bounded_section(
        &summaries
            .iter()
            .map(|summary| {
                let topics = if summary.topics.is_empty() {
                    "no topics".to_string()
                } else {
                    summary.topics.join(", ")
                };
                format!(
                    "- [{}] {} ({topics})\n{}",
                    summary.summary_id, summary.title, summary.summary
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n"),
    )
}

fn format_rag_chunks(chunks: &[AgentRagChunk]) -> String {
    bounded_section(
        &chunks
            .iter()
            .map(|chunk| format!("- [{}]\n{}", chunk.chunk_id, chunk.chunk_text))
            .collect::<Vec<_>>()
            .join("\n\n"),
    )
}

fn format_todos(todos: &[TodoContext]) -> String {
    bounded_section(
        &todos
            .iter()
            .map(|todo| {
                format!(
                    "- #{} [{}] P{} {}: {}{}",
                    todo.id,
                    if todo.completed { "done" } else { "active" },
                    todo.priority,
                    todo.deadline.as_deref().unwrap_or("no deadline"),
                    todo.title,
                    if todo.description.is_empty() {
                        String::new()
                    } else {
                        format!(" - {}", todo.description)
                    }
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

fn format_milestones(milestones: &[MilestoneContext]) -> String {
    bounded_section(
        &milestones
            .iter()
            .map(|milestone| {
                format!(
                    "- #{} [{}] {} due {}",
                    milestone.index, milestone.status, milestone.name, milestone.deadline
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

fn format_notes(notes: &[NoteContext]) -> String {
    bounded_section(
        &notes
            .iter()
            .map(|note| {
                format!(
                    "- Note #{} \"{}\" ({})\n{}",
                    note.id, note.title, note.color, note.content
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n"),
    )
}

fn push_non_empty(lines: &mut Vec<String>, label: &str, value: &str) {
    if !value.trim().is_empty() {
        lines.push(format!("{label}: {}", value.trim()));
    }
}

fn bounded_section(value: &str) -> String {
    truncate_chars(value, MAX_PROMPT_SECTION_CHARS)
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }
    value.chars().take(max_chars).collect::<String>() + "..."
}

fn validate_agent_llm_config(db: &Database) -> Result<(), String> {
    let effective = resolve_effective_agent_llm_settings(db)?;
    let mut missing = Vec::new();
    if effective.base_url.trim().is_empty() {
        missing.push("LLM_BASE_URL");
    }
    if effective.model.trim().is_empty() {
        missing.push("LLM_MODEL");
    }
    if effective.api_key.trim().is_empty() {
        missing.push("LLM_API_KEY");
    }
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!("Missing LLM setting(s): {}", missing.join(", ")))
    }
}

fn resolve_effective_agent_llm_settings(db: &Database) -> Result<EffectiveLlmSettings, String> {
    let saved = db.get_secretary_settings().map_err(|e| e.to_string())?;
    let saved_api_key = db.get_secretary_saved_api_key().unwrap_or_default();
    let env_base_url = std::env::var("LLM_BASE_URL").unwrap_or_default();
    let env_model = std::env::var("LLM_MODEL").unwrap_or_default();
    let env_api_key = std::env::var("LLM_API_KEY").unwrap_or_default();
    let base_url_from_env = !env_base_url.trim().is_empty();
    let model_from_env = !env_model.trim().is_empty();
    let api_key_from_env = !env_api_key.trim().is_empty();
    let api_key = if api_key_from_env {
        env_api_key
    } else {
        saved_api_key
    };
    Ok(EffectiveLlmSettings {
        base_url: if base_url_from_env {
            env_base_url
        } else {
            saved.base_url
        },
        model: if model_from_env {
            env_model
        } else {
            saved.model
        },
        has_api_key: !api_key.trim().is_empty(),
        api_key,
        base_url_from_env,
        model_from_env,
        api_key_from_env,
    })
}

async fn call_agent_llm_stream(
    db: &Database,
    settings: &EffectiveLlmSettings,
    system_prompt: &str,
    messages: &[AgentMessage],
    session_id: &str,
    agent_id: &str,
    app: &AppHandle,
    stream_id: &str,
) -> Result<String, String> {
    let mut payload_messages = agent_payload_messages(system_prompt, messages);
    let mut final_content = String::new();

    if let Some(url) = messages
        .iter()
        .rev()
        .find(|message| message.sender_type == 1)
        .and_then(|message| web_fetch_url_from_user_message(&message.content))
    {
        let call = LlmToolCall {
            id: "call_prefetch_web_fetch".to_string(),
            name: "web_fetch".to_string(),
            arguments: json!({ "url": url }).to_string(),
        };
        let result = execute_llm_tool_call(db, session_id, agent_id, &call);
        payload_messages.push(assistant_tool_call_message(&LlmStreamOutput {
            content: String::new(),
            tool_calls: vec![call.clone()],
        }));
        payload_messages.push(json!({
            "role": "tool",
            "tool_call_id": normalized_tool_call_id(&call, 0),
            "name": call.name,
            "content": serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string()),
        }));
    }

    for round in 0..MAX_AGENT_TOOL_ROUNDS {
        let tool_schemas = if round == 0 {
            llm_tool_schemas(db)?
        } else {
            Vec::new()
        };
        let output = stream_chat_completion(
            settings,
            &payload_messages,
            app,
            stream_id,
            agent_id,
            &tool_schemas,
        )
        .await?;
        final_content.push_str(&output.content);
        if output.tool_calls.is_empty() {
            return Ok(final_content);
        }

        payload_messages.push(assistant_tool_call_message(&output));
        for (index, call) in output.tool_calls.iter().enumerate() {
            let id = normalized_tool_call_id(call, index);
            let result = execute_llm_tool_call(db, session_id, agent_id, call);
            payload_messages.push(json!({
                "role": "tool",
                "tool_call_id": id,
                "name": call.name,
                "content": serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string()),
            }));
        }
    }

    Ok(final_content)
}

fn agent_payload_messages(system_prompt: &str, messages: &[AgentMessage]) -> Vec<Value> {
    let mut payload_messages = vec![json!({"role": "system", "content": system_prompt})];
    for message in messages
        .iter()
        .rev()
        .take(12)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
    {
        let role = if message.sender_type == 2 {
            "assistant"
        } else {
            "user"
        };
        let content = if message.sender_type == 2 {
            match message.agent_id.as_deref() {
                Some(agent_id) if !agent_id.trim().is_empty() => {
                    format!("@{agent_id}: {}", message.content)
                }
                _ => message.content.clone(),
            }
        } else {
            message.content.clone()
        };
        payload_messages.push(json!({"role": role, "content": content}));
    }
    payload_messages
}

fn assistant_tool_call_message(output: &LlmStreamOutput) -> Value {
    let tool_calls = output
        .tool_calls
        .iter()
        .enumerate()
        .map(|(index, call)| {
            json!({
                "id": normalized_tool_call_id(call, index),
                "type": "function",
                "function": {
                    "name": call.name,
                    "arguments": call.arguments,
                }
            })
        })
        .collect::<Vec<_>>();

    json!({
        "role": "assistant",
        "content": if output.content.is_empty() { Value::Null } else { json!(output.content) },
        "tool_calls": tool_calls,
    })
}

fn normalized_tool_call_id(call: &LlmToolCall, index: usize) -> String {
    if call.id.trim().is_empty() {
        format!("call_{index}")
    } else {
        call.id.clone()
    }
}

fn llm_tool_schemas(db: &Database) -> Result<Vec<Value>, String> {
    let mut tools = builtin_tools()
        .into_iter()
        .map(|tool| {
            json!({
                "type": "function",
                "function": {
                    "name": tool.name,
                    "description": tool.description,
                    "parameters": tool.argument_schema,
                }
            })
        })
        .collect::<Vec<_>>();
    for tool in db
        .list_agent_external_cli_tools()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(annotate_cli_availability)
        .filter(|tool| tool.enabled && tool.available)
    {
        tools.push(json!({
            "type": "function",
            "function": {
                "name": external_cli_function_name(&tool.tool_id),
                "description": format!(
                    "Run registered external CLI tool `{}` through app-owned policy. {}",
                    tool.display_name, tool.safety_class
                ),
                "parameters": tool.argument_schema,
            }
        }));
    }
    Ok(tools)
}

async fn stream_chat_completion(
    settings: &EffectiveLlmSettings,
    payload_messages: &[Value],
    app: &AppHandle,
    stream_id: &str,
    agent_id: &str,
    tool_schemas: &[Value],
) -> Result<LlmStreamOutput, String> {
    let base_url = settings.base_url.trim().trim_end_matches('/');
    let url = if base_url.ends_with("/chat/completions") {
        base_url.to_string()
    } else {
        format!("{base_url}/chat/completions")
    };
    let mut payload = json!({
        "model": settings.model,
        "messages": payload_messages,
        "stream": true,
    });
    if !tool_schemas.is_empty() {
        payload["tools"] = json!(tool_schemas);
        payload["tool_choice"] = json!("auto");
    }
    let client = reqwest::Client::new();
    let mut response = client
        .post(url)
        .bearer_auth(&settings.api_key)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("LLM stream request failed: {e}"))?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!(
            "LLM stream request failed with status {status}: {body}"
        ));
    }

    let mut buffer = String::new();
    let mut output = LlmStreamOutput::default();
    let mut partial_tool_calls: Vec<PartialLlmToolCall> = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| format!("LLM stream read failed: {e}"))?
    {
        buffer.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(index) = buffer.find('\n') {
            let line = buffer[..index].trim().to_string();
            buffer = buffer[index + 1..].to_string();
            let event = parse_sse_event(&line)?;
            if let Some(delta) = event.content {
                output.content.push_str(&delta);
                let _ = app.emit(
                    "agent-stream-chunk",
                    AgentStreamChunk {
                        stream_id: stream_id.to_string(),
                        agent_id: Some(agent_id.to_string()),
                        content: delta,
                    },
                );
            }
            merge_tool_call_deltas(&mut partial_tool_calls, event.tool_calls);
        }
    }
    let trailing = buffer.trim().to_string();
    let event = parse_sse_event(&trailing)?;
    if let Some(delta) = event.content {
        output.content.push_str(&delta);
        let _ = app.emit(
            "agent-stream-chunk",
            AgentStreamChunk {
                stream_id: stream_id.to_string(),
                agent_id: Some(agent_id.to_string()),
                content: delta,
            },
        );
    }
    merge_tool_call_deltas(&mut partial_tool_calls, event.tool_calls);
    output.tool_calls = partial_tool_calls
        .into_iter()
        .enumerate()
        .filter_map(|(index, partial)| {
            let name = partial.name.trim().to_string();
            if name.is_empty() {
                None
            } else {
                Some(LlmToolCall {
                    id: if partial.id.trim().is_empty() {
                        format!("call_{index}")
                    } else {
                        partial.id
                    },
                    name,
                    arguments: partial.arguments,
                })
            }
        })
        .collect();
    Ok(output)
}

fn parse_sse_event(line: &str) -> Result<LlmStreamEvent, String> {
    if line.is_empty() || line.starts_with(':') {
        return Ok(LlmStreamEvent::default());
    }
    let Some(data) = line.strip_prefix("data:") else {
        return Ok(LlmStreamEvent::default());
    };
    let data = data.trim();
    if data == "[DONE]" {
        return Ok(LlmStreamEvent::default());
    }
    let value: serde_json::Value =
        serde_json::from_str(data).map_err(|e| format!("Invalid stream chunk: {e}"))?;
    let content = value
        .pointer("/choices/0/delta/content")
        .or_else(|| value.pointer("/choices/0/message/content"))
        .and_then(|v| v.as_str())
        .map(|v| v.to_string());
    let tool_calls = value
        .pointer("/choices/0/delta/tool_calls")
        .or_else(|| value.pointer("/choices/0/message/tool_calls"))
        .and_then(Value::as_array)
        .map(|calls| {
            calls
                .iter()
                .enumerate()
                .map(|(fallback_index, call)| {
                    let index = call
                        .get("index")
                        .and_then(Value::as_u64)
                        .map(|value| value as usize)
                        .unwrap_or(fallback_index);
                    let partial = PartialLlmToolCall {
                        id: call
                            .get("id")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string(),
                        name: call
                            .pointer("/function/name")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string(),
                        arguments: call
                            .pointer("/function/arguments")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string(),
                    };
                    (index, partial)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Ok(LlmStreamEvent {
        content,
        tool_calls,
    })
}

fn merge_tool_call_deltas(
    partials: &mut Vec<PartialLlmToolCall>,
    deltas: Vec<(usize, PartialLlmToolCall)>,
) {
    for (index, delta) in deltas {
        while partials.len() <= index {
            partials.push(PartialLlmToolCall::default());
        }
        let partial = &mut partials[index];
        if !delta.id.is_empty() {
            partial.id = delta.id;
        }
        if !delta.name.is_empty() {
            partial.name.push_str(&delta.name);
        }
        if !delta.arguments.is_empty() {
            partial.arguments.push_str(&delta.arguments);
        }
    }
}

fn external_cli_function_name(tool_id: &str) -> String {
    format!("external_cli_{}", tool_id.trim())
}

fn external_cli_tool_id_from_function(name: &str) -> Option<String> {
    name.strip_prefix("external_cli_")
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
}

fn execute_llm_tool_call(
    db: &Database,
    session_id: &str,
    agent_id: &str,
    call: &LlmToolCall,
) -> AgentToolCallResult {
    let arguments = if call.arguments.trim().is_empty() {
        json!({})
    } else {
        match serde_json::from_str::<Value>(&call.arguments) {
            Ok(value) => value,
            Err(error) => {
                return record_llm_tool_call_error(
                    db,
                    session_id,
                    agent_id,
                    &call.name,
                    &json!({ "raw_arguments": call.arguments }),
                    "invalid_arguments",
                    format!("Tool arguments are not valid JSON: {error}"),
                )
            }
        }
    };
    if builtin_tools().iter().any(|tool| tool.name == call.name) {
        let input = AgentToolCallInput {
            session_id: Some(session_id.to_string()),
            agent_id: Some(agent_id.to_string()),
            tool_name: call.name.clone(),
            arguments,
        };
        execute_builtin_tool(db, input.clone()).unwrap_or_else(|error| {
            record_llm_tool_call_error(
                db,
                session_id,
                agent_id,
                &input.tool_name,
                &input.arguments,
                "error",
                error,
            )
        })
    } else if let Some(tool_id) = external_cli_tool_id_from_function(&call.name) {
        let input = AgentExternalCliCallInput {
            session_id: Some(session_id.to_string()),
            agent_id: Some(agent_id.to_string()),
            tool_id,
            arguments,
        };
        match execute_external_cli_tool(db, input) {
            Ok(result) => external_cli_result_as_agent_tool_result(&call.name, result),
            Err(error) => record_llm_tool_call_error(
                db,
                session_id,
                agent_id,
                &call.name,
                &json!({}),
                "error",
                error,
            ),
        }
    } else {
        record_llm_tool_call_error(
            db,
            session_id,
            agent_id,
            &call.name,
            &arguments,
            "unsupported_tool",
            format!("Unknown Agent tool: {}", call.name),
        )
    }
}

fn external_cli_result_as_agent_tool_result(
    function_name: &str,
    result: AgentExternalCliCallResult,
) -> AgentToolCallResult {
    let requires_confirmation = result.confirmation_status == "required";
    AgentToolCallResult {
        audit_id: result.audit_id.clone(),
        action_id: None,
        tool_name: function_name.to_string(),
        status: result.status.clone(),
        requires_confirmation,
        result: json!(result),
        message: if requires_confirmation {
            "External CLI tool requires confirmation before execution.".to_string()
        } else {
            "External CLI tool call completed.".to_string()
        },
    }
}

fn record_llm_tool_call_error(
    db: &Database,
    session_id: &str,
    agent_id: &str,
    tool_name: &str,
    arguments: &Value,
    status: &str,
    error: String,
) -> AgentToolCallResult {
    let audit_id = format!(
        "agent-tool-audit-{}",
        Local::now().timestamp_nanos_opt().unwrap_or_default()
    );
    let result = json!({ "error": error });
    let _ = db.insert_agent_builtin_tool_audit(
        &audit_id,
        Some(session_id),
        Some(agent_id),
        tool_name,
        None,
        arguments,
        status,
        &result,
    );
    AgentToolCallResult {
        audit_id,
        action_id: None,
        tool_name: tool_name.to_string(),
        status: status.to_string(),
        requires_confirmation: false,
        result,
        message: "Tool call failed.".to_string(),
    }
}

fn scan_and_persist_agents(app: &AppHandle, db: &Database) -> Result<(), String> {
    let settings = db
        .get_agent_plugin_directory_settings()
        .map_err(|e| e.to_string())?;
    let roots = plugin_roots(app, &settings);
    scan_roots_and_persist(db, roots)
}

fn scan_roots_and_persist(db: &Database, roots: Vec<(PathBuf, bool)>) -> Result<(), String> {
    let mut seen_ids = HashSet::new();
    for (root, bundled) in roots {
        if !root.exists() {
            continue;
        }
        let entries = match fs::read_dir(&root) {
            Ok(entries) => entries,
            Err(error) => {
                if bundled {
                    return Err(format!(
                        "Cannot read bundled plugin directory {}: {error}",
                        root.display()
                    ));
                }
                continue;
            }
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let plugin = validate_plugin_dir(&path, bundled, &mut seen_ids);
            db.upsert_agent_plugin(&plugin).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn plugin_roots(app: &AppHandle, settings: &AgentPluginDirectorySettings) -> Vec<(PathBuf, bool)> {
    let mut roots = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        roots.push((cwd.join("plugins"), true));
        roots.push((cwd.join("..").join("plugins"), true));
    }
    if let Ok(resource_dir) = app.path().resource_dir() {
        roots.push((resource_dir.join("plugins"), true));
    }
    if !settings.plugin_directory.trim().is_empty() {
        roots.push((PathBuf::from(settings.plugin_directory.trim()), false));
    }
    dedupe_roots(roots)
}

fn dedupe_roots(roots: Vec<(PathBuf, bool)>) -> Vec<(PathBuf, bool)> {
    let mut seen = HashSet::new();
    let mut deduped = Vec::new();
    for (root, bundled) in roots {
        let key = root.to_string_lossy().to_string();
        if seen.insert(key) {
            deduped.push((root, bundled));
        }
    }
    deduped
}

fn validate_plugin_dir(path: &Path, bundled: bool, seen_ids: &mut HashSet<String>) -> AgentPlugin {
    let mut diagnostics = Vec::new();
    validate_safe_folder_name(path, &mut diagnostics);
    validate_required_files(path, &mut diagnostics);
    validate_file_sizes(path, &mut diagnostics);

    let manifest = match read_manifest(path) {
        Ok(manifest) => {
            validate_manifest(&manifest, &mut diagnostics);
            Some(manifest)
        }
        Err(message) => {
            diagnostics.push(error("manifest.json", message));
            None
        }
    };
    let _config = match read_config(path) {
        Ok(config) => {
            validate_config(&config, &mut diagnostics);
            Some(config)
        }
        Err(message) => {
            diagnostics.push(error("config.json", message));
            None
        }
    };

    let fallback_id = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("invalid_agent")
        .to_string();
    let plugin_id = manifest
        .as_ref()
        .map(|manifest| manifest.plugin_id.clone())
        .unwrap_or(fallback_id);
    if !seen_ids.insert(plugin_id.clone()) {
        diagnostics.push(error(
            "plugin_id",
            format!("Duplicate plugin ID: {plugin_id}"),
        ));
    }

    let has_rag_knowledge = path.join("rag_knowledge.md").is_file();
    let lifecycle_state = if diagnostics.is_empty() {
        "loaded"
    } else {
        "invalid"
    }
    .to_string();
    let enabled = diagnostics.is_empty();
    AgentPlugin {
        plugin_id,
        plugin_name: manifest
            .as_ref()
            .map(|manifest| manifest.plugin_name.clone())
            .unwrap_or_else(|| "Invalid Agent".to_string()),
        plugin_version: manifest
            .as_ref()
            .map(|manifest| manifest.plugin_version.clone())
            .unwrap_or_else(|| "0.0.0".to_string()),
        author: manifest
            .as_ref()
            .map(|manifest| manifest.author.clone())
            .unwrap_or_default(),
        description: manifest
            .as_ref()
            .map(|manifest| manifest.description.clone())
            .unwrap_or_default(),
        tags: manifest
            .as_ref()
            .map(|manifest| manifest.tags.clone())
            .unwrap_or_default(),
        path: path.to_string_lossy().to_string(),
        avatar_path: path.join("avatar.png").to_string_lossy().to_string(),
        readme_path: path.join("README.md").to_string_lossy().to_string(),
        bundled,
        enabled,
        lifecycle_state,
        rag_enabled: manifest
            .as_ref()
            .map(|manifest| manifest.rag_enabled)
            .unwrap_or(false),
        is_multi_agent_supported: manifest
            .as_ref()
            .map(|manifest| manifest.is_multi_agent_supported)
            .unwrap_or(false),
        has_rag_knowledge,
        validation_diagnostics: diagnostics,
    }
}

fn read_manifest(path: &Path) -> Result<AgentManifest, String> {
    let text = read_bounded_text(&path.join("manifest.json"))?;
    serde_json::from_str(&text).map_err(|error| error.to_string())
}

fn read_config(path: &Path) -> Result<AgentConfig, String> {
    let text = read_bounded_text(&path.join("config.json"))?;
    serde_json::from_str(&text).map_err(|error| error.to_string())
}

fn read_rag_knowledge(path: &Path) -> Result<String, String> {
    read_bounded_text(&path.join("rag_knowledge.md"))
}

fn read_bounded_text(path: &Path) -> Result<String, String> {
    let metadata = fs::metadata(path).map_err(|error| error.to_string())?;
    if metadata.len() > MAX_TEXT_FILE_BYTES {
        return Err(format!("{} is larger than 512KB", path.display()));
    }
    fs::read_to_string(path).map_err(|error| error.to_string())
}

fn validate_safe_folder_name(path: &Path, diagnostics: &mut Vec<AgentValidationDiagnostic>) {
    let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
        diagnostics.push(error("path", "Plugin folder name is not valid UTF-8"));
        return;
    };
    if name.contains('/') || name.contains('\\') || name == "." || name == ".." {
        diagnostics.push(error("path", "Plugin folder name is unsafe"));
    }
}

fn validate_required_files(path: &Path, diagnostics: &mut Vec<AgentValidationDiagnostic>) {
    for file in REQUIRED_PLUGIN_FILES {
        if !path.join(file).is_file() {
            diagnostics.push(error(file, format!("Missing required file: {file}")));
        }
    }
}

fn validate_file_sizes(path: &Path, diagnostics: &mut Vec<AgentValidationDiagnostic>) {
    for file in [
        "manifest.json",
        "system_prompt.md",
        "config.json",
        "README.md",
        "rag_knowledge.md",
    ] {
        let file_path = path.join(file);
        if file_path.exists() {
            match fs::metadata(&file_path) {
                Ok(metadata) if metadata.len() > MAX_TEXT_FILE_BYTES => {
                    diagnostics.push(error(file, format!("{file} is larger than 512KB")));
                }
                Err(io_error) => diagnostics.push(error(file, io_error.to_string())),
                _ => {}
            }
        }
    }
    let avatar_path = path.join("avatar.png");
    if avatar_path.exists() {
        match fs::metadata(&avatar_path) {
            Ok(metadata) if metadata.len() > MAX_AVATAR_BYTES => {
                diagnostics.push(error("avatar.png", "avatar.png is larger than 2MB"));
            }
            Err(io_error) => diagnostics.push(error("avatar.png", io_error.to_string())),
            _ => {}
        }
    }
}

fn validate_manifest(manifest: &AgentManifest, diagnostics: &mut Vec<AgentValidationDiagnostic>) {
    if !valid_plugin_id(&manifest.plugin_id) {
        diagnostics.push(error(
            "plugin_id",
            "Plugin ID must use lowercase letters, digits, underscores, and be at most 32 characters",
        ));
    }
    required_text("plugin_name", &manifest.plugin_name, diagnostics);
    required_text("plugin_version", &manifest.plugin_version, diagnostics);
    required_text("author", &manifest.author, diagnostics);
    required_text("description", &manifest.description, diagnostics);
    required_text("create_time", &manifest.create_time, diagnostics);
    required_text("update_time", &manifest.update_time, diagnostics);
    required_text("min_app_version", &manifest.min_app_version, diagnostics);
    if manifest.tags.iter().any(|tag| tag.trim().is_empty()) {
        diagnostics.push(error("tags", "Tags cannot be empty"));
    }
}

fn validate_config(config: &AgentConfig, diagnostics: &mut Vec<AgentValidationDiagnostic>) {
    if !(0.0..=2.0).contains(&config.temperature) {
        diagnostics.push(error(
            "temperature",
            "temperature must be between 0.0 and 2.0",
        ));
    }
    if !(0.0..=1.0).contains(&config.top_p) {
        diagnostics.push(error("top_p", "top_p must be between 0.0 and 1.0"));
    }
    if config.rag_top_k < 0 || config.rag_top_k > 20 {
        diagnostics.push(error("rag_top_k", "rag_top_k must be between 0 and 20"));
    }
    if config.embedding_dim <= 0 || config.embedding_dim > 8192 {
        diagnostics.push(error(
            "embedding_dim",
            "embedding_dim must be between 1 and 8192",
        ));
    }
    required_text("response_style", &config.response_style, diagnostics);
}

fn chunk_rag_knowledge(content: &str) -> Vec<String> {
    let paragraphs = content
        .lines()
        .map(str::trim)
        .collect::<Vec<_>>()
        .split(|line| line.is_empty())
        .filter_map(|group| {
            let text = group.join("\n").trim().to_string();
            if text.is_empty() {
                None
            } else {
                Some(text)
            }
        })
        .collect::<Vec<_>>();

    let mut chunks = Vec::new();
    let mut current = String::new();
    for paragraph in paragraphs {
        if !current.is_empty()
            && current.chars().count() + paragraph.chars().count() + 2 > RAG_CHUNK_TARGET_CHARS
        {
            chunks.push(current.clone());
            current = overlap_tail(&current, RAG_CHUNK_OVERLAP_CHARS);
        }
        if !current.is_empty() {
            current.push_str("\n\n");
        }
        current.push_str(&paragraph);
    }
    if !current.trim().is_empty() {
        chunks.push(current);
    }
    chunks
}

fn overlap_tail(value: &str, max_chars: usize) -> String {
    let chars = value.chars().collect::<Vec<_>>();
    let start = chars.len().saturating_sub(max_chars);
    chars[start..].iter().collect::<String>().trim().to_string()
}

fn stable_hash_hex(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn lexical_score(query: &str, text: &str) -> usize {
    let text_lower = text.to_lowercase();
    query
        .split(|char: char| !char.is_alphanumeric())
        .map(str::trim)
        .filter(|word| word.len() >= 2)
        .map(|word| text_lower.matches(&word.to_lowercase()).count())
        .sum()
}

fn sqlite_vec_available() -> bool {
    std::env::var("SQLITE_VEC_EXTENSION_PATH")
        .map(|path| !path.trim().is_empty() && Path::new(path.trim()).exists())
        .unwrap_or(false)
}

fn rag_status_message(plugin: &AgentPlugin, indexed_chunks: usize, stale: bool) -> String {
    if !plugin.rag_enabled {
        return "RAG is disabled for this Agent.".to_string();
    }
    if !plugin.has_rag_knowledge {
        return "No rag_knowledge.md file is present.".to_string();
    }
    if indexed_chunks == 0 {
        return "RAG knowledge is ready to be indexed.".to_string();
    }
    if stale {
        return "RAG index is stale and should be rebuilt.".to_string();
    }
    if sqlite_vec_available() {
        "RAG chunks are indexed; sqlite-vec extension path is configured.".to_string()
    } else {
        "RAG chunks are indexed as metadata; sqlite-vec extension is not configured.".to_string()
    }
}

fn valid_plugin_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 32
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn required_text(field: &str, value: &str, diagnostics: &mut Vec<AgentValidationDiagnostic>) {
    if value.trim().is_empty() {
        diagnostics.push(error(field, format!("{field} is required")));
    }
}

fn error(field: impl Into<String>, message: impl Into<String>) -> AgentValidationDiagnostic {
    AgentValidationDiagnostic {
        severity: "error".to_string(),
        field: Some(field.into()),
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_test_dir(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("lazy_todo_agents_{name}_{suffix}"))
    }

    fn write_valid_plugin(root: &Path, plugin_id: &str) {
        fs::create_dir_all(root).expect("create plugin dir");
        fs::write(
            root.join("manifest.json"),
            format!(
                r#"{{
  "plugin_id": "{plugin_id}",
  "plugin_name": "Test Agent",
  "plugin_version": "1.0.0",
  "author": "Test",
  "description": "A test Agent",
  "tags": ["test"],
  "create_time": "2026-04-29",
  "update_time": "2026-04-29",
  "min_app_version": "1.0.0",
  "rag_enabled": false,
  "is_multi_agent_supported": true
}}"#
            ),
        )
        .expect("write manifest");
        fs::write(root.join("system_prompt.md"), "You are a test Agent.").expect("write prompt");
        fs::write(
            root.join("config.json"),
            r#"{
  "temperature": 0.5,
  "top_p": 0.8,
  "rag_top_k": 3,
  "embedding_dim": 1536,
  "response_style": "test",
  "ban_topics": []
}"#,
        )
        .expect("write config");
        fs::write(root.join("avatar.png"), [137, 80, 78, 71]).expect("write avatar");
        fs::write(root.join("README.md"), "# Test Agent").expect("write readme");
    }

    fn write_valid_plugin_zip(zip_path: &Path, folder_name: &str, plugin_id: &str) {
        let file = fs::File::create(zip_path).expect("create zip");
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip.add_directory(format!("{folder_name}/"), options)
            .expect("add folder");
        let files = [
            (
                "manifest.json",
                format!(
                    r#"{{
  "plugin_id": "{plugin_id}",
  "plugin_name": "Zipped Agent",
  "plugin_version": "1.0.0",
  "author": "Test",
  "description": "A zipped Agent",
  "tags": ["zip"],
  "create_time": "2026-04-29",
  "update_time": "2026-04-29",
  "min_app_version": "1.0.0",
  "rag_enabled": false,
  "is_multi_agent_supported": true
}}"#
                ),
            ),
            ("system_prompt.md", "You are a zipped Agent.".to_string()),
            (
                "config.json",
                r#"{
  "temperature": 0.5,
  "top_p": 0.8,
  "rag_top_k": 3,
  "embedding_dim": 1536,
  "response_style": "test",
  "ban_topics": []
}"#
                .to_string(),
            ),
            ("avatar.png", "\u{89}PNG".to_string()),
            ("README.md", "# Zipped Agent".to_string()),
        ];
        for (name, content) in files {
            zip.start_file(format!("{folder_name}/{name}"), options)
                .expect("start file");
            zip.write_all(content.as_bytes()).expect("write file");
        }
        zip.finish().expect("finish zip");
    }

    fn write_unsafe_zip(zip_path: &Path) {
        let file = fs::File::create(zip_path).expect("create zip");
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip.start_file("../escape.txt", options)
            .expect("start unsafe file");
        zip.write_all(b"escape").expect("write unsafe file");
        zip.finish().expect("finish unsafe zip");
    }

    #[test]
    fn validates_complete_plugin_directory() {
        let root = unique_test_dir("valid");
        write_valid_plugin(&root, "test_agent");
        let mut seen = HashSet::new();

        let plugin = validate_plugin_dir(&root, true, &mut seen);

        assert_eq!(plugin.plugin_id, "test_agent");
        assert_eq!(plugin.lifecycle_state, "loaded");
        assert!(plugin.enabled);
        assert!(plugin.validation_diagnostics.is_empty());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn reports_missing_required_file() {
        let root = unique_test_dir("missing");
        write_valid_plugin(&root, "missing_agent");
        fs::remove_file(root.join("README.md")).expect("remove readme");
        let mut seen = HashSet::new();

        let plugin = validate_plugin_dir(&root, false, &mut seen);

        assert_eq!(plugin.lifecycle_state, "invalid");
        assert!(plugin
            .validation_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("README.md")));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn reports_duplicate_plugin_id() {
        let root = unique_test_dir("duplicate");
        write_valid_plugin(&root, "duplicate_agent");
        let mut seen = HashSet::new();
        seen.insert("duplicate_agent".to_string());

        let plugin = validate_plugin_dir(&root, false, &mut seen);

        assert_eq!(plugin.lifecycle_state, "invalid");
        assert!(plugin
            .validation_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("Duplicate plugin ID")));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn reports_invalid_config_ranges() {
        let root = unique_test_dir("bad_config");
        write_valid_plugin(&root, "bad_config_agent");
        fs::write(
            root.join("config.json"),
            r#"{
  "temperature": 9.0,
  "top_p": 2.0,
  "rag_top_k": 50,
  "embedding_dim": 0,
  "response_style": "",
  "ban_topics": []
}"#,
        )
        .expect("write bad config");
        let mut seen = HashSet::new();

        let plugin = validate_plugin_dir(&root, false, &mut seen);

        assert_eq!(plugin.lifecycle_state, "invalid");
        for field in [
            "temperature",
            "top_p",
            "rag_top_k",
            "embedding_dim",
            "response_style",
        ] {
            assert!(plugin
                .validation_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.field.as_deref() == Some(field)));
        }
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn chunks_rag_knowledge_deterministically() {
        let content = "# One\n\nAlpha beta gamma.\n\n# Two\n\nDelta epsilon.";
        let chunks = chunk_rag_knowledge(content);

        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].contains("# One"));
        assert!(chunks[0].contains("# Two"));
        assert_eq!(
            stable_hash_hex(content.as_bytes()),
            stable_hash_hex(content.as_bytes())
        );
    }

    #[test]
    fn scores_relevant_rag_chunks_higher() {
        let low = lexical_score("ren education", "ritual propriety and daily conduct");
        let high = lexical_score(
            "ren education",
            "ren supports education and self cultivation",
        );

        assert!(high > low);
    }

    #[test]
    fn exposes_builtin_tools_as_openai_function_schemas() {
        let db_root = unique_test_dir("llm_builtin_schema_db");
        let db = Database::new(&db_root).expect("create db");
        let tools = llm_tool_schemas(&db).expect("tool schemas");

        assert!(tools.iter().any(|tool| {
            tool.pointer("/function/name").and_then(Value::as_str) == Some("read_todo_list")
        }));
        let write_file = tools
            .iter()
            .find(|tool| {
                tool.pointer("/function/name").and_then(Value::as_str) == Some("write_file")
            })
            .expect("write_file schema");
        assert_eq!(
            write_file.get("type").and_then(Value::as_str),
            Some("function")
        );
        assert_eq!(
            write_file
                .pointer("/function/parameters/required/0")
                .and_then(Value::as_str),
            Some("path")
        );
        let _ = fs::remove_dir_all(db_root);
    }

    #[test]
    fn parses_streamed_tool_call_chunks() {
        let first = parse_sse_event(
            r#"data: {"choices":[{"delta":{"tool_calls":[{"index":0,"id":"call_abc","type":"function","function":{"name":"read_todo_list","arguments":"{\"include_"}}]}}]}"#,
        )
        .expect("first chunk");
        let second = parse_sse_event(
            r#"data: {"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":"completed\":false}"}}]}}]}"#,
        )
        .expect("second chunk");
        let mut partials = Vec::new();
        merge_tool_call_deltas(&mut partials, first.tool_calls);
        merge_tool_call_deltas(&mut partials, second.tool_calls);

        assert_eq!(partials.len(), 1);
        assert_eq!(partials[0].id, "call_abc");
        assert_eq!(partials[0].name, "read_todo_list");
        assert_eq!(partials[0].arguments, r#"{"include_completed":false}"#);
    }

    #[test]
    fn llm_tool_call_routes_through_builtin_executor() {
        let db_root = unique_test_dir("llm_tool_route_db");
        let db = Database::new(&db_root).expect("create db");
        db.add_todo("Answer through a tool", "", 2, None, None, None, None, None)
            .expect("add todo");
        let call = LlmToolCall {
            id: "call_todos".to_string(),
            name: "read_todo_list".to_string(),
            arguments: r#"{"include_completed":false}"#.to_string(),
        };

        let result = execute_llm_tool_call(&db, "session-tools", "secretary", &call);

        assert_eq!(result.status, "completed");
        assert_eq!(result.tool_name, "read_todo_list");
        assert_eq!(
            result
                .result
                .pointer("/todos/0/title")
                .and_then(Value::as_str),
            Some("Answer through a tool")
        );
        let _ = fs::remove_dir_all(db_root);
    }

    #[test]
    fn llm_tool_schemas_include_enabled_external_cli_tools() {
        let db_root = unique_test_dir("llm_external_schema_db");
        let db = Database::new(&db_root).expect("create db");
        let input = normalize_external_cli_tool(SaveAgentExternalCliTool {
            tool_id: Some("echo_tool".to_string()),
            display_name: "Echo Tool".to_string(),
            executable: "/bin/echo".to_string(),
            allowed_subcommands: Vec::new(),
            argument_schema: json!({
                "type": "object",
                "properties": {
                    "args": { "type": "array", "items": { "type": "string" } }
                }
            }),
            working_directory: String::new(),
            environment_allowlist: Vec::new(),
            timeout_ms: 5_000,
            output_limit: 1_024,
            safety_class: "read".to_string(),
            enabled: true,
        })
        .expect("normalize");
        validate_external_cli_registration(&input).expect("validate");
        db.save_agent_external_cli_tool(&input)
            .expect("save external cli");

        let tools = llm_tool_schemas(&db).expect("tool schemas");

        assert!(tools.iter().any(|tool| {
            tool.pointer("/function/name").and_then(Value::as_str) == Some("external_cli_echo_tool")
        }));
        let _ = fs::remove_dir_all(db_root);
    }

    #[test]
    fn llm_tool_call_routes_through_external_cli_executor() {
        let db_root = unique_test_dir("llm_external_route_db");
        let db = Database::new(&db_root).expect("create db");
        let input = normalize_external_cli_tool(SaveAgentExternalCliTool {
            tool_id: Some("echo_tool".to_string()),
            display_name: "Echo Tool".to_string(),
            executable: "/bin/echo".to_string(),
            allowed_subcommands: Vec::new(),
            argument_schema: json!({
                "type": "object",
                "properties": {
                    "args": { "type": "array", "items": { "type": "string" } }
                }
            }),
            working_directory: String::new(),
            environment_allowlist: Vec::new(),
            timeout_ms: 5_000,
            output_limit: 1_024,
            safety_class: "read".to_string(),
            enabled: true,
        })
        .expect("normalize");
        validate_external_cli_registration(&input).expect("validate");
        db.save_agent_external_cli_tool(&input)
            .expect("save external cli");
        let call = LlmToolCall {
            id: "call_echo".to_string(),
            name: "external_cli_echo_tool".to_string(),
            arguments: r#"{"args":["hello"]}"#.to_string(),
        };

        let result = execute_llm_tool_call(&db, "session-tools", "secretary", &call);

        assert_eq!(result.status, "completed");
        assert_eq!(result.tool_name, "external_cli_echo_tool");
        assert_eq!(
            result
                .result
                .pointer("/stdout")
                .and_then(Value::as_str)
                .map(str::trim),
            Some("hello")
        );
        let _ = fs::remove_dir_all(db_root);
    }

    #[test]
    fn validates_external_cli_registration_inputs() {
        let valid = SaveAgentExternalCliTool {
            tool_id: Some("echo_tool".to_string()),
            display_name: "Echo Tool".to_string(),
            executable: "/bin/echo".to_string(),
            allowed_subcommands: vec!["hello".to_string()],
            argument_schema: json!({ "type": "object", "properties": {} }),
            working_directory: String::new(),
            environment_allowlist: vec!["PATH".to_string()],
            timeout_ms: 5_000,
            output_limit: 4_096,
            safety_class: "read".to_string(),
            enabled: true,
        };

        let normalized = normalize_external_cli_tool(valid).expect("normalize");
        validate_external_cli_registration(&normalized).expect("valid registration");

        let missing = SaveAgentExternalCliTool {
            executable: "/definitely/not/a/real/executable".to_string(),
            ..normalized.clone()
        };
        assert!(validate_external_cli_registration(&missing)
            .expect_err("missing executable")
            .contains("unavailable"));

        let bad_schema = SaveAgentExternalCliTool {
            argument_schema: json!({ "type": "array" }),
            ..normalized.clone()
        };
        assert!(validate_external_cli_registration(&bad_schema)
            .expect_err("bad schema")
            .contains("argument_schema.type"));

        let bad_subcommand = SaveAgentExternalCliTool {
            allowed_subcommands: vec!["hello;rm".to_string()],
            ..normalized
        };
        assert!(validate_external_cli_registration(&bad_subcommand)
            .expect_err("bad subcommand")
            .contains("shell metacharacters"));
    }

    #[test]
    fn persists_external_cli_tool_registration() {
        let db_root = unique_test_dir("external_cli_db");
        let db = Database::new(&db_root).expect("create db");
        let input = normalize_external_cli_tool(SaveAgentExternalCliTool {
            tool_id: None,
            display_name: "Echo Tool".to_string(),
            executable: "/bin/echo".to_string(),
            allowed_subcommands: vec!["hello".to_string()],
            argument_schema: json!({ "type": "object", "properties": {} }),
            working_directory: String::new(),
            environment_allowlist: vec!["PATH".to_string()],
            timeout_ms: 5_000,
            output_limit: 4_096,
            safety_class: "read".to_string(),
            enabled: true,
        })
        .expect("normalize");
        validate_external_cli_registration(&input).expect("validate");

        let saved = db
            .save_agent_external_cli_tool(&input)
            .expect("save external cli");
        assert_eq!(saved.tool_id, "echo_tool");
        assert_eq!(saved.allowed_subcommands, vec!["hello"]);
        assert!(saved.enabled);

        let disabled = db
            .set_agent_external_cli_tool_enabled("echo_tool", false)
            .expect("disable external cli");
        assert!(!disabled.enabled);
        assert_eq!(
            db.list_agent_external_cli_tools()
                .expect("list external cli")
                .len(),
            1
        );

        db.delete_agent_external_cli_tool("echo_tool")
            .expect("delete external cli");
        assert!(db
            .list_agent_external_cli_tools()
            .expect("list after delete")
            .is_empty());
        let _ = fs::remove_dir_all(db_root);
    }

    #[test]
    fn executes_external_cli_with_validated_arguments_and_bounded_output() {
        let db_root = unique_test_dir("external_cli_execute_db");
        let db = Database::new(&db_root).expect("create db");
        let input = normalize_external_cli_tool(SaveAgentExternalCliTool {
            tool_id: Some("echo_tool".to_string()),
            display_name: "Echo Tool".to_string(),
            executable: "/bin/echo".to_string(),
            allowed_subcommands: Vec::new(),
            argument_schema: json!({
                "type": "object",
                "properties": {
                    "args": { "type": "array", "items": { "type": "string" } }
                }
            }),
            working_directory: String::new(),
            environment_allowlist: Vec::new(),
            timeout_ms: 5_000,
            output_limit: 1_024,
            safety_class: "read".to_string(),
            enabled: true,
        })
        .expect("normalize");
        validate_external_cli_registration(&input).expect("validate");
        db.save_agent_external_cli_tool(&input)
            .expect("save external cli");

        let result = execute_external_cli_tool(
            &db,
            AgentExternalCliCallInput {
                session_id: Some("session-cli".to_string()),
                agent_id: Some("secretary".to_string()),
                tool_id: "echo_tool".to_string(),
                arguments: json!({ "args": ["hello"] }),
            },
        )
        .expect("execute cli");
        assert_eq!(result.status, "completed");
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(result.stdout.trim(), "hello");

        let invalid = execute_external_cli_tool(
            &db,
            AgentExternalCliCallInput {
                session_id: Some("session-cli".to_string()),
                agent_id: Some("secretary".to_string()),
                tool_id: "echo_tool".to_string(),
                arguments: json!({ "unknown": true }),
            },
        )
        .expect("invalid call result");
        assert_eq!(invalid.status, "validation_error");
        assert!(invalid.stderr.contains("not allowed"));

        let long = execute_external_cli_tool(
            &db,
            AgentExternalCliCallInput {
                session_id: Some("session-cli".to_string()),
                agent_id: Some("secretary".to_string()),
                tool_id: "echo_tool".to_string(),
                arguments: json!({ "args": ["x".repeat(2_000)] }),
            },
        )
        .expect("long output");
        assert_eq!(long.status, "completed");
        assert!(long.truncated);
        assert_eq!(long.stdout.len(), 1_024);
        let _ = fs::remove_dir_all(db_root);
    }

    #[test]
    fn external_cli_audit_masks_sensitive_arguments() {
        let db_root = unique_test_dir("external_cli_mask_db");
        let db = Database::new(&db_root).expect("create db");
        let input = normalize_external_cli_tool(SaveAgentExternalCliTool {
            tool_id: Some("echo_tool".to_string()),
            display_name: "Echo Tool".to_string(),
            executable: "/bin/echo".to_string(),
            allowed_subcommands: Vec::new(),
            argument_schema: json!({
                "type": "object",
                "properties": {
                    "args": { "type": "array", "items": { "type": "string" } },
                    "api_key": { "type": "string" }
                }
            }),
            working_directory: String::new(),
            environment_allowlist: Vec::new(),
            timeout_ms: 5_000,
            output_limit: 1_024,
            safety_class: "read".to_string(),
            enabled: true,
        })
        .expect("normalize");
        validate_external_cli_registration(&input).expect("validate");
        db.save_agent_external_cli_tool(&input)
            .expect("save external cli");

        execute_external_cli_tool(
            &db,
            AgentExternalCliCallInput {
                session_id: Some("session-cli".to_string()),
                agent_id: Some("secretary".to_string()),
                tool_id: "echo_tool".to_string(),
                arguments: json!({ "args": ["hello"], "api_key": "secret-value" }),
            },
        )
        .expect("execute cli");

        let conn = rusqlite::Connection::open(db.db_path()).expect("open db");
        let arguments_json: String = conn
            .query_row(
                "SELECT arguments_json FROM agent_external_cli_audit WHERE tool_id = 'echo_tool' ORDER BY created_at DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("audit arguments");
        assert!(arguments_json.contains("***"));
        assert!(!arguments_json.contains("secret-value"));
        let _ = fs::remove_dir_all(db_root);
    }

    #[test]
    fn external_cli_write_tool_creates_pending_action_and_executes_after_confirmation() {
        let db_root = unique_test_dir("external_cli_confirm_db");
        let db = Database::new(&db_root).expect("create db");
        let input = normalize_external_cli_tool(SaveAgentExternalCliTool {
            tool_id: Some("echo_write_tool".to_string()),
            display_name: "Echo Write".to_string(),
            executable: "/bin/echo".to_string(),
            allowed_subcommands: Vec::new(),
            argument_schema: json!({
                "type": "object",
                "properties": {
                    "args": { "type": "array", "items": { "type": "string" } }
                }
            }),
            working_directory: String::new(),
            environment_allowlist: Vec::new(),
            timeout_ms: 5_000,
            output_limit: 4_096,
            safety_class: "write".to_string(),
            enabled: true,
        })
        .expect("normalize");
        validate_external_cli_registration(&input).expect("validate");
        db.save_agent_external_cli_tool(&input)
            .expect("save external cli");

        let result = execute_external_cli_tool(
            &db,
            AgentExternalCliCallInput {
                session_id: Some("session-cli".to_string()),
                agent_id: Some("secretary".to_string()),
                tool_id: "echo_write_tool".to_string(),
                arguments: json!({ "args": ["confirmed"] }),
            },
        )
        .expect("execute cli");
        assert_eq!(result.status, "requires_confirmation");
        assert_eq!(result.confirmation_status, "required");
        let actions = db
            .list_pending_agent_tool_actions()
            .expect("pending actions");
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].tool_name, "external_cli_echo_write_tool");
        assert_eq!(
            actions[0].preview.pointer("/kind").and_then(Value::as_str),
            Some("external_cli")
        );

        let confirmed = confirm_pending_tool_action(
            &db,
            ConfirmAgentToolActionInput {
                action_id: actions[0].action_id.clone(),
                accepted: true,
            },
        )
        .expect("confirm cli");
        assert_eq!(confirmed.status, "completed");
        assert_eq!(
            confirmed
                .result
                .pointer("/confirmation_status")
                .and_then(Value::as_str),
            Some("accepted")
        );
        assert!(confirmed
            .result
            .pointer("/stdout")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .contains("confirmed"));
        let _ = fs::remove_dir_all(db_root);
    }

    #[test]
    fn external_cli_execution_enforces_timeout() {
        if !Path::new("/bin/sleep").exists() {
            return;
        }
        let db_root = unique_test_dir("external_cli_timeout_db");
        let db = Database::new(&db_root).expect("create db");
        let input = normalize_external_cli_tool(SaveAgentExternalCliTool {
            tool_id: Some("sleep_tool".to_string()),
            display_name: "Sleep Tool".to_string(),
            executable: "/bin/sleep".to_string(),
            allowed_subcommands: Vec::new(),
            argument_schema: json!({
                "type": "object",
                "properties": {
                    "args": { "type": "array", "items": { "type": "string" } }
                }
            }),
            working_directory: String::new(),
            environment_allowlist: Vec::new(),
            timeout_ms: 1_000,
            output_limit: 1_024,
            safety_class: "read".to_string(),
            enabled: true,
        })
        .expect("normalize");
        validate_external_cli_registration(&input).expect("validate");
        db.save_agent_external_cli_tool(&input)
            .expect("save external cli");

        let result = execute_external_cli_tool(
            &db,
            AgentExternalCliCallInput {
                session_id: Some("session-cli".to_string()),
                agent_id: Some("secretary".to_string()),
                tool_id: "sleep_tool".to_string(),
                arguments: json!({ "args": ["2"] }),
            },
        )
        .expect("execute cli");
        assert_eq!(result.status, "timeout");
        assert!(result.timed_out);
        let _ = fs::remove_dir_all(db_root);
    }

    #[test]
    fn validates_bundled_agent_plugins() {
        let plugin_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace root")
            .join("plugins");
        let mut seen = HashSet::new();

        let secretary = validate_plugin_dir(&plugin_root.join("secretary"), true, &mut seen);
        let confucius = validate_plugin_dir(&plugin_root.join("confucius"), true, &mut seen);
        let socrates = validate_plugin_dir(&plugin_root.join("socrates"), true, &mut seen);
        let steve_jobs = validate_plugin_dir(&plugin_root.join("steve_jobs"), true, &mut seen);
        let uncle_bob = validate_plugin_dir(&plugin_root.join("uncle_bob"), true, &mut seen);
        let english_teacher =
            validate_plugin_dir(&plugin_root.join("english_teacher"), true, &mut seen);

        assert_eq!(secretary.plugin_id, "secretary");
        assert_eq!(secretary.lifecycle_state, "loaded");
        assert!(secretary.has_rag_knowledge);
        assert!(secretary.validation_diagnostics.is_empty());
        assert_eq!(confucius.plugin_id, "confucius");
        assert_eq!(confucius.lifecycle_state, "loaded");
        assert!(confucius.has_rag_knowledge);
        assert!(confucius.validation_diagnostics.is_empty());
        assert_eq!(socrates.plugin_id, "socrates");
        assert_eq!(socrates.lifecycle_state, "loaded");
        assert!(socrates.has_rag_knowledge);
        assert!(socrates.validation_diagnostics.is_empty());
        assert_eq!(steve_jobs.plugin_id, "steve_jobs");
        assert_eq!(steve_jobs.lifecycle_state, "loaded");
        assert!(steve_jobs.has_rag_knowledge);
        assert!(steve_jobs.validation_diagnostics.is_empty());
        assert_eq!(uncle_bob.plugin_id, "uncle_bob");
        assert_eq!(uncle_bob.lifecycle_state, "loaded");
        assert!(uncle_bob.has_rag_knowledge);
        assert!(uncle_bob.validation_diagnostics.is_empty());
        assert_eq!(english_teacher.plugin_id, "english_teacher");
        assert_eq!(english_teacher.lifecycle_state, "loaded");
        assert!(english_teacher.has_rag_knowledge);
        assert!(english_teacher.validation_diagnostics.is_empty());
    }

    #[test]
    fn validates_and_persists_group_agent_session() {
        let db_root = unique_test_dir("group_agent_session_db");
        let plugin_root = unique_test_dir("group_agent_session_plugins");
        let agent_one_dir = plugin_root.join("agent_one");
        let agent_two_dir = plugin_root.join("agent_two");
        write_valid_plugin(&agent_one_dir, "agent_one");
        write_valid_plugin(&agent_two_dir, "agent_two");
        let db = Database::new(&db_root).expect("create db");
        let mut seen = HashSet::new();
        let agent_one = validate_plugin_dir(&agent_one_dir, false, &mut seen);
        let agent_two = validate_plugin_dir(&agent_two_dir, false, &mut seen);
        db.upsert_agent_plugin(&agent_one)
            .expect("upsert agent one");
        db.upsert_agent_plugin(&agent_two)
            .expect("upsert agent two");

        let plugins = validate_agent_group_plugins(
            &db,
            &[
                "agent_one".to_string(),
                "agent_two".to_string(),
                "agent_one".to_string(),
            ],
        )
        .expect("validate group");
        assert_eq!(
            plugins
                .iter()
                .map(|plugin| plugin.plugin_id.as_str())
                .collect::<Vec<_>>(),
            vec!["agent_one", "agent_two"]
        );

        let session = db
            .save_agent_session(&AgentSession {
                session_id: "group-session-test".to_string(),
                session_type: 2,
                agent_ids: plugins
                    .iter()
                    .map(|plugin| plugin.plugin_id.clone())
                    .collect(),
                session_title: "Group chat: Test Agent, Test Agent".to_string(),
                memory_enabled: true,
                messages: Vec::new(),
                created_at: String::new(),
                updated_at: String::new(),
            })
            .expect("save group session");
        assert_eq!(session.session_type, 2);
        assert_eq!(session.agent_ids, vec!["agent_one", "agent_two"]);
        let _ = fs::remove_dir_all(db_root);
        let _ = fs::remove_dir_all(plugin_root);
    }

    #[test]
    fn installs_plugin_zip_and_rejects_path_traversal() {
        let db_root = unique_test_dir("zip_install_db");
        let plugin_root = unique_test_dir("zip_install_plugins");
        let zip_path = unique_test_dir("plugin_zip").with_extension("zip");
        let unsafe_zip_path = unique_test_dir("unsafe_plugin_zip").with_extension("zip");
        fs::create_dir_all(&plugin_root).expect("create plugin root");
        write_valid_plugin_zip(&zip_path, "zipped_agent", "zipped_agent");
        write_unsafe_zip(&unsafe_zip_path);
        let db = Database::new(&db_root).expect("create db");
        db.save_agent_plugin_directory_settings(&SaveAgentPluginDirectorySettings {
            plugin_directory: plugin_root.to_string_lossy().to_string(),
        })
        .expect("save plugin dir");

        let plugin =
            install_plugin_zip(&db, zip_path.to_str().expect("zip path")).expect("install zip");
        assert_eq!(plugin.plugin_id, "zipped_agent");
        assert!(plugin_root.join("zipped_agent").is_dir());
        assert!(db
            .list_agent_plugins()
            .expect("list plugins")
            .iter()
            .any(|plugin| plugin.plugin_id == "zipped_agent"));

        let error = install_plugin_zip(&db, unsafe_zip_path.to_str().expect("unsafe zip path"))
            .expect_err("reject traversal");
        assert!(error.contains("Unsafe ZIP entry path") || error.contains("plugin folder"));
        assert!(!plugin_root.join("escape.txt").exists());
        let _ = fs::remove_dir_all(db_root);
        let _ = fs::remove_dir_all(plugin_root);
        let _ = fs::remove_file(zip_path);
        let _ = fs::remove_file(unsafe_zip_path);
    }

    #[test]
    fn uninstalls_user_plugin_and_cleans_rag_but_rejects_unsafe_paths() {
        let db_root = unique_test_dir("uninstall_db");
        let plugin_root = unique_test_dir("uninstall_plugins");
        let plugin_dir = plugin_root.join("test_agent");
        let outside_dir = unique_test_dir("outside_plugin");
        fs::create_dir_all(&plugin_root).expect("create plugin root");
        write_valid_plugin(&plugin_dir, "test_agent");
        write_valid_plugin(&outside_dir, "outside_agent");
        let db = Database::new(&db_root).expect("create db");
        db.save_agent_plugin_directory_settings(&SaveAgentPluginDirectorySettings {
            plugin_directory: plugin_root.to_string_lossy().to_string(),
        })
        .expect("save plugin dir");

        let mut seen = HashSet::new();
        let plugin = validate_plugin_dir(&plugin_dir, false, &mut seen);
        db.upsert_agent_plugin(&plugin).expect("upsert plugin");
        db.replace_agent_rag_chunks(
            "test_agent",
            &[AgentRagChunk {
                chunk_id: "chunk-one".to_string(),
                plugin_id: "test_agent".to_string(),
                plugin_version: "1.0.0".to_string(),
                source_hash: "hash".to_string(),
                embedding_model: "pending".to_string(),
                embedding_dim: 1536,
                chunk_text: "knowledge".to_string(),
                created_at: String::new(),
            }],
        )
        .expect("insert rag chunk");

        uninstall_plugin_by_id(&db, "test_agent").expect("uninstall");
        assert!(!plugin_dir.exists());
        assert!(db
            .list_agent_rag_chunks("test_agent")
            .expect("rag chunks")
            .is_empty());
        assert!(db
            .list_agent_plugins()
            .expect("list plugins")
            .iter()
            .all(|plugin| plugin.plugin_id != "test_agent"));

        let outside_plugin = validate_plugin_dir(&outside_dir, false, &mut HashSet::new());
        db.upsert_agent_plugin(&outside_plugin)
            .expect("upsert outside plugin");
        let error = uninstall_plugin_by_id(&db, "outside_agent").expect_err("reject outside path");
        assert!(error.contains("outside the configured Agent plugin directory"));
        assert!(outside_dir.exists());
        let _ = fs::remove_dir_all(db_root);
        let _ = fs::remove_dir_all(plugin_root);
        let _ = fs::remove_dir_all(outside_dir);
    }

    #[test]
    fn refresh_scan_updates_manifest_without_reenabling_disabled_plugin() {
        let db_root = unique_test_dir("hot_refresh_db");
        let plugin_root = unique_test_dir("hot_refresh_plugins");
        let plugin_dir = plugin_root.join("hot_agent");
        fs::create_dir_all(&plugin_root).expect("create plugin root");
        write_valid_plugin(&plugin_dir, "hot_agent");
        let db = Database::new(&db_root).expect("create db");

        scan_roots_and_persist(&db, vec![(plugin_root.clone(), false)]).expect("initial scan");
        db.set_agent_plugin_enabled("hot_agent", false)
            .expect("disable plugin");
        fs::write(
            plugin_dir.join("manifest.json"),
            r#"{
  "plugin_id": "hot_agent",
  "plugin_name": "Hot Agent Reloaded",
  "plugin_version": "1.1.0",
  "author": "Test",
  "description": "A refreshed Agent",
  "tags": ["hot"],
  "create_time": "2026-04-29",
  "update_time": "2026-04-29",
  "min_app_version": "1.0.0",
  "rag_enabled": false,
  "is_multi_agent_supported": true
}"#,
        )
        .expect("rewrite manifest");

        scan_roots_and_persist(&db, vec![(plugin_root.clone(), false)]).expect("refresh scan");
        let plugin = db
            .list_agent_plugins()
            .expect("list plugins")
            .into_iter()
            .find(|plugin| plugin.plugin_id == "hot_agent")
            .expect("hot plugin");
        assert_eq!(plugin.plugin_name, "Hot Agent Reloaded");
        assert_eq!(plugin.plugin_version, "1.1.0");
        assert!(!plugin.enabled);
        let _ = fs::remove_dir_all(db_root);
        let _ = fs::remove_dir_all(plugin_root);
    }

    #[test]
    fn migrates_secretary_profile_memory_and_conversations_to_agents() {
        let db_root = unique_test_dir("secretary_migration_db");
        let db = Database::new(&db_root).expect("create db");
        let persona = db
            .save_secretary_persona(&crate::models::secretary::SaveSecretaryPersona {
                id: None,
                name: "Mira".to_string(),
                voice: "warm".to_string(),
                values: "protect attention".to_string(),
                style: "brief".to_string(),
                boundaries: "confirm writes".to_string(),
            })
            .expect("save persona");
        let profile = db
            .save_secretary_profile(&crate::models::secretary::SaveSecretaryProfile {
                id: None,
                name: "General Secretary".to_string(),
                role: "question_answer".to_string(),
                domain: "Personal productivity".to_string(),
                persona_id: Some(persona.id),
                skill_ids: Vec::new(),
            })
            .expect("save profile");
        db.save_secretary_settings(&crate::models::secretary::SaveSecretarySettings {
            base_url: None,
            model: None,
            api_key: None,
            skill_folder: None,
            conversation_folder: None,
            active_persona_id: Some(persona.id),
            active_profile_id: Some(profile.id),
        })
        .expect("save secretary settings");
        let memory = db
            .save_secretary_memory(&crate::models::secretary::SaveSecretaryMemory {
                id: None,
                content: "Walter likes local-first agents.".to_string(),
                scope: "global".to_string(),
                domain: Some("agents".to_string()),
                profile_id: Some(profile.id),
                status: Some("active".to_string()),
                pinned: Some(true),
                source_conversation_id: None,
            })
            .expect("save secretary memory");
        db.save_secretary_conversation(&crate::models::secretary::SecretaryConversation {
            id: 0,
            title: "Secretary memory chat".to_string(),
            profile_id: Some(profile.id),
            transcript_path: String::new(),
            messages: vec![
                SecretaryMessage {
                    role: "user".to_string(),
                    content: "remember this".to_string(),
                    created_at: String::new(),
                },
                SecretaryMessage {
                    role: "assistant".to_string(),
                    content: "I will keep it visible.".to_string(),
                    created_at: String::new(),
                },
            ],
            created_at: String::new(),
            updated_at: String::new(),
        })
        .expect("save conversation");

        let status = migrate_secretary_to_agents(&db).expect("migrate");
        assert_eq!(status.status, "completed");
        assert!(status.details.contains("\"memories_migrated\":1"));
        let memories = db
            .list_agent_memories(Some("secretary"))
            .expect("agent memories");
        assert!(memories
            .iter()
            .any(|item| item.memory_id == "secretary-migrated-profile"));
        assert!(memories
            .iter()
            .any(|item| item.memory_id == format!("secretary-memory-{}", memory.id)));
        let sessions = db.list_agent_sessions().expect("agent sessions");
        assert_eq!(sessions.len(), 1);
        let session = db
            .get_agent_session(&sessions[0].session_id)
            .expect("load migrated session");
        assert_eq!(session.agent_ids, vec!["secretary"]);
        assert_eq!(session.messages.len(), 2);
        assert_eq!(session.messages[0].sender_type, 1);
        assert_eq!(session.messages[1].agent_id.as_deref(), Some("secretary"));

        let second = migrate_secretary_to_agents(&db).expect("second migrate");
        assert_eq!(second.status, "completed");
        let session = db
            .get_agent_session(&sessions[0].session_id)
            .expect("load migrated session again");
        assert_eq!(session.messages.len(), 2);
        let _ = fs::remove_dir_all(db_root);
    }

    #[test]
    fn confirms_or_rejects_memory_proposals_before_prompt_inclusion() {
        let db_root = unique_test_dir("memory_proposal_db");
        let db = Database::new(&db_root).expect("create db");
        let rejected = create_memory_proposal(
            &db,
            Some("session-one".to_string()),
            Some("secretary".to_string()),
            None,
            "Do not store me".to_string(),
        )
        .expect("create rejected proposal");
        confirm_memory_proposal(
            &db,
            ConfirmAgentMemoryProposalInput {
                proposal_id: rejected.proposal_id.clone(),
                accepted: false,
                content: None,
                scope: None,
                agent_id: None,
            },
        )
        .expect("reject proposal");
        assert!(db
            .relevant_agent_memories("secretary", 10)
            .expect("memories after reject")
            .is_empty());

        let accepted = create_memory_proposal(
            &db,
            Some("session-two".to_string()),
            Some("secretary".to_string()),
            None,
            "Walter prefers explicit memory confirmation.".to_string(),
        )
        .expect("create accepted proposal");
        let status = confirm_memory_proposal(
            &db,
            ConfirmAgentMemoryProposalInput {
                proposal_id: accepted.proposal_id,
                accepted: true,
                content: Some("Walter prefers editable memory confirmation.".to_string()),
                scope: Some("agent".to_string()),
                agent_id: Some("secretary".to_string()),
            },
        )
        .expect("confirm proposal");
        assert_eq!(status.status, "accepted");
        let memories = db
            .relevant_agent_memories("secretary", 10)
            .expect("memories after accept");
        assert_eq!(memories.len(), 1);
        assert_eq!(
            memories[0].content,
            "Walter prefers editable memory confirmation."
        );
        assert_eq!(memories[0].scope, "agent");
        let _ = fs::remove_dir_all(db_root);
    }

    #[test]
    fn creates_conversation_summary_and_includes_relevant_previous_summary_in_prompt() {
        let db_root = unique_test_dir("summary_db");
        let plugin_root = unique_test_dir("summary_plugin");
        write_valid_plugin(&plugin_root, "secretary");
        let mut seen = HashSet::new();
        let plugin = validate_plugin_dir(&plugin_root, false, &mut seen);
        let db = Database::new(&db_root).expect("create db");
        for session_id in ["old-session", "current-session"] {
            db.save_agent_session(&AgentSession {
                session_id: session_id.to_string(),
                session_type: 1,
                agent_ids: vec!["secretary".to_string()],
                session_title: session_id.to_string(),
                memory_enabled: true,
                messages: Vec::new(),
                created_at: String::new(),
                updated_at: String::new(),
            })
            .expect("save session");
        }
        db.append_agent_message(&AgentMessage {
            message_id: "old-user".to_string(),
            session_id: "old-session".to_string(),
            sender_type: 1,
            agent_id: None,
            content: "We discussed launch checklist and memory controls.".to_string(),
            turn_index: 1,
            stream_status: "final".to_string(),
            error_text: String::new(),
            created_at: String::new(),
        })
        .expect("append old user");
        db.append_agent_message(&AgentMessage {
            message_id: "old-agent".to_string(),
            session_id: "old-session".to_string(),
            sender_type: 2,
            agent_id: Some("secretary".to_string()),
            content: "I suggested keeping confirmation visible.".to_string(),
            turn_index: 1,
            stream_status: "final".to_string(),
            error_text: String::new(),
            created_at: String::new(),
        })
        .expect("append old agent");
        let summary = refresh_conversation_summary(&db, "old-session").expect("summary");
        assert!(summary.summary.contains("launch checklist"));
        assert!(summary.topics.contains(&"launch".to_string()));

        let current = db
            .get_agent_session("current-session")
            .expect("current session");
        let selected = SelectedAppContext::default();
        let (prompt, used_context) = build_agent_system_prompt(
            &db,
            &plugin,
            &current,
            &selected,
            "what did we discuss before?",
        )
        .expect("prompt");
        assert!(prompt.contains("Relevant previous conversation summaries"));
        assert!(prompt.contains("launch checklist"));
        assert_eq!(
            used_context.conversation_summaries,
            vec![summary.summary_id]
        );
        let _ = fs::remove_dir_all(db_root);
        let _ = fs::remove_dir_all(plugin_root);
    }

    #[test]
    fn saves_agent_message_to_file_and_deletes_message() {
        let db_root = unique_test_dir("message_actions_db");
        let export_root = unique_test_dir("message_actions_exports");
        let db = Database::new(&db_root).expect("create db");
        db.save_app_settings(&crate::models::settings::AppSettings {
            page_size: 50,
            todo_display: "list".to_string(),
            note_display: "list".to_string(),
            note_template: String::new(),
            note_folder: export_root.to_string_lossy().to_string(),
            ..crate::models::settings::AppSettings::default()
        })
        .expect("save app settings");
        db.save_agent_session(&AgentSession {
            session_id: "message-action-session".to_string(),
            session_type: 1,
            agent_ids: vec!["secretary".to_string()],
            session_title: "Message actions".to_string(),
            memory_enabled: true,
            messages: Vec::new(),
            created_at: String::new(),
            updated_at: String::new(),
        })
        .expect("save session");
        db.append_agent_message(&AgentMessage {
            message_id: "assistant-message".to_string(),
            session_id: "message-action-session".to_string(),
            sender_type: 2,
            agent_id: Some("secretary".to_string()),
            content: "Save this answer.".to_string(),
            turn_index: 1,
            stream_status: "final".to_string(),
            error_text: String::new(),
            created_at: String::new(),
        })
        .expect("append message");

        let path = save_message_to_markdown_file(&db, "assistant-message").expect("save file");
        let content = fs::read_to_string(&path).expect("read saved file");
        assert!(content.contains("Save this answer."));
        assert!(Path::new(&path).starts_with(&export_root));

        db.delete_agent_message("assistant-message")
            .expect("delete message");
        let session = db
            .get_agent_session("message-action-session")
            .expect("load session");
        assert!(session.messages.is_empty());
        let _ = fs::remove_dir_all(db_root);
        let _ = fs::remove_dir_all(export_root);
    }

    #[test]
    fn prompt_includes_identity_memory_and_app_context() {
        let plugin_root = unique_test_dir("prompt_plugin");
        write_valid_plugin(&plugin_root, "secretary");
        let mut seen = HashSet::new();
        let plugin = validate_plugin_dir(&plugin_root, false, &mut seen);
        let db_root = unique_test_dir("prompt_db");
        let db = Database::new(&db_root).expect("create db");
        db.save_agent_user_identity(&SaveAgentUserIdentity {
            display_name: "Walter".to_string(),
            preferred_language: "zh-CN".to_string(),
            communication_style: "concise".to_string(),
            roles: vec!["engineer".to_string()],
            goals: vec!["build a soulful agents module".to_string()],
            boundaries: "Confirm before writes.".to_string(),
            important_facts: "Uses Lazy Todo App daily.".to_string(),
            enabled: true,
        })
        .expect("save identity");
        db.save_agent_memory(&SaveAgentMemory {
            memory_id: Some("memory-one".to_string()),
            content: "Walter cares about local-first memory.".to_string(),
            scope: "global".to_string(),
            agent_id: None,
            status: Some("active".to_string()),
            pinned: Some(true),
            source_session_id: None,
            source_agent_id: None,
            source_message_id: None,
        })
        .expect("save memory");
        db.add_todo(
            "Review Agents prompt context",
            "",
            1,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("add todo");
        db.insert_note(
            "Agent note",
            "This note should be visible to the Agent.",
            "yellow",
        )
        .expect("add note");
        let session = AgentSession {
            session_id: "session-one".to_string(),
            session_type: 1,
            agent_ids: vec!["secretary".to_string()],
            session_title: "Prompt test".to_string(),
            memory_enabled: true,
            messages: Vec::new(),
            created_at: String::new(),
            updated_at: String::new(),
        };
        db.save_agent_session(&session).expect("save session");
        let selected = SelectedAppContext {
            include_todos: true,
            include_milestones: false,
            include_notes: true,
            todo_ids: Vec::new(),
            milestone_indexes: Vec::new(),
            note_ids: Vec::new(),
        };

        let (prompt, used) =
            build_agent_system_prompt(&db, &plugin, &session, &selected, "what should I do?")
                .expect("build prompt");

        assert!(prompt.contains("App-owned user identity"));
        assert!(prompt.contains("Walter"));
        assert!(prompt.contains("App-owned durable memories"));
        assert!(prompt.contains("Todo context"));
        assert!(prompt.contains("Sticky Note context"));
        assert!(prompt.contains("call `web_fetch` before answering"));
        assert_eq!(used.memories, vec!["memory-one"]);
        assert_eq!(used.todos.len(), 1);
        assert_eq!(used.notes.len(), 1);
        let _ = fs::remove_dir_all(plugin_root);
        let _ = fs::remove_dir_all(db_root);
    }

    #[test]
    fn built_in_read_tools_return_app_data_and_audit() {
        let db_root = unique_test_dir("tool_read_db");
        let db = Database::new(&db_root).expect("create db");
        db.add_todo(
            "Ship tool registry",
            "with tests",
            1,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("add todo");
        let note = db
            .insert_note("Tool note", "Readable by an Agent.", "blue")
            .expect("add note");

        let todo_result = execute_builtin_tool(
            &db,
            AgentToolCallInput {
                session_id: Some("session-read".to_string()),
                agent_id: Some("secretary".to_string()),
                tool_name: "read_todo_list".to_string(),
                arguments: json!({ "include_completed": true }),
            },
        )
        .expect("read todos");
        assert_eq!(todo_result.status, "completed");
        assert_eq!(
            todo_result
                .result
                .pointer("/todos/0/title")
                .and_then(Value::as_str),
            Some("Ship tool registry")
        );

        let note_result = execute_builtin_tool(
            &db,
            AgentToolCallInput {
                session_id: Some("session-read".to_string()),
                agent_id: Some("secretary".to_string()),
                tool_name: "read_note".to_string(),
                arguments: json!({ "note_id": note.id }),
            },
        )
        .expect("read note");
        assert_eq!(
            note_result
                .result
                .pointer("/notes/0/title")
                .and_then(Value::as_str),
            Some("Tool note")
        );
        let _ = fs::remove_dir_all(db_root);
    }

    #[test]
    fn web_fetch_tool_is_read_only_and_blocks_local_targets() {
        let tool = builtin_tools()
            .into_iter()
            .find(|tool| tool.name == "web_fetch")
            .expect("web_fetch tool");
        assert_eq!(tool.safety_class, "read");
        assert_eq!(tool.permission_category, "app_context");
        assert!(!tool.requires_confirmation);

        let db_root = unique_test_dir("tool_web_fetch_db");
        let db = Database::new(&db_root).expect("create db");
        let rejected = execute_builtin_tool(
            &db,
            AgentToolCallInput {
                session_id: None,
                agent_id: Some("secretary".to_string()),
                tool_name: "web_fetch".to_string(),
                arguments: json!({ "url": "http://127.0.0.1:8080/private" }),
            },
        );
        assert!(rejected
            .expect_err("localhost rejected")
            .contains("local, private, and link-local"));

        let non_http = execute_builtin_tool(
            &db,
            AgentToolCallInput {
                session_id: None,
                agent_id: Some("secretary".to_string()),
                tool_name: "web_fetch".to_string(),
                arguments: json!({ "url": "file:///etc/passwd" }),
            },
        );
        assert!(non_http
            .expect_err("file URL rejected")
            .contains("http and https"));
        let _ = fs::remove_dir_all(db_root);
    }

    #[test]
    fn web_fetch_html_extraction_returns_readable_text() {
        let html = r#"
            <!doctype html>
            <html>
              <head>
                <title>Example &amp; Test</title>
                <style>.hidden { display: none; }</style>
              </head>
              <body>
                <h1>Hello&nbsp;world</h1>
                <script>secret()</script>
                <p>Translate &lt;this&gt; page.</p>
              </body>
            </html>
        "#;

        assert_eq!(extract_html_title(html), "Example & Test");
        let text = extract_html_text(html);
        assert!(text.contains("Hello world"));
        assert!(text.contains("Translate <this> page."));
        assert!(!text.contains("secret()"));
        assert!(!text.contains("display: none"));
    }

    #[test]
    fn extracts_web_fetch_url_from_user_message() {
        assert_eq!(
            web_fetch_url_from_user_message(
                "Translate and summarize https://martinfowler.com/articles/harness-engineering.html."
            )
            .as_deref(),
            Some("https://martinfowler.com/articles/harness-engineering.html")
        );
        assert_eq!(
            web_fetch_url_from_user_message(
                "Please read <https://example.com/page?x=1> and analyze it"
            )
            .as_deref(),
            Some("https://example.com/page?x=1")
        );
        assert!(web_fetch_url_from_user_message("No URL here").is_none());
    }

    #[test]
    fn write_note_tool_requires_confirmation_before_mutating() {
        let db_root = unique_test_dir("tool_write_note_db");
        let db = Database::new(&db_root).expect("create db");
        let note = db
            .insert_note("Before", "old content", "yellow")
            .expect("add note");

        let pending = execute_builtin_tool(
            &db,
            AgentToolCallInput {
                session_id: Some("session-write".to_string()),
                agent_id: Some("secretary".to_string()),
                tool_name: "write_note".to_string(),
                arguments: json!({
                    "note_id": note.id,
                    "title": "After",
                    "content": "new content"
                }),
            },
        )
        .expect("propose note edit");
        assert_eq!(pending.status, "pending_confirmation");
        assert!(pending.requires_confirmation);

        let unchanged = find_note(&db, note.id).expect("load unchanged note");
        assert_eq!(unchanged.title, "Before");
        let action_id = pending.action_id.expect("pending action id");
        let confirmed = confirm_pending_tool_action(
            &db,
            ConfirmAgentToolActionInput {
                action_id,
                accepted: true,
            },
        )
        .expect("confirm action");
        assert!(confirmed.accepted);
        assert_eq!(
            confirmed
                .result
                .pointer("/note/title")
                .and_then(Value::as_str),
            Some("After")
        );
        let updated = find_note(&db, note.id).expect("load updated note");
        assert_eq!(updated.content, "new content");
        let _ = fs::remove_dir_all(db_root);
    }

    #[test]
    fn todo_and_milestone_write_tools_are_confirmed_actions() {
        let db_root = unique_test_dir("tool_write_todo_milestone_db");
        let db = Database::new(&db_root).expect("create db");
        let add_todo = execute_builtin_tool(
            &db,
            AgentToolCallInput {
                session_id: None,
                agent_id: Some("secretary".to_string()),
                tool_name: "add_todo_item".to_string(),
                arguments: json!({ "title": "Confirm me", "priority": 1 }),
            },
        )
        .expect("propose todo add");
        confirm_pending_tool_action(
            &db,
            ConfirmAgentToolActionInput {
                action_id: add_todo.action_id.expect("todo action id"),
                accepted: true,
            },
        )
        .expect("confirm todo add");
        assert!(db
            .list_todos()
            .expect("todos")
            .iter()
            .any(|todo| todo.title == "Confirm me"));

        let mut settings = db.get_pomodoro_settings().expect("settings");
        settings.milestones = vec![crate::models::pomodoro::PomodoroMilestone {
            name: "Alpha".to_string(),
            deadline: "2026-05-01".to_string(),
            status: "active".to_string(),
        }];
        db.save_pomodoro_settings(&settings)
            .expect("save milestone");
        let milestone = execute_builtin_tool(
            &db,
            AgentToolCallInput {
                session_id: None,
                agent_id: Some("secretary".to_string()),
                tool_name: "change_milestone".to_string(),
                arguments: json!({ "index": 0, "status": "completed" }),
            },
        )
        .expect("propose milestone change");
        confirm_pending_tool_action(
            &db,
            ConfirmAgentToolActionInput {
                action_id: milestone.action_id.expect("milestone action id"),
                accepted: true,
            },
        )
        .expect("confirm milestone");
        assert_eq!(
            db.get_pomodoro_settings().expect("settings").milestones[0].status,
            "completed"
        );
        let _ = fs::remove_dir_all(db_root);
    }

    #[test]
    fn file_read_tool_enforces_safe_roots_and_text_limits() {
        let db_root = unique_test_dir("tool_file_read_db");
        let safe_root = unique_test_dir("tool_file_safe_root");
        let outside_root = unique_test_dir("tool_file_outside_root");
        fs::create_dir_all(&safe_root).expect("safe root");
        fs::create_dir_all(&outside_root).expect("outside root");
        let allowed_file = safe_root.join("allowed.txt");
        let outside_file = outside_root.join("outside.txt");
        let binary_file = safe_root.join("binary.bin");
        fs::write(&allowed_file, "hello from safe root").expect("write allowed");
        fs::write(&outside_file, "outside").expect("write outside");
        fs::write(&binary_file, b"abc\0def").expect("write binary");
        let db = Database::new(&db_root).expect("create db");
        db.save_agent_safe_file_root_settings(&SaveAgentSafeFileRootSettings {
            safe_file_roots: normalize_safe_roots(vec![safe_root.to_string_lossy().to_string()])
                .expect("normalize roots"),
        })
        .expect("save safe roots");

        let result = execute_builtin_tool(
            &db,
            AgentToolCallInput {
                session_id: None,
                agent_id: Some("secretary".to_string()),
                tool_name: "read_file".to_string(),
                arguments: json!({ "path": allowed_file.to_string_lossy() }),
            },
        )
        .expect("read safe file");
        assert_eq!(
            result.result.pointer("/content").and_then(Value::as_str),
            Some("hello from safe root")
        );

        let outside = execute_builtin_tool(
            &db,
            AgentToolCallInput {
                session_id: None,
                agent_id: Some("secretary".to_string()),
                tool_name: "read_file".to_string(),
                arguments: json!({ "path": outside_file.to_string_lossy() }),
            },
        );
        assert!(outside
            .expect_err("outside rejected")
            .contains("outside configured Agent safe roots"));

        let binary = execute_builtin_tool(
            &db,
            AgentToolCallInput {
                session_id: None,
                agent_id: Some("secretary".to_string()),
                tool_name: "read_file".to_string(),
                arguments: json!({ "path": binary_file.to_string_lossy() }),
            },
        );
        assert!(binary.expect_err("binary rejected").contains("binary"));
        let _ = fs::remove_dir_all(db_root);
        let _ = fs::remove_dir_all(safe_root);
        let _ = fs::remove_dir_all(outside_root);
    }

    #[test]
    fn write_file_tool_requires_confirmation_and_safe_root() {
        let db_root = unique_test_dir("tool_file_write_db");
        let safe_root = unique_test_dir("tool_file_write_safe_root");
        let outside_root = unique_test_dir("tool_file_write_outside_root");
        fs::create_dir_all(&safe_root).expect("safe root");
        fs::create_dir_all(&outside_root).expect("outside root");
        let target = safe_root.join("result.txt");
        let outside = outside_root.join("outside.txt");
        let db = Database::new(&db_root).expect("create db");
        db.save_agent_safe_file_root_settings(&SaveAgentSafeFileRootSettings {
            safe_file_roots: normalize_safe_roots(vec![safe_root.to_string_lossy().to_string()])
                .expect("normalize roots"),
        })
        .expect("save safe roots");

        let pending = execute_builtin_tool(
            &db,
            AgentToolCallInput {
                session_id: Some("session-file-write".to_string()),
                agent_id: Some("secretary".to_string()),
                tool_name: "write_file".to_string(),
                arguments: json!({
                    "path": target.to_string_lossy(),
                    "content": "created after confirmation\n"
                }),
            },
        )
        .expect("propose write file");
        assert_eq!(pending.status, "pending_confirmation");
        assert!(!target.exists());
        confirm_pending_tool_action(
            &db,
            ConfirmAgentToolActionInput {
                action_id: pending.action_id.expect("action id"),
                accepted: true,
            },
        )
        .expect("confirm write file");
        assert_eq!(
            fs::read_to_string(&target).expect("read written file"),
            "created after confirmation\n"
        );

        let rejected = execute_builtin_tool(
            &db,
            AgentToolCallInput {
                session_id: None,
                agent_id: Some("secretary".to_string()),
                tool_name: "write_file".to_string(),
                arguments: json!({
                    "path": outside.to_string_lossy(),
                    "content": "nope"
                }),
            },
        );
        assert!(rejected
            .expect_err("outside write rejected")
            .contains("outside configured Agent safe roots"));
        let _ = fs::remove_dir_all(db_root);
        let _ = fs::remove_dir_all(safe_root);
        let _ = fs::remove_dir_all(outside_root);
    }
}
