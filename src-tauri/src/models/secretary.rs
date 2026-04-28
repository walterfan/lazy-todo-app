use serde::{Deserialize, Serialize};

use crate::models::note::StickyNote;
use crate::models::pomodoro::PomodoroMilestone;
use crate::models::todo::Todo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretarySettings {
    pub base_url: String,
    pub model: String,
    #[serde(default)]
    pub has_saved_api_key: bool,
    #[serde(default)]
    pub skill_folder: String,
    #[serde(default)]
    pub conversation_folder: String,
    pub active_persona_id: Option<i64>,
    pub active_profile_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSecretarySettings {
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub skill_folder: Option<String>,
    pub conversation_folder: Option<String>,
    pub active_persona_id: Option<i64>,
    pub active_profile_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectiveLlmSettings {
    pub base_url: String,
    pub model: String,
    #[serde(skip_serializing)]
    pub api_key: String,
    pub base_url_from_env: bool,
    pub model_from_env: bool,
    pub api_key_from_env: bool,
    pub has_api_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskedSecretarySettings {
    pub saved: SecretarySettings,
    pub effective_base_url: String,
    pub effective_model: String,
    pub has_api_key: bool,
    pub base_url_from_env: bool,
    pub model_from_env: bool,
    pub api_key_from_env: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretaryPersona {
    pub id: i64,
    pub name: String,
    pub voice: String,
    pub values: String,
    pub style: String,
    pub boundaries: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSecretaryPersona {
    pub id: Option<i64>,
    pub name: String,
    pub voice: String,
    pub values: String,
    pub style: String,
    pub boundaries: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretaryProfile {
    pub id: i64,
    pub name: String,
    pub role: String,
    pub domain: String,
    pub persona_id: Option<i64>,
    pub skill_ids: Vec<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSecretaryProfile {
    pub id: Option<i64>,
    pub name: String,
    pub role: String,
    pub domain: String,
    pub persona_id: Option<i64>,
    #[serde(default)]
    pub skill_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretarySkill {
    pub id: i64,
    pub name: String,
    pub summary: String,
    pub path: String,
    pub content: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillScanResult {
    pub skills: Vec<SecretarySkill>,
    pub skipped: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretaryMemory {
    pub id: i64,
    pub content: String,
    pub scope: String,
    pub domain: String,
    pub profile_id: Option<i64>,
    pub status: String,
    pub pinned: bool,
    pub source_conversation_id: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSecretaryMemory {
    pub id: Option<i64>,
    pub content: String,
    pub scope: String,
    pub domain: Option<String>,
    pub profile_id: Option<i64>,
    pub status: Option<String>,
    pub pinned: Option<bool>,
    pub source_conversation_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretaryReminder {
    pub id: i64,
    pub title: String,
    pub notes: String,
    pub due_at: String,
    pub status: String,
    pub source_conversation_id: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSecretaryReminder {
    pub id: Option<i64>,
    pub title: String,
    pub notes: Option<String>,
    pub due_at: String,
    pub status: Option<String>,
    pub source_conversation_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretaryMessage {
    pub role: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretaryConversation {
    pub id: i64,
    pub title: String,
    pub profile_id: Option<i64>,
    pub transcript_path: String,
    pub messages: Vec<SecretaryMessage>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoContext {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub priority: i32,
    pub completed: bool,
    pub deadline: Option<String>,
    pub created_at: String,
}

impl From<Todo> for TodoContext {
    fn from(todo: Todo) -> Self {
        Self {
            id: todo.id,
            title: todo.title,
            description: todo.description,
            priority: todo.priority,
            completed: todo.completed,
            deadline: todo.deadline,
            created_at: todo.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneContext {
    pub index: usize,
    pub name: String,
    pub deadline: String,
    pub status: String,
}

impl MilestoneContext {
    pub fn from_milestone(index: usize, milestone: PomodoroMilestone) -> Self {
        Self {
            index,
            name: milestone.name,
            deadline: milestone.deadline,
            status: milestone.status,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteContext {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub color: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<StickyNote> for NoteContext {
    fn from(note: StickyNote) -> Self {
        Self {
            id: note.id,
            title: note.title,
            content: note.content,
            color: note.color,
            created_at: note.created_at,
            updated_at: note.updated_at,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SelectedAppContext {
    #[serde(default)]
    pub include_todos: bool,
    #[serde(default)]
    pub include_milestones: bool,
    #[serde(default)]
    pub include_notes: bool,
    #[serde(default)]
    pub todo_ids: Vec<i64>,
    #[serde(default)]
    pub milestone_indexes: Vec<usize>,
    #[serde(default)]
    pub note_ids: Vec<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsedContextMetadata {
    #[serde(default)]
    pub todos: Vec<i64>,
    #[serde(default)]
    pub milestones: Vec<usize>,
    #[serde(default)]
    pub notes: Vec<i64>,
    #[serde(default)]
    pub memories: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretaryAppContext {
    pub todos: Vec<TodoContext>,
    pub milestones: Vec<MilestoneContext>,
    pub notes: Vec<NoteContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedNoteEdit {
    pub note_id: i64,
    pub title: Option<String>,
    pub content: Option<String>,
    pub color: Option<String>,
    pub before_title: String,
    pub before_content: String,
    pub before_color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmNoteEditInput {
    pub conversation_id: Option<i64>,
    pub edit: ProposedNoteEdit,
    pub accepted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmNoteEditResult {
    pub accepted: bool,
    pub note: Option<StickyNote>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendSecretaryMessageInput {
    pub conversation_id: Option<i64>,
    pub profile_id: Option<i64>,
    pub message: String,
    pub selected_context: SelectedAppContext,
    pub stream_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendSecretaryMessageResult {
    pub conversation: SecretaryConversation,
    pub assistant_message: SecretaryMessage,
    pub used_context: UsedContextMetadata,
    pub proposed_memory: Option<String>,
    pub proposed_reminder: Option<SaveSecretaryReminder>,
    pub proposed_note_edit: Option<ProposedNoteEdit>,
}
