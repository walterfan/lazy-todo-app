import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import type {
  AgentBuiltinTool,
  AgentExternalCliTool,
  AgentExternalCliToolTestResult,
  AgentMemory,
  AgentMemoryProposal,
  AgentMigrationStatus,
  AgentPlugin,
  AgentPluginDetail,
  AgentPluginDirectorySettings,
  AgentRagStatus,
  AgentSafeFileRootSettings,
  AgentSession,
  AgentStreamEvent,
  AgentToolAction,
  AgentUsedContext,
  AgentUserIdentity,
  ConfirmAgentToolActionResult,
  ConfirmAgentMemoryProposalInput,
  SaveAgentMemory,
  SaveAgentExternalCliTool,
  SaveAgentPluginDirectorySettings,
  SaveAgentSafeFileRootSettings,
  SaveAgentUserIdentity,
  SendAgentMessageResult,
} from "../types/agents";
import { DEFAULT_SELECTED_CONTEXT } from "../types/secretary";

export function useAgents() {
  const [agents, setAgents] = useState<AgentPlugin[]>([]);
  const [sessions, setSessions] = useState<AgentSession[]>([]);
  const [session, setSession] = useState<AgentSession | null>(null);
  const [identity, setIdentity] = useState<AgentUserIdentity | null>(null);
  const [memories, setMemories] = useState<AgentMemory[]>([]);
  const [memoryProposals, setMemoryProposals] = useState<AgentMemoryProposal[]>([]);
  const [pluginDirectorySettings, setPluginDirectorySettings] = useState<AgentPluginDirectorySettings | null>(null);
  const [safeFileRootSettings, setSafeFileRootSettings] = useState<AgentSafeFileRootSettings | null>(null);
  const [builtinTools, setBuiltinTools] = useState<AgentBuiltinTool[]>([]);
  const [externalCliTools, setExternalCliTools] = useState<AgentExternalCliTool[]>([]);
  const [pendingToolActions, setPendingToolActions] = useState<AgentToolAction[]>([]);
  const [migrationStatus, setMigrationStatus] = useState<AgentMigrationStatus | null>(null);
  const [lastUsedContext, setLastUsedContext] = useState<AgentUsedContext | null>(null);
  const [selectedAgentId, setSelectedAgentId] = useState<string | null>(null);
  const [selectedAgentIds, setSelectedAgentIds] = useState<string[]>([]);
  const [ragStatus, setRagStatus] = useState<AgentRagStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [sending, setSending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [pendingUserMessage, setPendingUserMessage] = useState<string | null>(null);
  const [streamingMessage, setStreamingMessage] = useState("");
  const [streamingMessages, setStreamingMessages] = useState<Record<string, string>>({});
  const activeStreamId = useRef<string | null>(null);

  const selectedAgents = useMemo(() => {
    const enabled = agents.filter((agent) => agent.enabled);
    const ids = selectedAgentIds.length > 0
      ? selectedAgentIds
      : selectedAgentId
        ? [selectedAgentId]
        : [];
    const picked = ids
      .map((id) => enabled.find((agent) => agent.plugin_id === id))
      .filter((agent): agent is AgentPlugin => Boolean(agent));
    if (picked.length > 0) return picked;
    return enabled.slice(0, 1);
  }, [agents, selectedAgentId, selectedAgentIds]);

  const selectedAgent = useMemo(
    () => selectedAgents[0] ?? agents.find((agent) => agent.enabled) ?? null,
    [agents, selectedAgents]
  );

  const refresh = useCallback(async () => {
    setError(null);
    try {
      const [nextAgents, nextSessions, nextTools, nextExternalCliTools, nextActions, nextMemoryProposals, nextPluginDirectory, nextSafeFileRoots, nextMigrationStatus] = await Promise.all([
        invoke<AgentPlugin[]>("list_agents"),
        invoke<AgentSession[]>("list_agent_sessions"),
        invoke<AgentBuiltinTool[]>("list_agent_builtin_tools"),
        invoke<AgentExternalCliTool[]>("list_agent_external_cli_tools"),
        invoke<AgentToolAction[]>("list_pending_agent_tool_actions"),
        invoke<AgentMemoryProposal[]>("list_agent_memory_proposals"),
        invoke<AgentPluginDirectorySettings>("get_agent_plugin_directory_settings"),
        invoke<AgentSafeFileRootSettings>("get_agent_safe_file_root_settings"),
        invoke<AgentMigrationStatus>("get_agent_migration_status"),
      ]);
      setAgents(nextAgents);
      setSessions(nextSessions);
      setBuiltinTools(nextTools);
      setExternalCliTools(nextExternalCliTools);
      setPendingToolActions(nextActions);
      setMemoryProposals(nextMemoryProposals);
      setPluginDirectorySettings(nextPluginDirectory);
      setSafeFileRootSettings(nextSafeFileRoots);
      setMigrationStatus(nextMigrationStatus);
      const firstEnabled = nextAgents.find((agent) => agent.enabled)?.plugin_id ?? null;
      setSelectedAgentId((current) => current ?? firstEnabled);
      setSelectedAgentIds((current) => current.length > 0 ? current : firstEnabled ? [firstEnabled] : []);
      if (!session && nextSessions.length > 0) {
        setSession(nextSessions[0]);
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, [session]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  useEffect(() => {
    if (!selectedAgent) {
      setRagStatus(null);
      return;
    }
    invoke<AgentRagStatus>("get_agent_rag_status", { pluginId: selectedAgent.plugin_id })
      .then(setRagStatus)
      .catch(() => setRagStatus(null));
    invoke<AgentMemory[]>("list_agent_memories", { agentId: selectedAgent.plugin_id })
      .then(setMemories)
      .catch(() => setMemories([]));
  }, [selectedAgent]);

  useEffect(() => {
    invoke<AgentUserIdentity>("get_agent_user_identity")
      .then(setIdentity)
      .catch(() => setIdentity(null));
  }, []);

  useEffect(() => {
    const unlistenChunk = listen<AgentStreamEvent>("agent-stream-chunk", (event) => {
      if (event.payload.stream_id !== activeStreamId.current) return;
      const content = event.payload.content ?? "";
      const agentId = event.payload.agent_id ?? "agent";
      setStreamingMessage((current) => current + content);
      setStreamingMessages((current) => ({
        ...current,
        [agentId]: (current[agentId] ?? "") + content,
      }));
    });
    const unlistenError = listen<AgentStreamEvent>("agent-stream-error", (event) => {
      if (event.payload.stream_id !== activeStreamId.current) return;
      setError(event.payload.error ?? "Agent stream failed.");
      setPendingUserMessage(null);
      setStreamingMessage("");
      setStreamingMessages({});
      setSending(false);
      activeStreamId.current = null;
    });
    const unlistenFinish = listen<AgentStreamEvent>("agent-stream-finish", (event) => {
      if (event.payload.stream_id !== activeStreamId.current) return;
      if (event.payload.result) {
        setSession(event.payload.result.session);
        setLastUsedContext(event.payload.result.used_context);
      }
      setPendingUserMessage(null);
      setStreamingMessage("");
      setStreamingMessages({});
      setSending(false);
      activeStreamId.current = null;
      void refresh();
    });

    return () => {
      void unlistenChunk.then((fn) => fn());
      void unlistenError.then((fn) => fn());
      void unlistenFinish.then((fn) => fn());
    };
  }, [refresh]);

  const selectAgent = useCallback((agentId: string) => {
    setSelectedAgentId(agentId);
    setSelectedAgentIds([agentId]);
    setSession(null);
  }, []);

  const toggleAgent = useCallback((agentId: string) => {
    setSelectedAgentIds((current) => {
      const next = current.includes(agentId)
        ? current.filter((id) => id !== agentId)
        : [...current, agentId];
      const fallback = next.length > 0 ? next : [agentId];
      setSelectedAgentId(fallback[0] ?? null);
      return fallback;
    });
    setSession(null);
  }, []);

  const startSession = useCallback(async () => {
    if (selectedAgents.length === 0) return;
    const next = selectedAgents.length > 1
      ? await invoke<AgentSession>("start_agent_group_session", { agentIds: selectedAgents.map((agent) => agent.plugin_id) })
      : await invoke<AgentSession>("start_agent_session", { agentId: selectedAgents[0].plugin_id });
    setSession(next);
    setSelectedAgentId(next.agent_ids[0] ?? null);
    setSelectedAgentIds(next.agent_ids);
    await refresh();
  }, [refresh, selectedAgents]);

  const loadSession = useCallback(async (sessionId: string) => {
    const next = await invoke<AgentSession>("load_agent_session", { sessionId });
    setSession(next);
    setSelectedAgentId(next.agent_ids[0] ?? null);
    setSelectedAgentIds(next.agent_ids);
  }, []);

  const resetSession = useCallback(async (sessionId: string) => {
    const next = await invoke<AgentSession>("reset_agent_session", { sessionId });
    setSession((current) => current?.session_id === sessionId ? next : current);
    await refresh();
    return next;
  }, [refresh]);

  const deleteSession = useCallback(async (sessionId: string) => {
    const next = await invoke<AgentSession[]>("delete_agent_session", { sessionId });
    setSessions(next);
    setSession((current) => current?.session_id === sessionId ? null : current);
    await refresh();
    return next;
  }, [refresh]);

  const exportTranscript = useCallback(async (sessionId?: string) => {
    const id = sessionId ?? session?.session_id;
    if (!id) return "";
    return invoke<string>("export_agent_transcript", { sessionId: id });
  }, [session]);

  const saveMessageToFile = useCallback(async (messageId: string) => {
    return invoke<string>("save_agent_message_to_file", { messageId });
  }, []);

  const recordMessageToNote = useCallback(async (content: string, title = "Agent reply") => {
    await invoke("add_note", {
      input: {
        title,
        content,
        color: "blue",
      },
    });
  }, []);

  const deleteMessage = useCallback(async (messageId: string) => {
    const next = await invoke<AgentSession>("delete_agent_message", { messageId });
    setSession(next);
    await refresh();
    return next;
  }, [refresh]);

  const sendMessage = useCallback(async (message: string) => {
    if (!message.trim() || selectedAgents.length === 0) return;
    const streamId = `agent-${Date.now()}-${Math.random().toString(36).slice(2)}`;
    const targetAgents = targetAgentsForMessage(message, agents, selectedAgents);
    if (targetAgents.length === 0) return;
    activeStreamId.current = streamId;
    setPendingUserMessage(message.trim());
    setStreamingMessage("");
    setStreamingMessages({});
    setSending(true);
    setError(null);
    try {
      const useGroup = targetAgents.length > 1;
      const result = useGroup
        ? await invoke<SendAgentMessageResult>("send_agent_group_message_stream", {
            input: {
              session_id: session?.session_id ?? null,
              agent_ids: targetAgents.map((agent) => agent.plugin_id),
              message,
              selected_context: DEFAULT_SELECTED_CONTEXT,
              stream_id: streamId,
            },
          })
        : await invoke<SendAgentMessageResult>("send_agent_message_stream", {
            input: {
              session_id: session?.session_id ?? null,
              agent_id: targetAgents[0].plugin_id,
              message,
              selected_context: DEFAULT_SELECTED_CONTEXT,
              stream_id: streamId,
            },
          });
      setSession(result.session);
      setLastUsedContext(result.used_context);
      setPendingUserMessage(null);
      setStreamingMessage("");
      setStreamingMessages({});
      activeStreamId.current = null;
      await refresh();
    } catch (err) {
      setError(String(err));
      activeStreamId.current = null;
    } finally {
      setSending(false);
    }
  }, [agents, refresh, selectedAgents, session]);

  const rebuildRag = useCallback(async () => {
    if (!selectedAgent) return;
    const next = await invoke<AgentRagStatus>("rebuild_agent_rag_index", { pluginId: selectedAgent.plugin_id });
    setRagStatus(next);
  }, [selectedAgent]);

  const refreshAgents = useCallback(async () => {
    const next = await invoke<AgentPlugin[]>("refresh_agents");
    setAgents(next);
    return next;
  }, []);

  const setAgentEnabled = useCallback(async (pluginId: string, enabled: boolean) => {
    await invoke<void>("set_agent_enabled", { pluginId, enabled });
    await refresh();
  }, [refresh]);

  const installPluginZip = useCallback(async (zipPath: string) => {
    const plugin = await invoke<AgentPlugin>("install_agent_plugin_zip", {
      input: { zip_path: zipPath },
    });
    await refresh();
    return plugin;
  }, [refresh]);

  const uninstallPlugin = useCallback(async (pluginId: string) => {
    const next = await invoke<AgentPlugin[]>("uninstall_agent_plugin", { pluginId });
    setAgents(next);
    await refresh();
    return next;
  }, [refresh]);

  const loadPluginDetail = useCallback(async (pluginId: string) => {
    return invoke<AgentPluginDetail>("get_agent_plugin_detail", { pluginId });
  }, []);

  const saveIdentity = useCallback(async (input: SaveAgentUserIdentity) => {
    const next = await invoke<AgentUserIdentity>("save_agent_user_identity", { input });
    setIdentity(next);
    return next;
  }, []);

  const saveMemory = useCallback(async (input: SaveAgentMemory) => {
    const next = await invoke<AgentMemory>("save_agent_memory", { input });
    if (selectedAgent) {
      const list = await invoke<AgentMemory[]>("list_agent_memories", { agentId: selectedAgent.plugin_id });
      setMemories(list);
    }
    return next;
  }, [selectedAgent]);

  const savePluginDirectorySettings = useCallback(async (input: SaveAgentPluginDirectorySettings) => {
    const next = await invoke<AgentPluginDirectorySettings>("save_agent_plugin_directory_settings", { input });
    setPluginDirectorySettings(next);
    await refresh();
    return next;
  }, [refresh]);

  const saveSafeFileRootSettings = useCallback(async (input: SaveAgentSafeFileRootSettings) => {
    const next = await invoke<AgentSafeFileRootSettings>("save_agent_safe_file_root_settings", { input });
    setSafeFileRootSettings(next);
    return next;
  }, []);

  const confirmToolAction = useCallback(async (actionId: string, accepted: boolean) => {
    const result = await invoke<ConfirmAgentToolActionResult>("confirm_agent_tool_action", {
      input: { action_id: actionId, accepted },
    });
    const nextActions = await invoke<AgentToolAction[]>("list_pending_agent_tool_actions");
    setPendingToolActions(nextActions);
    await refresh();
    return result;
  }, [refresh]);

  const confirmMemoryProposal = useCallback(async (input: ConfirmAgentMemoryProposalInput) => {
    const result = await invoke<AgentMemoryProposal>("confirm_agent_memory_proposal", { input });
    const nextProposals = await invoke<AgentMemoryProposal[]>("list_agent_memory_proposals");
    setMemoryProposals(nextProposals);
    if (selectedAgent) {
      const list = await invoke<AgentMemory[]>("list_agent_memories", { agentId: selectedAgent.plugin_id });
      setMemories(list);
    }
    await refresh();
    return result;
  }, [refresh, selectedAgent]);

  const runSecretaryMigration = useCallback(async () => {
    const next = await invoke<AgentMigrationStatus>("run_agent_secretary_migration");
    setMigrationStatus(next);
    await refresh();
    return next;
  }, [refresh]);

  const installExternalCliPresets = useCallback(async () => {
    const next = await invoke<AgentExternalCliTool[]>("install_agent_external_cli_presets");
    setExternalCliTools(next);
    await refresh();
    return next;
  }, [refresh]);

  const saveExternalCliTool = useCallback(async (input: SaveAgentExternalCliTool) => {
    const tool = await invoke<AgentExternalCliTool>("save_agent_external_cli_tool", { input });
    const tools = await invoke<AgentExternalCliTool[]>("list_agent_external_cli_tools");
    setExternalCliTools(tools);
    return tool;
  }, []);

  const testExternalCliTool = useCallback(async (input: SaveAgentExternalCliTool) => {
    return invoke<AgentExternalCliToolTestResult>("test_agent_external_cli_tool_registration", { input });
  }, []);

  const setExternalCliToolEnabled = useCallback(async (toolId: string, enabled: boolean) => {
    const tool = await invoke<AgentExternalCliTool>("set_agent_external_cli_tool_enabled", { toolId, enabled });
    const tools = await invoke<AgentExternalCliTool[]>("list_agent_external_cli_tools");
    setExternalCliTools(tools);
    return tool;
  }, []);

  const deleteExternalCliTool = useCallback(async (toolId: string) => {
    await invoke<void>("delete_agent_external_cli_tool", { toolId });
    const tools = await invoke<AgentExternalCliTool[]>("list_agent_external_cli_tools");
    setExternalCliTools(tools);
  }, []);

  return {
    agents,
    sessions,
    session,
    identity,
    memories,
    memoryProposals,
    pluginDirectorySettings,
    safeFileRootSettings,
    migrationStatus,
    builtinTools,
    externalCliTools,
    pendingToolActions,
    lastUsedContext,
    selectedAgent,
    selectedAgentId,
    selectedAgents,
    selectedAgentIds,
    ragStatus,
    loading,
    sending,
    error,
    pendingUserMessage,
    streamingMessage,
    streamingMessages,
    refresh,
    selectAgent,
    toggleAgent,
    startSession,
    loadSession,
    resetSession,
    deleteSession,
    exportTranscript,
    saveMessageToFile,
    recordMessageToNote,
    deleteMessage,
    sendMessage,
    rebuildRag,
    refreshAgents,
    setAgentEnabled,
    installPluginZip,
    uninstallPlugin,
    loadPluginDetail,
    saveIdentity,
    saveMemory,
    savePluginDirectorySettings,
    saveSafeFileRootSettings,
    confirmToolAction,
    confirmMemoryProposal,
    runSecretaryMigration,
    installExternalCliPresets,
    saveExternalCliTool,
    testExternalCliTool,
    setExternalCliToolEnabled,
    deleteExternalCliTool,
  };
}

export type AgentsController = ReturnType<typeof useAgents>;

function targetAgentsForMessage(
  message: string,
  allAgents: AgentPlugin[],
  selectedAgents: AgentPlugin[],
): AgentPlugin[] {
  const lowerMessage = message.toLowerCase();
  const mentioned = allAgents.filter((agent) => {
    const idMention = `@${agent.plugin_id.toLowerCase()}`;
    const nameMention = `@${agent.plugin_name.toLowerCase()}`;
    return lowerMessage.includes(idMention) || lowerMessage.includes(nameMention);
  });
  const selectedIds = new Set(selectedAgents.map((agent) => agent.plugin_id));
  const selectedMentions = mentioned.filter((agent) => selectedIds.has(agent.plugin_id));
  if (selectedMentions.length > 0) return selectedMentions;
  if (mentioned.length > 0) return mentioned.filter((agent) => agent.enabled);
  return selectedAgents;
}
