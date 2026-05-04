export interface AgentValidationDiagnostic {
  severity: string;
  field: string | null;
  message: string;
}

export interface AgentDefinition {
  agent_id: string;
  agent_name: string;
  agent_version: string;
  author: string;
  description: string;
  tags: string[];
  path: string;
  avatar_path: string;
  readme_path: string;
  bundled: boolean;
  enabled: boolean;
  lifecycle_state: string;
  rag_enabled: boolean;
  is_multi_agent_supported: boolean;
  has_rag_knowledge: boolean;
  validation_diagnostics: AgentValidationDiagnostic[];
}

export interface AgentManifest {
  agent_id: string;
  agent_name: string;
  agent_version: string;
  author: string;
  description: string;
  tags: string[];
  create_time: string;
  update_time: string;
  min_app_version: string;
  rag_enabled: boolean;
  is_multi_agent_supported: boolean;
}

export interface AgentConfig {
  temperature: number;
  top_p: number;
  rag_top_k: number;
  embedding_dim: number;
  response_style: string;
  ban_topics: string[];
}

export interface AgentDefinitionDetail {
  agent: AgentDefinition;
  manifest: AgentManifest | null;
  config: AgentConfig | null;
  system_prompt: string;
  readme: string;
}

export interface InstallAgentZipInput {
  zip_path: string;
}

export interface AgentDirectorySettings {
  agent_directory: string;
}

export interface SaveAgentDirectorySettings {
  agent_directory: string;
}

export interface AgentSafeFileRootSettings {
  safe_file_roots: string[];
}

export interface SaveAgentSafeFileRootSettings {
  safe_file_roots: string[];
}

export interface AgentMessage {
  message_id: string;
  session_id: string;
  sender_type: number;
  agent_id: string | null;
  content: string;
  turn_index: number;
  stream_status: string;
  error_text: string;
  created_at: string;
}

export interface AgentSession {
  session_id: string;
  session_type: number;
  agent_ids: string[];
  session_title: string;
  memory_enabled: boolean;
  messages: AgentMessage[];
  created_at: string;
  updated_at: string;
}

export interface SendAgentMessageResult {
  session: AgentSession;
  assistant_message: AgentMessage;
  used_context: AgentUsedContext;
}

export interface AgentUserIdentity {
  display_name: string;
  preferred_language: string;
  communication_style: string;
  roles: string[];
  goals: string[];
  boundaries: string;
  important_facts: string;
  enabled: boolean;
  updated_at: string;
}

export interface SaveAgentUserIdentity {
  display_name: string;
  preferred_language: string;
  communication_style: string;
  roles: string[];
  goals: string[];
  boundaries: string;
  important_facts: string;
  enabled: boolean;
}

export interface AgentMemory {
  memory_id: string;
  content: string;
  scope: string;
  agent_id: string | null;
  status: string;
  pinned: boolean;
  source_session_id: string | null;
  source_agent_id: string | null;
  source_message_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface AgentMemoryProposal {
  proposal_id: string;
  source_session_id: string | null;
  source_agent_id: string | null;
  source_message_id: string | null;
  proposed_text: string;
  status: string;
  created_at: string;
  updated_at: string;
}

export interface SaveAgentMemory {
  memory_id?: string | null;
  content: string;
  scope: string;
  agent_id?: string | null;
  status?: string | null;
  pinned?: boolean | null;
  source_session_id?: string | null;
  source_agent_id?: string | null;
  source_message_id?: string | null;
}

export interface ConfirmAgentMemoryProposalInput {
  proposal_id: string;
  accepted: boolean;
  content?: string | null;
  scope?: string | null;
  agent_id?: string | null;
}

export interface AgentUsedContext {
  todos: number[];
  milestones: number[];
  notes: number[];
  memories: string[];
  rag_chunks: string[];
  conversation_summaries: string[];
  previous_messages: string[];
}

export interface AgentStreamEvent {
  stream_id: string;
  agent_id?: string | null;
  content?: string;
  error?: string;
  result?: SendAgentMessageResult;
}

export interface AgentRagStatus {
  agent_id: string;
  rag_enabled: boolean;
  has_rag_knowledge: boolean;
  indexed_chunks: number;
  source_hash: string | null;
  current_source_hash: string | null;
  stale: boolean;
  stale_reasons: string[];
  vector_search_available: boolean;
  message: string;
}

export interface AgentMigrationStatus {
  migration_id: string;
  status: string;
  details: string;
  created_at: string;
  updated_at: string;
}

export interface AgentBuiltinTool {
  name: string;
  description: string;
  permission_category: string;
  safety_class: string;
  requires_confirmation: boolean;
  argument_schema: unknown;
  result_schema: unknown;
}

export interface AgentExternalCliTool {
  tool_id: string;
  display_name: string;
  executable: string;
  allowed_subcommands: string[];
  argument_schema: unknown;
  working_directory: string;
  environment_allowlist: string[];
  timeout_ms: number;
  output_limit: number;
  safety_class: string;
  enabled: boolean;
  available: boolean;
  availability_error: string;
  created_at: string;
  updated_at: string;
}

export interface SaveAgentExternalCliTool {
  tool_id?: string | null;
  display_name: string;
  executable: string;
  allowed_subcommands: string[];
  argument_schema: unknown;
  working_directory: string;
  environment_allowlist: string[];
  timeout_ms: number;
  output_limit: number;
  safety_class: string;
  enabled: boolean;
}

export interface AgentExternalCliToolTestResult {
  executable: string;
  resolved_path: string;
  available: boolean;
  message: string;
}

export interface AgentToolAction {
  action_id: string;
  session_id: string | null;
  agent_id: string | null;
  tool_name: string;
  arguments: unknown;
  preview: unknown;
  status: string;
  created_at: string;
  updated_at: string;
}

export interface ConfirmAgentToolActionResult {
  action_id: string;
  accepted: boolean;
  tool_name: string;
  status: string;
  result: unknown;
}

export type { SelectedAppContext };
import type { SelectedAppContext } from "./secretary";
