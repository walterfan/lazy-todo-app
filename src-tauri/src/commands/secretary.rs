use std::fs;
use std::path::{Path, PathBuf};

use chrono::Local;
use serde::Serialize;
use serde_json::json;
use tauri::{AppHandle, Emitter, State};

use crate::db::Database;
use crate::models::secretary::{
    ConfirmNoteEditInput, ConfirmNoteEditResult, EffectiveLlmSettings, MaskedSecretarySettings,
    MilestoneContext, NoteContext, ProposedNoteEdit, SaveSecretaryMemory, SaveSecretaryPersona,
    SaveSecretaryProfile, SaveSecretaryReminder, SaveSecretarySettings, SecretaryAppContext,
    SecretaryConversation, SecretaryMemory, SecretaryMessage, SecretaryPersona, SecretaryProfile,
    SecretaryReminder, SecretarySkill, SelectedAppContext,
    SendSecretaryMessageInput, SendSecretaryMessageResult, SkillScanResult, TodoContext,
    UsedContextMetadata,
};

const MAX_SKILL_BYTES: u64 = 128 * 1024;
const MAX_NOTE_CHARS: usize = 1200;

#[derive(Clone, Serialize)]
struct SecretaryStreamChunk {
    stream_id: String,
    content: String,
}

#[derive(Clone, Serialize)]
struct SecretaryStreamError {
    stream_id: String,
    error: String,
}

#[derive(Clone, Serialize)]
struct SecretaryStreamFinish {
    stream_id: String,
    result: SendSecretaryMessageResult,
}

#[tauri::command]
pub fn get_secretary_settings(db: State<'_, Database>) -> Result<MaskedSecretarySettings, String> {
    let saved = db.get_secretary_settings().map_err(|e| e.to_string())?;
    let effective = resolve_effective_llm_settings(&db)?;
    Ok(MaskedSecretarySettings {
        saved,
        effective_base_url: effective.base_url,
        effective_model: effective.model,
        has_api_key: effective.has_api_key,
        base_url_from_env: effective.base_url_from_env,
        model_from_env: effective.model_from_env,
        api_key_from_env: effective.api_key_from_env,
    })
}

#[tauri::command]
pub fn save_secretary_settings(
    db: State<'_, Database>,
    input: SaveSecretarySettings,
) -> Result<MaskedSecretarySettings, String> {
    db.save_secretary_settings(&input).map_err(|e| e.to_string())?;
    get_secretary_settings(db)
}

