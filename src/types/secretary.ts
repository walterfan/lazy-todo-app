import type { StickyNote } from "./note";

export interface SecretarySettings {
  base_url: string;
  model: string;
  has_saved_api_key: boolean;
  skill_folder: string;
  conversation_folder: string;
  active_persona_id: number | null;
  active_profile_id: number | null;
}

export interface MaskedSecretarySettings {
  saved: SecretarySettings;
  effective_base_url: string;
  effective_model: string;
  has_api_key: boolean;
  base_url_from_env: boolean;
  model_from_env: boolean;
  api_key_from_env: boolean;
}

export interface SecretaryPersona {
  id: number;
  name: string;
  voice: string;
  values: string;
  style: string;
  boundaries: string;
  created_at: string;
  updated_at: string;
}

export interface SaveSecretaryPersona {
  id?: number | null;
  name: string;
  voice: string;
  values: string;
  style: string;
  boundaries: string;
}

export interface SecretaryProfile {
  id: number;
  name: string;
  role: string;
  domain: string;
  persona_id: number | null;
  skill_ids: number[];
  created_at: string;
  updated_at: string;
}

export interface SaveSecretaryProfile {
  id?: number | null;
  name: string;
  role: string;
  domain: string;
  persona_id: number | null;
  skill_ids: number[];
}

export interface SecretarySkill {
  id: number;
  name: string;
  summary: string;
  path: string;
  content: string;
  updated_at: string;
}

export interface SecretaryMemory {
  id: number;
  content: string;
  scope: string;
  domain: string;
  profile_id: number | null;
  status: string;
  pinned: boolean;
  source_conversation_id: number | null;
  created_at: string;
  updated_at: string;
}

export interface SaveSecretaryMemory {
  id?: number | null;
  content: string;
  scope: string;
  domain?: string | null;
  profile_id?: number | null;
  status?: string | null;
  pinned?: boolean | null;
  source_conversation_id?: number | null;
}

export interface SecretaryReminder {
  id: number;
  title: string;
  notes: string;
  due_at: string;
  status: string;
  source_conversation_id: number | null;
  created_at: string;
  updated_at: string;
}

export interface SaveSecretaryReminder {
  id?: number | null;
  title: string;
  notes?: string | null;
  due_at: string;
  status?: string | null;
  source_conversation_id?: number | null;
}

export interface SecretaryMessage {
  role: string;
  content: string;
  created_at: string;
}

export interface SecretaryConversation {
  id: number;
  title: string;
  profile_id: number | null;
  transcript_path: string;
  messages: SecretaryMessage[];
  created_at: string;
  updated_at: string;
}

export interface TodoContext {
  id: number;
  title: string;
  description: string;
  priority: number;
  completed: boolean;
  deadline: string | null;
  created_at: string;
}

export interface MilestoneContext {
  index: number;
  name: string;
  deadline: string;
  status: string;
}

export interface NoteContext {
  id: number;
  title: string;
  content: string;
  color: string;
  created_at: string;
  updated_at: string;
}

export interface SelectedAppContext {
  include_todos: boolean;
  include_milestones: boolean;
  include_notes: boolean;
  todo_ids: number[];
  milestone_indexes: number[];
  note_ids: number[];
}

export interface UsedContextMetadata {
  todos: number[];
  milestones: number[];
  notes: number[];
  memories: number[];
}

export interface SecretaryAppContext {
  todos: TodoContext[];
  milestones: MilestoneContext[];
  notes: NoteContext[];
}

export interface ProposedNoteEdit {
  note_id: number;
  title: string | null;
  content: string | null;
  color: string | null;
  before_title: string;
  before_content: string;
  before_color: string;
}

export interface ConfirmNoteEditResult {
  accepted: boolean;
  note: StickyNote | null;
}

export interface SendSecretaryMessageResult {
  conversation: SecretaryConversation;
  assistant_message: SecretaryMessage;
  used_context: UsedContextMetadata;
  proposed_memory: string | null;
  proposed_reminder: SaveSecretaryReminder | null;
  proposed_note_edit: ProposedNoteEdit | null;
}

export interface SecretaryStreamEvent {
  stream_id: string;
  content?: string;
  error?: string;
  result?: SendSecretaryMessageResult;
}

export const DEFAULT_SELECTED_CONTEXT: SelectedAppContext = {
  include_todos: true,
  include_milestones: true,
  include_notes: false,
  todo_ids: [],
  milestone_indexes: [],
  note_ids: [],
};
