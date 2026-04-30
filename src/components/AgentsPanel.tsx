import { useState, type MouseEvent } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { AgentsController } from "../hooks/useAgents";
import type { AgentMessage } from "../types/agents";
import type { Translator } from "../i18n";

interface AgentsPanelProps {
  agents: AgentsController;
  onRecordMessageToNote?: (content: string, title: string) => Promise<void>;
  t: Translator;
}

type MessageActionStatus = {
  messageId: string;
  text: string;
  kind: "success" | "error";
};

export function AgentsPanel({ agents, onRecordMessageToNote, t }: AgentsPanelProps) {
  const [message, setMessage] = useState("");
  const [exportText, setExportText] = useState("");
  const [memoryDrafts, setMemoryDrafts] = useState<Record<string, string>>({});
  const [messageActionStatus, setMessageActionStatus] = useState<MessageActionStatus | null>(null);
  const [sessionActionStatus, setSessionActionStatus] = useState("");
  const [confirmingDeleteSessionId, setConfirmingDeleteSessionId] = useState<string | null>(null);
  const [deletingSessionId, setDeletingSessionId] = useState<string | null>(null);
  const selectedAgents = agents.selectedAgents;
  const isGroupChat = selectedAgents.length > 1;
  const selectedAgentNames = selectedAgents.map((agent) => agent.plugin_name).join(", ");

  if (agents.loading) {
    return <div className="loading">{t("agentsLoading")}</div>;
  }

  const send = () => {
    if (!message.trim()) return;
    const text = message;
    setMessage("");
    void agents.sendMessage(text);
  };

  const actionError = (error: unknown) => error instanceof Error ? error.message : String(error);

  const copyMessage = async (item: AgentMessage) => {
    try {
      await navigator.clipboard.writeText(item.content);
      setMessageActionStatus({ messageId: item.message_id, text: t("copied"), kind: "success" });
    } catch (error) {
      setMessageActionStatus({ messageId: item.message_id, text: t("copyFailed", { message: actionError(error) }), kind: "error" });
    }
  };

  const saveMessage = async (messageId: string) => {
    try {
      const path = await agents.saveMessageToFile(messageId);
      setMessageActionStatus({ messageId, text: t("savedTo", { path }), kind: "success" });
    } catch (error) {
      setMessageActionStatus({ messageId, text: t("saveFailed", { message: actionError(error) }), kind: "error" });
    }
  };

  const recordMessage = async (item: AgentMessage) => {
    const title = `Agent reply - ${item.agent_id ?? "agent"}`;
    try {
      if (onRecordMessageToNote) {
        await onRecordMessageToNote(item.content, title);
      } else {
        await agents.recordMessageToNote(item.content, title);
      }
      setMessageActionStatus({ messageId: item.message_id, text: t("recordedIntoNotes"), kind: "success" });
    } catch (error) {
      setMessageActionStatus({ messageId: item.message_id, text: t("recordFailed", { message: actionError(error) }), kind: "error" });
    }
  };

  const deleteMessage = async (messageId: string) => {
    try {
      await agents.deleteMessage(messageId);
      setMessageActionStatus({ messageId, text: t("messageDeleted"), kind: "success" });
    } catch (error) {
      setMessageActionStatus({ messageId, text: t("deleteFailed", { message: actionError(error) }), kind: "error" });
    }
  };

  const resetSession = async (sessionId: string) => {
    if (!window.confirm(t("resetSessionConfirm"))) return;
    try {
      await agents.resetSession(sessionId);
      setExportText("");
      setSessionActionStatus(t("sessionReset"));
    } catch (error) {
      setSessionActionStatus(t("resetFailed", { message: actionError(error) }));
    }
  };

  const requestDeleteSession = (event: MouseEvent<HTMLButtonElement>, sessionId: string) => {
    event.preventDefault();
    event.stopPropagation();
    setConfirmingDeleteSessionId(sessionId);
    setSessionActionStatus(t("deleteSessionConfirm"));
  };

  const cancelDeleteSession = (event: MouseEvent<HTMLButtonElement>) => {
    event.preventDefault();
    event.stopPropagation();
    setConfirmingDeleteSessionId(null);
    setSessionActionStatus("");
  };

  const deleteSession = async (event: MouseEvent<HTMLButtonElement>, sessionId: string) => {
    event.preventDefault();
    event.stopPropagation();
    setDeletingSessionId(sessionId);
    try {
      await agents.deleteSession(sessionId);
      setExportText("");
      setConfirmingDeleteSessionId(null);
      setSessionActionStatus(t("sessionDeleted"));
    } catch (error) {
      setSessionActionStatus(t("deleteSessionFailed", { message: actionError(error) }));
    } finally {
      setDeletingSessionId(null);
    }
  };

  return (
    <div className="secretary-shell">
      {agents.error && <div className="secretary-error">{agents.error}</div>}

      <section className="secretary-main">
        <div className="secretary-identity">
          {selectedAgents.length > 0 && (
            <div className="agent-avatar-stack" aria-hidden="true">
              {selectedAgents.slice(0, 4).map((agent) => (
                <img
                  className="agent-avatar"
                  src={convertFileSrc(agent.avatar_path)}
                  alt=""
                  key={agent.plugin_id}
                />
              ))}
            </div>
          )}
          <div>
            <div className="secretary-name">{isGroupChat ? t("groupChat") : agents.selectedAgent?.plugin_name ?? t("agents")}</div>
            <div className="secretary-subtitle">
              {isGroupChat ? selectedAgentNames : agents.selectedAgent?.description ?? t("selectAgentPrompt")}
            </div>
            {!isGroupChat && agents.selectedAgent && (
              <div className="agent-meta">
                {agents.selectedAgent.author} · v{agents.selectedAgent.plugin_version}
              </div>
            )}
            {isGroupChat && (
              <div className="agent-meta">{t("selectedAgents", { count: selectedAgents.length })}</div>
            )}
          </div>
          <div className="secretary-status">
            {!isGroupChat && agents.selectedAgent?.tags.slice(0, 4).map((tag) => <span key={tag}>{tag}</span>)}
            {isGroupChat && selectedAgents.slice(0, 4).map((agent) => <span key={agent.plugin_id}>@{agent.plugin_name}</span>)}
            {agents.ragStatus && <span>{agents.ragStatus.indexed_chunks} RAG chunks</span>}
            <span>{agents.memories.filter((memory) => memory.status === "active").length} memories</span>
          </div>
        </div>

        <div className="secretary-transcript">
          {agents.session?.messages.length ? (
            agents.session.messages.map((item) => (
              <div
                className={`secretary-message ${item.sender_type === 2 ? "secretary-message-assistant" : "secretary-message-user"}`}
                key={item.message_id}
              >
                <div className="secretary-message-meta">
                  {item.sender_type === 2 ? item.agent_id ?? "agent" : "user"} · {item.created_at}
                </div>
                <div className="secretary-message-body">{item.content}</div>
                {item.sender_type === 2 && (
                  <div className="agent-message-actions" aria-label="Agent message actions">
                    <button type="button" title={t("copyToClipboard")} aria-label={t("copyToClipboard")} onClick={() => void copyMessage(item)}>
                      ⧉
                    </button>
                    <button type="button" title={t("saveIntoFile")} aria-label={t("saveIntoFile")} onClick={() => void saveMessage(item.message_id)}>
                      ⇩
                    </button>
                    <button type="button" title={t("recordIntoNote")} aria-label={t("recordIntoNote")} onClick={() => void recordMessage(item)}>
                      ◫
                    </button>
                    <button type="button" title={t("deleteMessage")} aria-label={t("deleteMessage")} onClick={() => void deleteMessage(item.message_id)}>
                      ×
                    </button>
                  </div>
                )}
                {messageActionStatus?.messageId === item.message_id && (
                  <div className={`agent-message-action-status ${messageActionStatus.kind}`}>
                    {messageActionStatus.text}
                  </div>
                )}
              </div>
            ))
          ) : (
            !agents.pendingUserMessage && !agents.streamingMessage && (
              <div className="secretary-empty">
                {t("agentEmpty")}
              </div>
            )
          )}

          {agents.pendingUserMessage && (
            <div className="secretary-message secretary-message-user secretary-message-pending">
              <div className="secretary-message-meta">{t("userSending")}</div>
              <div className="secretary-message-body">{agents.pendingUserMessage}</div>
            </div>
          )}

          {Object.entries(agents.streamingMessages).length > 0 ? (
            Object.entries(agents.streamingMessages).map(([agentId, content]) => (
              <div className="secretary-message secretary-message-assistant secretary-message-streaming" key={agentId}>
                <div className="secretary-message-meta">{agentName(agents.agents, agentId)} · streaming</div>
                <div className="secretary-message-body">{content}</div>
              </div>
            ))
          ) : agents.streamingMessage && (
            <div className="secretary-message secretary-message-assistant secretary-message-streaming">
              <div className="secretary-message-meta">{agents.selectedAgent?.plugin_name ?? "agent"} · streaming</div>
              <div className="secretary-message-body">{agents.streamingMessage}</div>
            </div>
          )}
        </div>

        {agents.pendingToolActions.map((action) => (
          <div className="secretary-proposal" key={action.action_id}>
            <div className="secretary-note-edit-header">
              <strong>{toolActionTitle(action.tool_name, action.preview)}</strong>
              <span>{action.status}</span>
            </div>
            {isExternalCliPreview(action.preview) ? (
              <div className="agent-cli-preview">
                <div><span>Command</span><code>{commandPreview(action.preview)}</code></div>
                <div><span>Safety</span><code>{action.preview.safety_class}</code></div>
                <div><span>Working dir</span><code>{action.preview.working_directory || "app default"}</code></div>
                <div><span>Timeout</span><code>{action.preview.timeout_ms} ms</code></div>
              </div>
            ) : (
              <pre className="agent-tool-preview">{formatPreview(action.preview)}</pre>
            )}
            <div className="secretary-actions">
              <button onClick={() => void agents.confirmToolAction(action.action_id, false)}>
                Reject
              </button>
              <button onClick={() => void agents.confirmToolAction(action.action_id, true)}>
                Confirm
              </button>
            </div>
          </div>
        ))}

        {agents.memoryProposals.map((proposal) => {
          const draft = memoryDrafts[proposal.proposal_id] ?? proposal.proposed_text;
          return (
            <div className="secretary-proposal" key={proposal.proposal_id}>
              <div className="secretary-note-edit-header">
                <strong>Proposed memory</strong>
                <span>{proposal.status}</span>
              </div>
              <textarea
                className="agent-memory-proposal-text"
                value={draft}
                onChange={(event) =>
                  setMemoryDrafts((current) => ({
                    ...current,
                    [proposal.proposal_id]: event.target.value,
                  }))
                }
              />
              <div className="agent-meta">
                {proposal.source_agent_id ?? "agent"} · {proposal.source_session_id ?? "no session"}
              </div>
              <div className="secretary-actions">
                <button
                  onClick={() =>
                    void agents.confirmMemoryProposal({
                      proposal_id: proposal.proposal_id,
                      accepted: false,
                    })
                  }
                >
                  Reject
                </button>
                <button
                  onClick={() =>
                    void agents.confirmMemoryProposal({
                      proposal_id: proposal.proposal_id,
                      accepted: true,
                      content: draft,
                      scope: "global",
                      agent_id: agents.selectedAgentId,
                    })
                  }
                >
                  Remember
                </button>
              </div>
            </div>
          );
        })}

        <div className="secretary-composer">
          {selectedAgents.length > 1 && (
            <div className="agent-mention-row">
              {selectedAgents.map((agent) => (
                <button
                  type="button"
                  key={agent.plugin_id}
                  onClick={() => setMessage((current) => `${current}${current.endsWith(" ") || !current ? "" : " "}@${agent.plugin_name} `)}
                >
                  @{agent.plugin_name}
                </button>
              ))}
            </div>
          )}
          <textarea
            value={message}
            onChange={(event) => setMessage(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter" && !event.shiftKey) {
                event.preventDefault();
                send();
              }
            }}
            placeholder={selectedAgents.length > 1 ? t("messageGroup") : agents.selectedAgent ? t("messageAgent", { name: agents.selectedAgent.plugin_name }) : t("selectAgentFirst")}
            disabled={selectedAgents.length === 0 || agents.sending}
          />
          <div className="secretary-composer-actions">
            <button onClick={() => void agents.startSession()} disabled={selectedAgents.length === 0 || agents.sending}>
              {t("newChat")}
            </button>
            <button onClick={send} disabled={!message.trim() || selectedAgents.length === 0 || agents.sending}>
              {agents.sending ? t("sending") : t("send")}
            </button>
          </div>
        </div>
      </section>

      <aside className="secretary-side">
        <div className="secretary-panel">
          <h3>{t("agents")}</h3>
          <div className="secretary-list">
            {agents.agents.map((agent) => (
              <button
                key={agent.plugin_id}
                className={`secretary-list-item agent-select-item ${agents.selectedAgentIds.includes(agent.plugin_id) ? "active" : ""}`}
                onClick={() => agents.toggleAgent(agent.plugin_id)}
                disabled={!agent.enabled}
                title={agent.validation_diagnostics.map((item) => item.message).join("\n")}
              >
                <strong><span className="agent-select-check">{agents.selectedAgentIds.includes(agent.plugin_id) ? "✓" : ""}</span>{agent.plugin_name}</strong>
                <span>{agent.plugin_version} · {agent.bundled ? "built-in" : "local"}</span>
              </button>
            ))}
          </div>
        </div>

        <div className="secretary-panel">
          <h3>{t("knowledge")}</h3>
          <div className="secretary-list-item">
            <strong>{agents.ragStatus?.message ?? t("noAgentSelected")}</strong>
            {agents.ragStatus?.stale && <span>{agents.ragStatus.stale_reasons.join(", ")}</span>}
          </div>
          <div className="secretary-actions">
            <button onClick={() => void agents.rebuildRag()} disabled={!agents.selectedAgent?.has_rag_knowledge}>
              Rebuild
            </button>
          </div>
        </div>

        <div className="secretary-panel">
          <h3>{t("tools")}</h3>
          <div className="secretary-list">
            {agents.builtinTools.slice(0, 8).map((tool) => (
              <div className="secretary-list-item" key={tool.name}>
                <strong>{tool.name}</strong>
                <span>{tool.requires_confirmation ? "confirm" : "read"}</span>
              </div>
            ))}
          </div>
        </div>

        <div className="secretary-panel">
          <h3>{t("memory")}</h3>
          <div className="secretary-list">
            {agents.identity?.enabled && agents.identity.display_name && (
              <div className="secretary-list-item">
                <strong>{agents.identity.display_name}</strong>
                <span>{agents.identity.communication_style || agents.identity.preferred_language || "Identity enabled"}</span>
              </div>
            )}
            {agents.memories.slice(0, 4).map((memory) => (
              <div className="secretary-list-item" key={memory.memory_id}>
                <strong>{memory.pinned ? "Pinned memory" : "Memory"}</strong>
                <span>{memory.content}</span>
              </div>
            ))}
            {agents.memories.length === 0 && <div className="secretary-list-item">{t("noMemories")}</div>}
          </div>
        </div>

        {agents.lastUsedContext && (
          <div className="secretary-panel">
            <h3>{t("usedContext")}</h3>
            <div className="secretary-list-item">
              <strong>
                {agents.lastUsedContext.todos.length} todos · {agents.lastUsedContext.milestones.length} milestones ·{" "}
                {agents.lastUsedContext.notes.length} notes
              </strong>
              <span>
                {agents.lastUsedContext.memories.length} memories · {agents.lastUsedContext.rag_chunks.length} RAG ·{" "}
                {(agents.lastUsedContext.conversation_summaries ?? []).length} summaries ·{" "}
                {agents.lastUsedContext.previous_messages.length} previous messages
              </span>
            </div>
          </div>
        )}

        <div className="secretary-panel">
          <h3>{t("sessions")}</h3>
          <div className="secretary-actions">
            <button
              onClick={async () => setExportText(await agents.exportTranscript())}
              disabled={!agents.session}
            >
              {t("export")}
            </button>
          </div>
          {sessionActionStatus && <div className="agent-session-status">{sessionActionStatus}</div>}
          {exportText && <pre className="agent-export-preview">{exportText}</pre>}
          <div className="secretary-list">
            {agents.sessions.slice(0, 12).map((session) => (
              <div
                key={session.session_id}
                className={`agent-session-row ${agents.session?.session_id === session.session_id ? "active" : ""}`}
              >
                <button
                  type="button"
                  className="agent-session-main"
                  onClick={() => void agents.loadSession(session.session_id)}
                  title={t("loadSession")}
                >
                  <strong>{session.session_title || "Agent chat"}</strong>
                  <span>{session.updated_at}</span>
                </button>
                <div className="agent-session-actions">
                  <button
                    type="button"
                    title={t("resetSession")}
                    aria-label={t("resetSession")}
                    onClick={() => void resetSession(session.session_id)}
                  >
                    ↺
                  </button>
                  {confirmingDeleteSessionId === session.session_id ? (
                    <>
                      <button
                        type="button"
                        className="agent-session-cancel-delete"
                        title={t("cancel")}
                        aria-label={t("cancel")}
                        onClick={cancelDeleteSession}
                        disabled={deletingSessionId === session.session_id}
                      >
                        {t("cancel")}
                      </button>
                      <button
                        type="button"
                        className="agent-session-delete-confirm"
                        title={t("deleteSession")}
                        aria-label={t("deleteSession")}
                        onClick={(event) => void deleteSession(event, session.session_id)}
                        disabled={deletingSessionId === session.session_id}
                      >
                        ×
                      </button>
                    </>
                  ) : (
                    <button
                      type="button"
                      title={t("deleteSession")}
                      aria-label={t("deleteSession")}
                      onClick={(event) => requestDeleteSession(event, session.session_id)}
                    >
                      ×
                    </button>
                  )}
                </div>
              </div>
            ))}
            {agents.sessions.length === 0 && <div className="secretary-list-item">{t("noSessions")}</div>}
          </div>
        </div>
      </aside>
    </div>
  );
}

function formatPreview(value: unknown): string {
  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return String(value);
  }
}

interface ExternalCliPreview {
  kind: "external_cli";
  tool_id: string;
  display_name: string;
  executable: string;
  argv: string[];
  working_directory: string;
  safety_class: string;
  timeout_ms: number;
}

function isExternalCliPreview(value: unknown): value is ExternalCliPreview {
  return Boolean(
    value &&
      typeof value === "object" &&
      (value as { kind?: unknown }).kind === "external_cli" &&
      Array.isArray((value as { argv?: unknown }).argv)
  );
}

function commandPreview(preview: ExternalCliPreview): string {
  return [preview.executable, ...preview.argv].join(" ");
}

function agentName(agents: { plugin_id: string; plugin_name: string }[], agentId: string): string {
  return agents.find((agent) => agent.plugin_id === agentId)?.plugin_name ?? agentId;
}

function toolActionTitle(toolName: string, preview: unknown): string {
  if (isExternalCliPreview(preview)) {
    return `${preview.display_name || preview.tool_id} external CLI`;
  }
  return toolName;
}