#[tauri::command]
pub fn validate_secretary_config(db: State<'_, Database>) -> Result<(), String> {
    let effective = resolve_effective_llm_settings(&db)?;
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

#[tauri::command]
pub fn list_secretary_personas(db: State<'_, Database>) -> Result<Vec<SecretaryPersona>, String> {
    db.list_secretary_personas().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_secretary_persona(
    db: State<'_, Database>,
    input: SaveSecretaryPersona,
) -> Result<SecretaryPersona, String> {
    db.save_secretary_persona(&input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_secretary_persona(db: State<'_, Database>, id: i64) -> Result<(), String> {
    db.delete_secretary_persona(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_secretary_profiles(db: State<'_, Database>) -> Result<Vec<SecretaryProfile>, String> {
    db.list_secretary_profiles().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_secretary_profile(
    db: State<'_, Database>,
    input: SaveSecretaryProfile,
) -> Result<SecretaryProfile, String> {
    db.save_secretary_profile(&input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_secretary_profile(db: State<'_, Database>, id: i64) -> Result<(), String> {
    db.delete_secretary_profile(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_secretary_skills(db: State<'_, Database>) -> Result<Vec<SecretarySkill>, String> {
    db.list_secretary_skills().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn refresh_secretary_skills(db: State<'_, Database>) -> Result<SkillScanResult, String> {
    let settings = db.get_secretary_settings().map_err(|e| e.to_string())?;
    let folder = settings.skill_folder.trim();
    if folder.is_empty() {
        return Ok(SkillScanResult {
            skills: db.list_secretary_skills().map_err(|e| e.to_string())?,
            skipped: vec!["No skill folder configured.".to_string()],
        });
    }

    let mut scanned = Vec::new();
    let mut skipped = Vec::new();
    let dir = PathBuf::from(folder);
    let entries = fs::read_dir(&dir).map_err(|e| format!("Cannot read skill folder: {e}"))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path.extension().and_then(|v| v.to_str()).unwrap_or("").to_lowercase();
        if !matches!(ext.as_str(), "md" | "txt" | "skill") {
            skipped.push(format!("{}: unsupported extension", path.display()));
            continue;
        }
        let meta = match entry.metadata() {
            Ok(meta) => meta,
            Err(e) => {
                skipped.push(format!("{}: {e}", path.display()));
                continue;
            }
        };
        if meta.len() > MAX_SKILL_BYTES {
            skipped.push(format!("{}: file is larger than 128KB", path.display()));
            continue;
        }
        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(e) => {
                skipped.push(format!("{}: {e}", path.display()));
                continue;
            }
        };
        let (name, summary) = skill_name_summary(&path, &content);
        scanned.push(SecretarySkill {
            id: 0,
            name,
            summary,
            path: path.to_string_lossy().to_string(),
            content,
            updated_at: String::new(),
        });
    }
    let skills = db.replace_secretary_skills(&scanned).map_err(|e| e.to_string())?;
    Ok(SkillScanResult { skills, skipped })
}

#[tauri::command]
pub fn list_secretary_memories(db: State<'_, Database>) -> Result<Vec<SecretaryMemory>, String> {
    db.list_secretary_memories().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_secretary_memory(
    db: State<'_, Database>,
    input: SaveSecretaryMemory,
) -> Result<SecretaryMemory, String> {
    db.save_secretary_memory(&input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_secretary_memory(db: State<'_, Database>, id: i64) -> Result<(), String> {
    db.delete_secretary_memory(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_secretary_reminders(db: State<'_, Database>) -> Result<Vec<SecretaryReminder>, String> {
    db.list_secretary_reminders().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn due_secretary_reminders(db: State<'_, Database>) -> Result<Vec<SecretaryReminder>, String> {
    db.due_secretary_reminders().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_secretary_reminder(
    db: State<'_, Database>,
    input: SaveSecretaryReminder,
) -> Result<SecretaryReminder, String> {
    db.save_secretary_reminder(&input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_secretary_reminder(db: State<'_, Database>, id: i64) -> Result<(), String> {
    db.delete_secretary_reminder(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_secretary_app_context(db: State<'_, Database>) -> Result<SecretaryAppContext, String> {
    build_app_context(&db, &SelectedAppContext {
        include_todos: true,
        include_milestones: true,
        include_notes: true,
        ..Default::default()
    })
}

#[tauri::command]
pub fn confirm_secretary_note_edit(
    db: State<'_, Database>,
    input: ConfirmNoteEditInput,
) -> Result<ConfirmNoteEditResult, String> {
    if !input.accepted {
        if let Some(id) = input.conversation_id {
            append_conversation_event(&db, id, "system", "Secretary note edit rejected.")?;
        }
        return Ok(ConfirmNoteEditResult { accepted: false, note: None });
    }

    let note = db
        .update_note(
            input.edit.note_id,
            input.edit.title.as_deref(),
            input.edit.content.as_deref(),
            input.edit.color.as_deref(),
        )
        .map_err(|e| e.to_string())?;
    if let Some(id) = input.conversation_id {
        append_conversation_event(
            &db,
            id,
            "system",
            &format!("Secretary note edit accepted for note #{}.", input.edit.note_id),
        )?;
    }
    Ok(ConfirmNoteEditResult { accepted: true, note: Some(note) })
}

#[tauri::command]
pub fn start_secretary_conversation(
    db: State<'_, Database>,
    profile_id: Option<i64>,
) -> Result<SecretaryConversation, String> {
    let now = now_string();
    let title = format!("Secretary conversation {}", now);
    let conversation = SecretaryConversation {
        id: 0,
        title,
        profile_id,
        transcript_path: String::new(),
        messages: Vec::new(),
        created_at: now.clone(),
        updated_at: now,
    };
    db.save_secretary_conversation(&conversation).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn send_secretary_message(
    db: State<'_, Database>,
    input: SendSecretaryMessageInput,
) -> Result<SendSecretaryMessageResult, String> {
    validate_secretary_config(db.clone())?;
    let effective = resolve_effective_llm_settings(&db)?;
    let profile_id = input.profile_id.or_else(|| db.get_secretary_settings().ok().and_then(|s| s.active_profile_id));
    let profile = match profile_id {
        Some(id) => Some(db.get_secretary_profile(id).map_err(|e| e.to_string())?),
        None => None,
    };
    let persona = match profile.as_ref().and_then(|p| p.persona_id) {
        Some(id) => db.get_secretary_persona(id).ok(),
        None => None,
    };
    let memories = db
        .relevant_secretary_memories(profile_id, profile.as_ref().map(|p| p.domain.as_str()).unwrap_or(""), 6)
        .map_err(|e| e.to_string())?;
    let app_context = build_app_context(&db, &input.selected_context)?;
    let used_context = UsedContextMetadata {
        todos: app_context.todos.iter().map(|t| t.id).collect(),
        milestones: app_context.milestones.iter().map(|m| m.index).collect(),
        notes: app_context.notes.iter().map(|n| n.id).collect(),
        memories: memories.iter().map(|m| m.id).collect(),
    };

    let mut conversation = match input.conversation_id {
        Some(id) => db.get_secretary_conversation(id).map_err(|e| e.to_string())?,
        None => start_secretary_conversation(db.clone(), profile_id)?,
    };
    let user_message = SecretaryMessage {
        role: "user".to_string(),
        content: input.message.trim().to_string(),
        created_at: now_string(),
    };
    conversation.messages.push(user_message.clone());

    let system_prompt = build_system_prompt(persona.as_ref(), profile.as_ref(), &memories, &app_context);
    let assistant_content = call_llm(&effective, &system_prompt, &conversation.messages).await?;
    let assistant_message = SecretaryMessage {
        role: "assistant".to_string(),
        content: assistant_content.clone(),
        created_at: now_string(),
    };
    conversation.messages.push(assistant_message.clone());
    conversation.updated_at = now_string();
    if conversation.title.starts_with("Secretary conversation ") && !input.message.trim().is_empty() {
        conversation.title = input.message.trim().chars().take(48).collect();
    }
    conversation = db.save_secretary_conversation(&conversation).map_err(|e| e.to_string())?;

    let proposed_note_edit = propose_note_edit(&input.message, &assistant_content, &app_context.notes);
    let conversation_id = conversation.id;
    Ok(SendSecretaryMessageResult {
        conversation,
        assistant_message,
        used_context,
        proposed_memory: propose_memory(&input.message),
        proposed_reminder: propose_reminder(&input.message, conversation_id),
        proposed_note_edit,
    })
}

#[tauri::command]
pub async fn send_secretary_message_stream(
    app: AppHandle,
    db: State<'_, Database>,
    input: SendSecretaryMessageInput,
) -> Result<SendSecretaryMessageResult, String> {
    let stream_id = input
        .stream_id
        .clone()
        .unwrap_or_else(|| format!("secretary-{}", Local::now().timestamp_millis()));
    let result = send_secretary_message_stream_inner(&app, db, input, &stream_id).await;
    match result {
        Ok(result) => {
            let _ = app.emit("secretary-stream-finish", SecretaryStreamFinish {
                stream_id,
                result: result.clone(),
            });
            Ok(result)
        }
        Err(error) => {
            let _ = app.emit("secretary-stream-error", SecretaryStreamError {
                stream_id,
                error: error.clone(),
            });
            Err(error)
        }
    }
}

async fn send_secretary_message_stream_inner(
    app: &AppHandle,
    db: State<'_, Database>,
    input: SendSecretaryMessageInput,
    stream_id: &str,
) -> Result<SendSecretaryMessageResult, String> {
    validate_secretary_config(db.clone())?;
    let effective = resolve_effective_llm_settings(&db)?;
    let profile_id = input.profile_id.or_else(|| db.get_secretary_settings().ok().and_then(|s| s.active_profile_id));
    let profile = match profile_id {
        Some(id) => Some(db.get_secretary_profile(id).map_err(|e| e.to_string())?),
        None => None,
    };
    let persona = match profile.as_ref().and_then(|p| p.persona_id) {
        Some(id) => db.get_secretary_persona(id).ok(),
        None => None,
    };
    let memories = db
        .relevant_secretary_memories(profile_id, profile.as_ref().map(|p| p.domain.as_str()).unwrap_or(""), 6)
        .map_err(|e| e.to_string())?;
    let app_context = build_app_context(&db, &input.selected_context)?;
    let used_context = UsedContextMetadata {
        todos: app_context.todos.iter().map(|t| t.id).collect(),
        milestones: app_context.milestones.iter().map(|m| m.index).collect(),
        notes: app_context.notes.iter().map(|n| n.id).collect(),
        memories: memories.iter().map(|m| m.id).collect(),
    };

    let mut conversation = match input.conversation_id {
        Some(id) => db.get_secretary_conversation(id).map_err(|e| e.to_string())?,
        None => start_secretary_conversation(db.clone(), profile_id)?,
    };
    let user_message = SecretaryMessage {
        role: "user".to_string(),
        content: input.message.trim().to_string(),
        created_at: now_string(),
    };
    conversation.messages.push(user_message);

    let system_prompt = build_system_prompt(persona.as_ref(), profile.as_ref(), &memories, &app_context);
    let assistant_content = call_llm_stream(&effective, &system_prompt, &conversation.messages, app, stream_id).await?;
    let assistant_message = SecretaryMessage {
        role: "assistant".to_string(),
        content: assistant_content.clone(),
        created_at: now_string(),
    };
    conversation.messages.push(assistant_message.clone());
    conversation.updated_at = now_string();
    if conversation.title.starts_with("Secretary conversation ") && !input.message.trim().is_empty() {
        conversation.title = input.message.trim().chars().take(48).collect();
    }
    conversation = db.save_secretary_conversation(&conversation).map_err(|e| e.to_string())?;
    let proposed_note_edit = propose_note_edit(&input.message, &assistant_content, &app_context.notes);
    let conversation_id = conversation.id;
    Ok(SendSecretaryMessageResult {
        conversation,
        assistant_message,
        used_context,
        proposed_memory: propose_memory(&input.message),
        proposed_reminder: propose_reminder(&input.message, conversation_id),
        proposed_note_edit,
    })
}

#[tauri::command]
pub fn list_secretary_conversations(db: State<'_, Database>) -> Result<Vec<SecretaryConversation>, String> {
    db.list_secretary_conversations().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn load_secretary_conversation(db: State<'_, Database>, id: i64) -> Result<SecretaryConversation, String> {
    db.get_secretary_conversation(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_secretary_transcript(
    db: State<'_, Database>,
    id: i64,
) -> Result<SecretaryConversation, String> {
    let settings = db.get_secretary_settings().map_err(|e| e.to_string())?;
    let folder = settings.conversation_folder.trim();
    if folder.is_empty() {
        return Err("Conversation folder is not configured.".to_string());
    }
    let mut conversation = db.get_secretary_conversation(id).map_err(|e| e.to_string())?;
    fs::create_dir_all(folder).map_err(|e| e.to_string())?;
    let file_base = sanitize_filename(&format!("{}-{}", conversation.id, conversation.title));
    let md_path = Path::new(folder).join(format!("{file_base}.md"));
    let json_path = Path::new(folder).join(format!("{file_base}.json"));
    fs::write(&md_path, conversation_to_markdown(&conversation)).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(&conversation).map_err(|e| e.to_string())?;
    fs::write(json_path, json).map_err(|e| e.to_string())?;
    conversation.transcript_path = md_path.to_string_lossy().to_string();
    db.save_secretary_conversation(&conversation).map_err(|e| e.to_string())
}

fn resolve_effective_llm_settings(db: &Database) -> Result<EffectiveLlmSettings, String> {
    let saved = db.get_secretary_settings().map_err(|e| e.to_string())?;
    let saved_api_key = db.get_secretary_saved_api_key().unwrap_or_default();
    let env_base_url = std::env::var("LLM_BASE_URL").unwrap_or_default();
    let env_model = std::env::var("LLM_MODEL").unwrap_or_default();
    let env_api_key = std::env::var("LLM_API_KEY").unwrap_or_default();
    let base_url_from_env = !env_base_url.trim().is_empty();
    let model_from_env = !env_model.trim().is_empty();
    let api_key_from_env = !env_api_key.trim().is_empty();
    let api_key = if api_key_from_env { env_api_key } else { saved_api_key };
    Ok(EffectiveLlmSettings {
        base_url: if base_url_from_env { env_base_url } else { saved.base_url },
        model: if model_from_env { env_model } else { saved.model },
        has_api_key: !api_key.trim().is_empty(),
        api_key,
        base_url_from_env,
        model_from_env,
        api_key_from_env,
    })
}

fn skill_name_summary(path: &Path, content: &str) -> (String, String) {
    let fallback = path.file_stem().and_then(|v| v.to_str()).unwrap_or("Skill").to_string();
    let mut name = fallback;
    let mut summary = String::new();
    for line in content.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if line.starts_with('#') && name == path.file_stem().and_then(|v| v.to_str()).unwrap_or("Skill") {
            name = line.trim_start_matches('#').trim().to_string();
            continue;
        }
        if summary.is_empty() && !line.starts_with("---") {
            summary = line.chars().take(160).collect();
            break;
        }
    }
    (name, summary)
}

fn build_app_context(db: &Database, selected: &SelectedAppContext) -> Result<SecretaryAppContext, String> {
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
            .filter(|(index, _)| selected.milestone_indexes.is_empty() || selected.milestone_indexes.contains(index))
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
                    note.content = note.content.chars().take(MAX_NOTE_CHARS).collect::<String>() + "...";
                }
                NoteContext::from(note)
            })
            .collect()
    } else {
        Vec::new()
    };
    Ok(SecretaryAppContext { todos, milestones, notes })
}

fn build_system_prompt(
    persona: Option<&SecretaryPersona>,
    profile: Option<&SecretaryProfile>,
    memories: &[SecretaryMemory],
    context: &SecretaryAppContext,
) -> String {
    let mut sections = vec![
        "You are a personal secretary inside Lazy Todo App. Be practical, discreet, and concise.".to_string(),
        "You may suggest next steps, ask clarifying questions, propose memories, propose reminders, and propose note edits. Never claim that you changed data unless the user confirmed an app action.".to_string(),
    ];
    if let Some(persona) = persona {
        sections.push(format!(
            "Persona: {}\nVoice: {}\nValues: {}\nStyle: {}\nBoundaries: {}",
            persona.name, persona.voice, persona.values, persona.style, persona.boundaries
        ));
    }
    if let Some(profile) = profile {
        sections.push(format!(
            "Role: {}\nDomain: {}\nRole guidance: {}",
            profile.role,
            profile.domain,
            role_guidance(&profile.role)
        ));
    }
    if !memories.is_empty() {
        let text = memories
            .iter()
            .map(|m| format!("- [{}] {}", m.scope, m.content))
            .collect::<Vec<_>>()
            .join("\n");
        sections.push(format!("Relevant local memories:\n{text}"));
    }
    if !context.todos.is_empty() {
        let text = context.todos.iter().map(|todo| {
            format!(
                "- #{} [{}] P{} {}{}: {}",
                todo.id,
                if todo.completed { "done" } else { "active" },
                todo.priority,
                todo.deadline.as_deref().unwrap_or("no deadline"),
                if todo.description.is_empty() { "" } else { " " },
                todo.title
            )
        }).collect::<Vec<_>>().join("\n");
        sections.push(format!("Todo context:\n{text}"));
    }
    if !context.milestones.is_empty() {
        let text = context.milestones.iter().map(|m| {
            format!("- #{} [{}] {} due {}", m.index, m.status, m.name, m.deadline)
        }).collect::<Vec<_>>().join("\n");
        sections.push(format!("Milestone context:\n{text}"));
    }
    if !context.notes.is_empty() {
        let text = context.notes.iter().map(|note| {
            format!("- Note #{} \"{}\" ({})\n{}", note.id, note.title, note.color, note.content)
        }).collect::<Vec<_>>().join("\n\n");
        sections.push(format!("Sticky Note context:\n{text}"));
    }
    sections.join("\n\n")
}

fn role_guidance(role: &str) -> &'static str {
    match role {
        "question_asker" => "Ask focused questions in the selected domain to help the user think.",
        "idea_critic" => "Critique ideas honestly and constructively in the selected domain.",
        "idea_raiser" => "Raise useful ideas and options in the selected domain.",
        _ => "Answer questions and give grounded help in the selected domain.",
    }
}

async fn call_llm(
    settings: &EffectiveLlmSettings,
    system_prompt: &str,
    messages: &[SecretaryMessage],
) -> Result<String, String> {
    let base_url = settings.base_url.trim().trim_end_matches('/');
    let url = if base_url.ends_with("/chat/completions") {
        base_url.to_string()
    } else {
        format!("{base_url}/chat/completions")
    };
    let mut payload_messages = vec![json!({"role": "system", "content": system_prompt})];
    for message in messages.iter().rev().take(12).collect::<Vec<_>>().into_iter().rev() {
        let role = if message.role == "assistant" { "assistant" } else { "user" };
        payload_messages.push(json!({"role": role, "content": message.content}));
    }
    let payload = json!({
        "model": settings.model,
        "messages": payload_messages,
    });
    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .bearer_auth(&settings.api_key)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("LLM request failed: {e}"))?;
    let status = response.status();
    let body: serde_json::Value = response.json().await.map_err(|e| format!("Invalid LLM response: {e}"))?;
    if !status.is_success() {
        return Err(format!("LLM request failed with status {status}: {body}"));
    }
    body.pointer("/choices/0/message/content")
        .and_then(|v| v.as_str())
        .map(|v| v.to_string())
        .ok_or_else(|| "LLM response did not contain choices[0].message.content".to_string())
}

async fn call_llm_stream(
    settings: &EffectiveLlmSettings,
    system_prompt: &str,
    messages: &[SecretaryMessage],
    app: &AppHandle,
    stream_id: &str,
) -> Result<String, String> {
    let base_url = settings.base_url.trim().trim_end_matches('/');
    let url = if base_url.ends_with("/chat/completions") {
        base_url.to_string()
    } else {
        format!("{base_url}/chat/completions")
    };
    let mut payload_messages = vec![json!({"role": "system", "content": system_prompt})];
    for message in messages.iter().rev().take(12).collect::<Vec<_>>().into_iter().rev() {
        let role = if message.role == "assistant" { "assistant" } else { "user" };
        payload_messages.push(json!({"role": role, "content": message.content}));
    }
    let payload = json!({
        "model": settings.model,
        "messages": payload_messages,
        "stream": true,
    });
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
        return Err(format!("LLM stream request failed with status {status}: {body}"));
    }

    let mut buffer = String::new();
    let mut content = String::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| format!("LLM stream read failed: {e}"))?
    {
        buffer.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(index) = buffer.find('\n') {
            let line = buffer[..index].trim().to_string();
            buffer = buffer[index + 1..].to_string();
            if let Some(delta) = parse_sse_delta(&line)? {
                content.push_str(&delta);
                let _ = app.emit("secretary-stream-chunk", SecretaryStreamChunk {
                    stream_id: stream_id.to_string(),
                    content: delta,
                });
            }
        }
    }
    let trailing = buffer.trim().to_string();
    if let Some(delta) = parse_sse_delta(&trailing)? {
        content.push_str(&delta);
        let _ = app.emit("secretary-stream-chunk", SecretaryStreamChunk {
            stream_id: stream_id.to_string(),
            content: delta,
        });
    }
    Ok(content)
}

fn parse_sse_delta(line: &str) -> Result<Option<String>, String> {
    if line.is_empty() || line.starts_with(':') {
        return Ok(None);
    }
    let Some(data) = line.strip_prefix("data:") else {
        return Ok(None);
    };
    let data = data.trim();
    if data == "[DONE]" {
        return Ok(None);
    }
    let value: serde_json::Value = serde_json::from_str(data).map_err(|e| format!("Invalid stream chunk: {e}"))?;
    Ok(value
        .pointer("/choices/0/delta/content")
        .or_else(|| value.pointer("/choices/0/message/content"))
        .and_then(|v| v.as_str())
        .map(|v| v.to_string()))
}

fn propose_memory(message: &str) -> Option<String> {
    let lower = message.to_lowercase();
    if lower.contains("remember ") || lower.contains("note that ") {
        Some(message.trim().to_string())
    } else {
        None
    }
}

fn propose_reminder(message: &str, conversation_id: i64) -> Option<SaveSecretaryReminder> {
    let lower = message.to_lowercase();
    if !(lower.contains("remind") || lower.contains("follow up")) {
        return None;
    }
    Some(SaveSecretaryReminder {
        id: None,
        title: message.trim().chars().take(80).collect(),
        notes: Some("Proposed by Secretary from conversation.".to_string()),
        due_at: Local::now().format("%Y-%m-%d 09:00:00").to_string(),
        status: Some("proposed".to_string()),
        source_conversation_id: Some(conversation_id),
    })
}

fn propose_note_edit(message: &str, assistant: &str, notes: &[NoteContext]) -> Option<ProposedNoteEdit> {
    let lower = message.to_lowercase();
    if notes.len() != 1 || !(lower.contains("note") && (lower.contains("update") || lower.contains("rewrite") || lower.contains("improve") || lower.contains("summarize"))) {
        return None;
    }
    let note = &notes[0];
    Some(ProposedNoteEdit {
        note_id: note.id,
        title: None,
        content: Some(assistant.to_string()),
        color: None,
        before_title: note.title.clone(),
        before_content: note.content.clone(),
        before_color: note.color.clone(),
    })
}

fn append_conversation_event(db: &Database, id: i64, role: &str, content: &str) -> Result<(), String> {
    let mut conversation = db.get_secretary_conversation(id).map_err(|e| e.to_string())?;
    conversation.messages.push(SecretaryMessage {
        role: role.to_string(),
        content: content.to_string(),
        created_at: now_string(),
    });
    db.save_secretary_conversation(&conversation).map_err(|e| e.to_string())?;
    Ok(())
}

fn conversation_to_markdown(conversation: &SecretaryConversation) -> String {
    let mut out = format!("# {}\n\n", conversation.title);
    for message in &conversation.messages {
        out.push_str(&format!("## {} · {}\n\n{}\n\n", message.role, message.created_at, message.content));
    }
    out
}

fn sanitize_filename(input: &str) -> String {
    let cleaned = input
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect::<String>();
    cleaned.trim_matches('-').chars().take(80).collect()
}

fn now_string() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}
